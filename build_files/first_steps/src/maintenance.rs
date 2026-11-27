// The Maintenance page of Celenn OS Tools (tools.rs): what `ujust` offers
// to keep the computer in shape, as buttons.
//
//   - Disk space: how full the disk is (the Disk Usage Analyzer shows the
//     details) and "Clean up", which runs /usr/libexec/celenn-clean, the
//     same script as `ujust clean-system`: app parts no app uses any more,
//     unused container images, old Homebrew downloads, then (administrator
//     password) the same for everyone's apps and the system's caches. The
//     space it freed shows when it ends.
//   - The trash: how much it holds, "Empty". Files in it for more than 30
//     days go by themselves, and a notification warns when the disk is
//     almost full (housekeeping.rs); two switches turn these off.
//   - The apps that come with Celenn OS (/usr/share/flatpak/preinstall.d):
//     when some were removed, "Reinstall" puts them back, as the first boot
//     installed them.
//   - Firmware updates: opens GNOME Firmware.

use crate::common::{ToastAsk, dim_caption, fill, label, row_title, spawn, state_icon, two_lines};
use crate::failure::{self, Card, CardAction, CardMessage, Outcome};
use crate::housekeeping;
use crate::i18n::tr;
use crate::units::size;
use cosmic::app::Task;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, settings, text};
use cosmic::{Element, cosmic_theme, theme};
use std::collections::BTreeSet;
use std::process::{Command, Stdio};

const CLEAN: &str = "/usr/libexec/celenn-clean";
const PREINSTALL_DIR: &str = "/usr/share/flatpak/preinstall.d";
const DISK_USAGE: &str = "org.gnome.baobab";
const FIRMWARE: &str = "org.gnome.Firmware";

pub const TEXTS: &[&str] = &[
    "Disk space",
    "{used} used of {total}",
    "{} free",
    "See what takes space",
    "Free up space",
    "Removes what nothing uses any more: leftover parts of removed apps, unused container images, old downloads of tools.",
    "Clean up",
    "Cleaning up…",
    "{} freed",
    "Everything was already tidy.",
    "Trash",
    "{} in the trash",
    "The trash is empty.",
    "Empty",
    "Emptying…",
    "Automatically",
    "Delete files that stayed in the trash for more than 30 days",
    "Like on Windows and the Mac. Newer ones can still be put back.",
    "Warn when the disk is almost full",
    "A notification when less than 10 % is free (20 GB at most).",
    "Apps that come with Celenn OS",
    "All of them are installed.",
    "One of them was removed. Put it back?",
    "{} of them were removed. Put them back?",
    "Reinstall",
    "Installing…",
    "The apps are back.",
    "Firmware updates",
    "New versions of the BIOS, SSD, dock… from their makers.",
    "Open",
    "Clean up the system",
];

#[derive(Clone, Debug, Default)]
pub struct Disk {
    pub total: u64,
    pub used: u64,
    pub free: u64,
}

