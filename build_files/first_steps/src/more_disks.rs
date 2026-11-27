// The "Other disks" tool (more.rs, disks.rs): one switch per disk, then
// Apply. Apply asks for the administrator password once (pkexec,
// celenn-tools-helper disks), then says, under the list, what could not
// be done and why.

use super::{Message, Page, refresh};
use crate::common::state_icon;
use crate::disks::{self, Disk, Status};
use crate::i18n::tr;
use cosmic::app::Task;
use cosmic::iced::core::text::Wrapping;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, settings, text};
use cosmic::{Element, cosmic_theme, theme};
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
pub enum DisksMessage {
    /// The tool was opened: the switches show what is set now
    Open,
    Switch(String, bool),
    Apply,
    /// The helper's output, and whether everything was done (None: the
    /// password prompt was dismissed)
    Done(Option<(bool, String)>),
}

/// The list: the disks switched on, what Apply said
#[derive(Default)]
pub struct DisksView {
    on: BTreeSet<String>,
    applying: bool,
    notes: Vec<String>,
}

fn switched_on(disks: &[Disk]) -> BTreeSet<String> {
    disks
        .iter()
        .filter(|disk| matches!(disk.status, Status::On { .. }))
        .map(|disk| disk.uuid.clone())
        .collect()
}

/// pkexec celenn-tools-helper disks UUID=on|off...
async fn apply(changes: Vec<String>) -> Option<(bool, String)> {
    let output = tokio::process::Command::new("pkexec")
        .arg(disks::HELPER)
        .arg("disks")
        .args(changes)
        .stdin(std::process::Stdio::null())
        .output()
        .await
        .ok()?;
    // pkexec: 126 the prompt dismissed, 127 not authorized
    if matches!(output.status.code(), Some(126 | 127)) {
        return None;
    }
    Some((
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
    ))
}

impl Page {
    pub(super) fn disks_update(&mut self, message: DisksMessage) -> Task<Message> {
        let disks = self.state.as_ref().map(|state| state.disks.as_slice());
        match message {
            DisksMessage::Open => {
                if !self.disks.applying {
                    self.disks.notes.clear();
                    self.disks.on = switched_on(disks.unwrap_or_default());
                }
            }
            DisksMessage::Switch(uuid, on) => {
                if on {
                    self.disks.on.insert(uuid);
                } else {
                    self.disks.on.remove(&uuid);
                }
            }
            DisksMessage::Apply => {
                let now = switched_on(disks.unwrap_or_default());
                let changes: Vec<String> = disks
                    .unwrap_or_default()
                    .iter()
                    .filter(|disk| disk.switchable())
                    .filter_map(|disk| {
                        let wanted = self.disks.on.contains(&disk.uuid);
                        (wanted != now.contains(&disk.uuid))
                            .then(|| format!("{}={}", disk.uuid, if wanted { "on" } else { "off" }))
                    })
                    .collect();
                if changes.is_empty() {
                    return Task::none();
                }
                self.disks.applying = true;
                self.disks.notes.clear();
                return Task::perform(apply(changes), |result| {
                    cosmic::Action::App(Message::Disks(DisksMessage::Done(result)))
                });
            }
            DisksMessage::Done(result) => {
                self.disks.applying = false;
                let Some((success, output)) = result else {
                    return Task::none();
                };
                let (notes, windows) = disks::results(&output, disks.unwrap_or_default());
                for uuid in &windows {
                    disks::remember_windows(uuid);
                }
                self.disks.notes = notes;
                let toast = if success && self.disks.notes.is_empty() {
                    self.toast("Done.")
                } else {
                    Task::none()
                };
                // The list then shows the disks as they are now (Page::update,
                // Message::State)
                return Task::batch([toast, refresh()]);
            }
        }
        Task::none()
    }

    /// The disks read again: the switches show what is set now
    pub(super) fn disks_reread(&mut self, disks: &[Disk]) {
        if !self.disks.applying {
            self.disks.on = switched_on(disks);
        }
    }

    /// The list of disks, each with its switch, then Apply
    pub(super) fn disks_list<'a>(&'a self, disks: &'a [Disk]) -> Element<'a, Message> {
        let cosmic_theme::Spacing {
            space_xxs, space_s, ..
        } = theme::spacing();
        if disks.is_empty() {
            return settings::section()
                .add(
                    settings::item_row(vec![
                        state_icon("drive-harddisk-symbolic", false),
                        crate::common::label(tr("Other disks"), disks::summary(disks)),
                    ])
                    .spacing(space_s),
                )
                .into();
        }
        let idle = !self.disks.applying;
        let section = disks.iter().fold(settings::section(), |section, disk| {
            let icon = match disk.status {
                _ if disk.problem() => "dialog-warning-symbolic",
                Status::Encrypted { .. } => "system-lock-screen-symbolic",
                _ => "drive-harddisk-symbolic",
            };
            let control: Element<'a, Message> = if disk.switchable() {
                let uuid = disk.uuid.clone();
                widget::toggler(self.disks.on.contains(&disk.uuid))
                    .on_toggle_maybe(idle.then_some(move |on| {
                        Message::Disks(DisksMessage::Switch(uuid.clone(), on))
                    }))
                    .into()
            } else if disk.status == Status::ByHand {
                widget::space::horizontal().width(0).into()
            } else {
                text::caption(tr("Not offered")).into()
            };
            section.add(
                settings::item_row(vec![
                    state_icon(icon, disk.problem()),
                    crate::common::two_lines(
                        crate::common::row_title(disk.name.clone()).into(),
                        disk.caption(),
                    ),
                    control,
                ])
                .spacing(space_s),
            )
        });
        let changed = self.disks.on != switched_on(disks);
        let apply: Element<'a, Message> = if self.disks.applying {
            widget::row::with_capacity(2)
                .spacing(space_xxs)
                .align_y(Alignment::Center)
                .push(widget::progress_bar::indeterminate_circular().size(20.0))
                .push(text::body(tr("Working…")))
                .into()
        } else {
            widget::button::suggested(tr("Apply"))
                .on_press_maybe(
                    (changed && self.busy.is_none()).then_some(Message::Disks(DisksMessage::Apply)),
                )
                .into()
        };
        let mut column = widget::column::with_capacity(5)
            .spacing(space_s)
            .push(section);
        for note in &self.disks.notes {
            column = column.push(
                widget::row::with_capacity(2)
                    .spacing(space_xxs)
                    .align_y(Alignment::Center)
                    .push(state_icon("dialog-warning-symbolic", true))
                    .push(
                        text::body(note.as_str())
                            .wrapping(Wrapping::Word)
                            .width(Length::Fill),
                    ),
            );
        }
        column
            .push(
                text::caption(tr(
                    "They open at every startup, even before you log in, and always at the same place: Files, Steam and your apps find them. USB drives are not concerned.",
                ))
                .wrapping(Wrapping::Word)
                .width(Length::Fill),
            )
            .push(
                widget::row::with_capacity(3)
                    .spacing(space_s)
                    .align_y(Alignment::Center)
                    .push(widget::space::horizontal().width(Length::Fill))
                    .push(apply),
            )
            .into()
    }
}