#[derive(Clone, Debug)]
pub struct State {
    pub disk: Option<Disk>,
    /// Apps of the first boot that are not installed (ids)
    pub missing: Vec<String>,
    /// Bytes in the trash
    pub trash: u64,
    pub trash_auto: bool,
    pub disk_warning: bool,
    pub disk_usage_app: bool,
    pub firmware_app: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Busy {
    Cleaning,
    Reinstalling,
    Emptying,
}

#[derive(Clone, Debug)]
pub enum Message {
    State(State),
    CleanUp,
    Reinstall,
    EmptyTrash,
    TrashAuto(bool),
    DiskWarning(bool),
    /// End of a background command (None: the password prompt was
    /// dismissed)
    Done(Busy, Outcome),
    Card(CardMessage),
    Open(&'static str),
    /// A toast for the window to show (tools.rs)
    Toast(ToastAsk<Message>),
}

pub struct Page {
    state: Option<State>,
    busy: Option<Busy>,
    /// Free space before cleaning up, to tell how much it freed
    free_before: Option<u64>,
    /// What did not work, until closed or done again
    card: Option<Card<Busy>>,
}

/// The parts of the page, each in its own tool (tools.rs): disk space and
/// the trash; putting back the apps that come with Celenn OS; firmware
/// updates
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Part {
    Space,
    Repair,
    Firmware,
}

/// Size and use of the disk that holds the home folders and apps (/var:
/// the system's own files are on the same disk)
pub fn disk() -> Option<Disk> {
    let output = Command::new("df")
        .args(["-B1", "--output=size,used,avail", "/var"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let mut numbers = text
        .lines()
        .nth(1)?
        .split_whitespace()
        .map(|n| n.parse::<u64>().ok());
    Some(Disk {
        total: numbers.next()??,
        used: numbers.next()??,
        free: numbers.next()??,
    })
}

/// The flatpak apps installed for everyone
fn installed() -> BTreeSet<String> {
    Command::new("flatpak")
        .args(["list", "--system", "--columns=application"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(|line| line.trim().to_owned())
                .filter(|line| !line.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// The apps of the first boot ([Flatpak Preinstall <id>])
fn preinstall() -> Vec<String> {
    let mut ids = Vec::new();
    for entry in std::fs::read_dir(PREINSTALL_DIR)
        .into_iter()
        .flatten()
        .flatten()
    {
        let Ok(text) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        ids.extend(text.lines().filter_map(|line| {
            line.trim()
                .strip_prefix("[Flatpak Preinstall ")
                .and_then(|rest| rest.strip_suffix(']'))
                .map(str::to_owned)
        }));
        // Themes for older apps are not apps people remove
        ids.retain(|id| !id.starts_with("org.gtk.Gtk3theme."));
    }
    ids.sort();
    ids.dedup();
    ids
}

pub fn read_state() -> State {
    let installed = installed();
    State {
        disk: disk(),
        missing: preinstall()
            .into_iter()
            .filter(|id| !installed.contains(id))
            .collect(),
        trash: housekeeping::trash_size(),
        trash_auto: housekeeping::trash_auto(),
        disk_warning: housekeeping::disk_warning(),
        disk_usage_app: installed.contains(DISK_USAGE),
        firmware_app: installed.contains(FIRMWARE),
    }
}

fn refresh() -> Task<Message> {
    Task::perform(
        async {
            tokio::task::spawn_blocking(read_state)
                .await
                .unwrap_or_else(|_| State {
                    disk: None,
                    missing: Vec::new(),
                    trash: 0,
                    trash_auto: true,
                    disk_warning: true,
                    disk_usage_app: false,
                    firmware_app: false,
                })
        },
        |state| cosmic::Action::App(Message::State(state)),
    )
}

impl Page {
    pub fn new() -> (Self, Task<Message>) {
        let page = Page {
            state: None,
            busy: None,
            free_before: None,
            // A failure as the screenshot test shows it
            card: crate::common::preview("failure").then(|| {
                Card::new(
                    tr("Clean up"),
                    failure::Failure {
                        cause: failure::Cause::Offline,
                        details: "error: Could not resolve hostname\nExit status: 1".to_owned(),
                    },
                    Busy::Cleaning,
                )
            }),
        };
        (page, refresh())
    }

    fn toast(&mut self, message: String) -> Task<Message> {
        crate::common::toast(message, None, Message::Toast)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::State(state) => self.state = Some(state),
            Message::CleanUp => {
                self.card = None;
                self.busy = Some(Busy::Cleaning);
                self.free_before = disk().map(|disk| disk.free);
                return Task::perform(failure::run(CLEAN, Vec::new(), None), |result| {
                    cosmic::Action::App(Message::Done(Busy::Cleaning, result))
                });
            }
            Message::Reinstall => {
                let Some(state) = &self.state else {
                    return Task::none();
                };
                self.card = None;
                self.busy = Some(Busy::Reinstalling);
                let mut args: Vec<String> =
                    ["install", "--system", "--noninteractive", "-y", "flathub"]
                        .map(str::to_owned)
                        .to_vec();
                args.extend(state.missing.iter().cloned());
                return Task::perform(failure::run("flatpak", args, None), |result| {
                    cosmic::Action::App(Message::Done(Busy::Reinstalling, result))
                });
            }
            Message::EmptyTrash => {
                self.card = None;
                self.busy = Some(Busy::Emptying);
                let args = vec!["trash".to_owned(), "--empty".to_owned()];
                return Task::perform(failure::run("gio", args, None), |result| {
                    cosmic::Action::App(Message::Done(Busy::Emptying, result))
                });
            }
            Message::TrashAuto(on) => {
                housekeeping::set_trash_auto(on);
                if let Some(state) = &mut self.state {
                    state.trash_auto = housekeeping::trash_auto();
                }
            }
            Message::DiskWarning(on) => {
                housekeeping::set_disk_warning(on);
                if let Some(state) = &mut self.state {
                    state.disk_warning = housekeeping::disk_warning();
                }
            }
            Message::Done(busy, result) => {
                self.busy = None;
                let toast = match (busy, result) {
                    (_, None) => Task::none(),
                    (_, Some(Err(failure))) => {
                        let title = match busy {
                            Busy::Cleaning => tr("Clean up"),
                            Busy::Reinstalling => tr("Reinstall"),
                            Busy::Emptying => tr("Empty"),
                        };
                        self.card = Some(Card::new(title, failure, busy));
                        Task::none()
                    }
                    (Busy::Cleaning, Some(Ok(()))) => {
                        let freed = self
                            .free_before
                            .zip(disk().map(|disk| disk.free))
                            .map_or(0, |(before, after)| after.saturating_sub(before));
                        // Less than 10 MB: nothing worth saying
                        if freed < 10_000_000 {
                            self.toast(tr("Everything was already tidy.").to_owned())
                        } else {
                            self.toast(fill(tr("{} freed"), &size(freed)))
                        }
                    }
                    (Busy::Reinstalling, Some(Ok(()))) => {
                        self.toast(tr("The apps are back.").to_owned())
                    }
                    (Busy::Emptying, Some(Ok(()))) => {
                        self.toast(tr("The trash is empty.").to_owned())
                    }
                };
                return Task::batch([toast, refresh()]);
            }
            Message::Card(message) => {
                let Some(card) = &mut self.card else {
                    return Task::none();
                };
                match card.update(message) {
                    CardAction::None => {}
                    CardAction::Close => self.card = None,
                    CardAction::Retry(busy) => {
                        self.card = None;
                        return self.update(match busy {
                            Busy::Cleaning => Message::CleanUp,
                            Busy::Reinstalling => Message::Reinstall,
                            Busy::Emptying => Message::EmptyTrash,
                        });
                    }
                }
            }
            Message::Open(app) => spawn("flatpak", &["run", app]),
            // The window shows it (tools.rs)
            Message::Toast(_) => {}
        }
        Task::none()
    }

    /// Whether GNOME Firmware is installed (Update shows it then)
    pub fn has_firmware(&self) -> bool {
        self.state.as_ref().is_some_and(|state| state.firmware_app)
    }

    /// How many of the apps that come with Celenn OS were removed
    pub fn missing(&self) -> usize {
        self.state.as_ref().map_or(0, |state| state.missing.len())
    }

    /// One part of the page (tools.rs gives it its title and frame)
    pub fn view(&self, part: Part) -> Element<'_, Message> {
        let cosmic_theme::Spacing {
            space_xxs, space_s, ..
        } = theme::spacing();
        let idle = self.busy.is_none();
        let mut content = widget::column::with_capacity(6)
            .spacing(space_s)
            .align_x(Alignment::Center);
        // What did not work, in the part it belongs to
        let card_part = |busy: Busy| {
            if busy == Busy::Reinstalling {
                Part::Repair
            } else {
                Part::Space
            }
        };
        if let Some(card) = self
            .card
            .as_ref()
            .filter(|card| card_part(card.job) == part)
        {
            content = content.push(card.view(Message::Card));
        }
        let Some(state) = &self.state else {
            return content.into();
        };
        if part == Part::Firmware {
            return self.firmware(state);
        }
        if part == Part::Repair {
            return content.push(self.apps(state)).into();
        }

        // Disk space
        let mut disk = settings::section().title(tr("Disk space"));
        if let Some(info) = &state.disk {
            let share = if info.total > 0 {
                info.used as f32 / info.total as f32
            } else {
                0.0
            };
            let used = tr("{used} used of {total}")
                .replace("{used}", &size(info.used))
                .replace("{total}", &size(info.total));
            let mut row = vec![
                state_icon("drive-harddisk-symbolic", share >= 0.9),
                widget::column::with_capacity(3)
                    .spacing(space_xxs)
                    .push(row_title(used))
                    .push(
                        widget::progress_bar::determinate_linear(share)
                            .width(Length::Fill)
                            .girth(6),
                    )
                    .push(dim_caption(fill(tr("{} free"), &size(info.free))))
                    .width(Length::Fill)
                    .into(),
            ];
            if state.disk_usage_app {
                row.push(
                    widget::button::standard(tr("See what takes space"))
                        .on_press(Message::Open(DISK_USAGE))
                        .into(),
                );
            }
            disk = disk.add(settings::item_row(row).spacing(space_s));
        }
        let clean: Element<'_, Message> = if self.busy == Some(Busy::Cleaning) {
            widget::row::with_capacity(2)
                .spacing(space_xxs)
                .align_y(Alignment::Center)
                .push(widget::progress_bar::indeterminate_circular().size(20.0))
                .push(text::body(tr("Cleaning up…")))
                .into()
        } else {
            widget::button::suggested(tr("Clean up"))
                .on_press_maybe(idle.then_some(Message::CleanUp))
                .into()
        };
        // The error card above stands for the change that did not work
        // (its Try again does it again): its row makes room for it, so that
        // the page still fits without scrolling
        let failed = self.card.as_ref().map(|card| card.job);
        if failed != Some(Busy::Cleaning) {
            disk = disk.add(
            settings::item_row(vec![
                state_icon("edit-clear-all-symbolic", false),
                label(
                    tr("Clean up the system"),
                    tr("Removes what nothing uses any more: leftover parts of removed apps, unused container images, old downloads of tools.").to_owned(),
                ),
                clean,
            ])
            .spacing(space_s),
            );
        }

        // Trash
        let trash_caption = if state.trash == 0 {
            tr("The trash is empty.").to_owned()
        } else {
            fill(tr("{} in the trash"), &size(state.trash))
        };
        let mut trash_row = vec![
            state_icon("user-trash-symbolic", false),
            label(tr("Trash"), trash_caption),
        ];
        if self.busy == Some(Busy::Emptying) {
            trash_row.push(
                widget::row::with_capacity(2)
                    .spacing(space_xxs)
                    .align_y(Alignment::Center)
                    .push(widget::progress_bar::indeterminate_circular().size(20.0))
                    .push(text::body(tr("Emptying…")))
                    .into(),
            );
        } else if state.trash > 0 {
            trash_row.push(
                widget::button::standard(tr("Empty"))
                    .on_press_maybe(idle.then_some(Message::EmptyTrash))
                    .into(),
            );
        }
        if failed != Some(Busy::Emptying) {
            disk = disk.add(settings::item_row(trash_row).spacing(space_s));
        }
        content = content.push(disk);

        // What happens by itself (housekeeping.rs): set aside while an
        // error card is shown, so that the page still fits without
        // scrolling (it comes back when the card closes)
        if failed.is_none() {
            content = content.push(
                settings::section()
                    .title(tr("Automatically"))
                    .add(
                        settings::item_row(vec![
                            label(
                                tr("Delete files that stayed in the trash for more than 30 days"),
                                tr("Like on Windows and the Mac. Newer ones can still be put back.")
                                    .to_owned(),
                            ),
                            widget::toggler(state.trash_auto)
                                .on_toggle(Message::TrashAuto)
                                .into(),
                        ])
                        .spacing(space_s),
                    )
                    .add(
                        settings::item_row(vec![
                            label(
                                tr("Warn when the disk is almost full"),
                                tr("A notification when less than 10 % is free (20 GB at most).")
                                    .to_owned(),
                            ),
                            widget::toggler(state.disk_warning)
                                .on_toggle(Message::DiskWarning)
                                .into(),
                        ])
                        .spacing(space_s),
                    ),
            );
        }

        content.into()
    }

    /// The apps that come with Celenn OS, and the button that puts back
    /// the ones removed
    fn apps(&self, state: &State) -> Element<'_, Message> {
        let cosmic_theme::Spacing {
            space_xxs, space_s, ..
        } = theme::spacing();
        let idle = self.busy.is_none();
        // Apps that come with Celenn OS
        let caption = if state.missing.is_empty() {
            tr("All of them are installed.").to_owned()
        } else {
            match state.missing.len() {
                1 => tr("One of them was removed. Put it back?").to_owned(),
                count => fill(
                    tr("{} of them were removed. Put them back?"),
                    &count.to_string(),
                ),
            }
        };
        let mut apps_row = vec![
            state_icon("view-app-grid-symbolic", false),
            label(tr("Apps that come with Celenn OS"), caption),
        ];
        if self.busy == Some(Busy::Reinstalling) {
            apps_row.push(
                widget::row::with_capacity(2)
                    .spacing(space_xxs)
                    .align_y(Alignment::Center)
                    .push(widget::progress_bar::indeterminate_circular().size(20.0))
                    .push(text::body(tr("Installing…")))
                    .into(),
            );
        } else if !state.missing.is_empty() {
            apps_row.push(
                widget::button::standard(tr("Reinstall"))
                    .on_press_maybe(idle.then_some(Message::Reinstall))
                    .into(),
            );
        }
        settings::section()
            .add(settings::item_row(apps_row).spacing(space_s))
            .into()
    }

    /// Firmware updates: GNOME Firmware, when it is installed
    fn firmware(&self, state: &State) -> Element<'_, Message> {
        let space_s = theme::spacing().space_s;
        let mut system = settings::section();
        // Firmware
        if state.firmware_app {
            system = system.add(
                settings::item_row(vec![
                    state_icon("application-x-firmware-symbolic", false),
                    two_lines(
                        row_title(tr("Firmware updates")).into(),
                        tr("New versions of the BIOS, SSD, dock… from their makers.").to_owned(),
                    ),
                    widget::button::standard(tr("Open"))
                        .on_press(Message::Open(FIRMWARE))
                        .into(),
                ])
                .spacing(space_s),
            );
        }
        system.into()
    }
}
