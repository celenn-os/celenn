// cosmic-blue Tools, the Overview page (the window's first page): where
// things stand, in one sentence and four cards (system version, backups,
// disk space, firmware), each with the button to its page; and a search
// through every setting of the other pages.
// It only reads: nothing changes from here, and nothing is ever said
// outside the window (no notifications about updates).

use crate::PageId;
use crate::common::{dim_caption, fill, label, row_title, spawn};
use crate::i18n::tr;
use crate::maintenance::{self, Disk};
use crate::updates_page::{self, Summary};
use cosmic::app::Task;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, settings, text};
use cosmic::{Apply, Element, cosmic_theme, theme};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

pub const TEXTS: &[&str] = &[
    "Overview",
    "All is well",
    "One thing needs your attention",
    "{} things need your attention",
    "cosmic-blue looks after itself. Here is where things stand.",
    "System up to date",
    "Version {}. The next one installs by itself in the background.",
    "Update ready",
    "Version {} starts at the next restart.",
    "Backups",
    "Not set up yet. A USB drive is enough: Déjà Dup keeps a copy of your home folder.",
    "Set up",
    "Last backup today.",
    "Last backup {} days ago.",
    "Almost full: {} free.",
    "Nothing new from the makers.",
    "One device has an update.",
    "{} devices have an update.",
    "Search a setting",
    "Nothing found.",
];

/// The icon of the page's header: the app's own
const ICON: &str = "cosmic-blue-tools";
/// A backup older than this many days asks for attention
const BACKUP_DAYS: u64 = 14;

/// Every setting of the other pages that the search finds: its title,
/// words that also find it (its caption), its page. Texts of the pages
/// themselves, so they are already translated.
const SETTINGS: &[(&str, &str, PageId)] = &[
    ("Updates", "", PageId::Updates),
    ("Check now", "", PageId::Updates),
    ("Get new features early", "", PageId::Updates),
    ("An older version", "", PageId::Updates),
    ("What's new", "", PageId::Updates),
    ("Disk space", "", PageId::Maintenance),
    (
        "Free up space",
        "Removes what nothing uses any more: leftover parts of removed apps, unused container images, old downloads of the developer tools. Your files stay.",
        PageId::Maintenance,
    ),
    ("Trash", "", PageId::Maintenance),
    (
        "Delete files that stayed in the trash for more than 30 days",
        "",
        PageId::Maintenance,
    ),
    ("Warn when the disk is almost full", "", PageId::Maintenance),
    ("Apps that come with cosmic-blue", "", PageId::Maintenance),
    (
        "Firmware updates",
        "New versions of the BIOS, SSD, dock… from their makers.",
        PageId::Maintenance,
    ),
    (
        "Battery charge limit",
        "Charging stops at 80 %: the battery lasts longer when the laptop stays plugged in.",
        PageId::Computer,
    ),
    ("NVIDIA graphics card", "", PageId::Computer),
    (
        "Firmware settings (BIOS/UEFI)",
        "Restart into the computer's own settings: boot order, virtualization, Secure Boot…",
        PageId::Computer,
    ),
    (
        "Microsoft fonts",
        "Arial, Times New Roman, Verdana… for documents made on Windows.",
        PageId::Software,
    ),
    ("Games", "Steam, Heroic, Lutris, Discord…", PageId::Software),
    (
        "Passwords",
        "Bitwarden (synced with your phone), Secrets, KeePassXC.",
        PageId::Software,
    ),
    (
        "Virtual machines",
        "virt-manager, to run Windows or another Linux in a window.",
        PageId::Software,
    ),
    (
        "FlatPark",
        "More apps in COSMIC Store, from outside Flathub. Less checked.",
        PageId::Software,
    ),
    (
        "Developer tools",
        "Docker, VS Code, Neovim…",
        PageId::Software,
    ),
    (
        "Dark theme",
        "Light text on dark backgrounds. Off: the light theme.",
        PageId::Appearance,
    ),
    (
        "Layout",
        "Top bar and dock: like the Mac. One bar at the bottom: like Windows.",
        PageId::Appearance,
    ),
    ("Dock always visible", "", PageId::Appearance),
    (
        "More in Settings",
        "Accent color, wallpaper, panel, dock, fonts…",
        PageId::Appearance,
    ),
    ("The cosmic-blue look", "", PageId::Appearance),
    ("All COSMIC settings", "", PageId::Appearance),
    ("Save my settings", "", PageId::Appearance),
    ("Documentation", "", PageId::Help),
    ("Ask a question", "", PageId::Help),
    ("Report a problem", "", PageId::Help),
];

/// At most this many results, so that the page does not scroll
const RESULTS: usize = 8;

#[derive(Clone, Debug)]
pub enum Backup {
    Never,
    /// Days since the last one
    Last(u64),
}

#[derive(Clone, Debug, Default)]
pub struct Facts {
    system: Option<Result<Summary, String>>,
    /// None: Déjà Dup is not installed
    backup: Option<Backup>,
    disk: Option<Disk>,
    /// Devices with a firmware update; None: fwupd did not answer
    firmware: Option<usize>,
}

#[derive(Clone, Debug)]
pub enum Message {
    Facts(Box<Facts>),
    Search(String),
    /// Opens a page (handled by the window, tools.rs)
    Go(PageId),
    Backups,
}

pub struct Page {
    facts: Option<Facts>,
    query: String,
}

fn backup() -> Option<Backup> {
    if !Command::new("sh")
        .args(["-c", "command -v deja-dup"])
        .stdout(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
    {
        return None;
    }
    let output = Command::new("gsettings")
        .args(["get", "org.gnome.DejaDup", "last-backup"])
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let last = String::from_utf8_lossy(&output.stdout)
        .trim()
        .trim_matches('\'')
        .to_owned();
    if last.is_empty() {
        return Some(Backup::Never);
    }
    let output = Command::new("date")
        .args(["-d", &last, "+%s"])
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let then: u64 = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .ok()?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(then, |now| now.as_secs());
    Some(Backup::Last(now.saturating_sub(then) / 86_400))
}

/// The devices with a firmware update, from fwupd's own list (no
/// download: fwupd refreshes it on its own timer)
fn firmware() -> Option<usize> {
    let output = Command::new("fwupdmgr")
        .args(["get-updates", "--json", "--no-unreported-check"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    // 2: nothing to update
    if output.status.code() == Some(2) {
        return Some(0);
    }
    if !output.status.success() {
        return None;
    }
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    Some(
        value["Devices"]
            .as_array()?
            .iter()
            .filter(|device| {
                device["Releases"]
                    .as_array()
                    .is_some_and(|releases| !releases.is_empty())
            })
            .count(),
    )
}

fn read_facts() -> Facts {
    Facts {
        system: Some(updates_page::summary()),
        backup: backup(),
        disk: maintenance::disk(),
        firmware: firmware(),
    }
}

/// Reads the facts again (each time the page is shown)
pub fn refresh() -> Task<Message> {
    Task::perform(
        async {
            tokio::task::spawn_blocking(read_facts)
                .await
                .unwrap_or_default()
        },
        |facts| cosmic::Action::App(Message::Facts(Box::new(facts))),
    )
}

/// Lower case, without accents, so that "theme" finds "Thème"
fn fold(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' => 'a',
            'ç' => 'c',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ñ' => 'n',
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ß' => 's',
            c => c,
        })
        .collect()
}

/// The settings whose title or caption hold every word of the query, in
/// the window's language or in English
fn search(query: &str) -> Vec<(&'static str, PageId)> {
    let words: Vec<String> = fold(query).split_whitespace().map(str::to_owned).collect();
    SETTINGS
        .iter()
        .filter(|(title, caption, _)| {
            let mut haystack = fold(&format!("{} {} {title}", tr(title), tr(caption)));
            if !caption.is_empty() {
                haystack.push(' ');
                haystack.push_str(&fold(caption));
            }
            words.iter().all(|word| haystack.contains(word.as_str()))
        })
        .map(|(title, _, page)| (*title, *page))
        .take(RESULTS)
        .collect()
}

/// A page's name, as in the sidebar
pub fn page_name(page: PageId) -> &'static str {
    match page {
        PageId::Overview => "Overview",
        PageId::Updates => "Updates",
        PageId::Maintenance => "Maintenance",
        PageId::Computer => "Computer",
        PageId::Software => "Extra software",
        PageId::Appearance => "Appearance",
        PageId::Help => "Help",
    }
}

/// One card: its state (fine or needs attention), its title, what it is
/// about, maybe a bar, and the button to its page
struct Card {
    fine: bool,
    title: String,
    caption: String,
    share: Option<f32>,
    button: &'static str,
    suggested: bool,
    message: Message,
}

impl Page {
    pub fn new() -> (Self, Task<Message>) {
        (
            Page {
                facts: None,
                query: String::new(),
            },
            refresh(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Facts(facts) => self.facts = Some(*facts),
            Message::Search(query) => self.query = query,
            Message::Backups => spawn("deja-dup", &[]),
            // tools.rs opens the page
            Message::Go(_) => {}
        }
        Task::none()
    }

    fn cards(facts: &Facts) -> Vec<Card> {
        let mut cards = Vec::with_capacity(5);
        match &facts.system {
            Some(Ok(system)) => cards.push(match (&system.staged, system.paused) {
                (_, true) => Card {
                    fine: false,
                    title: tr("Updates are paused").to_owned(),
                    caption: tr("This system stays on this version. Resume them to get one tested update a week again.").to_owned(),
                    share: None,
                    button: "Updates",
                    suggested: false,
                    message: Message::Go(PageId::Updates),
                },
                (Some(staged), false) => Card {
                    fine: true,
                    title: tr("Update ready").to_owned(),
                    caption: fill(tr("Version {} starts at the next restart."), staged),
                    share: None,
                    button: "Updates",
                    suggested: false,
                    message: Message::Go(PageId::Updates),
                },
                (None, false) => Card {
                    fine: true,
                    title: tr("System up to date").to_owned(),
                    caption: fill(
                        tr("Version {}. The next one installs by itself in the background."),
                        &system.version,
                    ),
                    share: None,
                    button: "Updates",
                    suggested: false,
                    message: Message::Go(PageId::Updates),
                },
            }),
            Some(Err(_)) => cards.push(Card {
                fine: false,
                title: tr("Updates").to_owned(),
                caption: tr("The system version could not be read: is this computer installed with cosmic-blue?").to_owned(),
                share: None,
                button: "Updates",
                suggested: false,
                message: Message::Go(PageId::Updates),
            }),
            None => {}
        }
        match facts.backup {
            Some(Backup::Never) => cards.push(Card {
                fine: false,
                title: tr("Backups").to_owned(),
                caption: tr("Not set up yet. A USB drive is enough: Déjà Dup keeps a copy of your home folder.").to_owned(),
                share: None,
                button: "Set up",
                suggested: true,
                message: Message::Backups,
            }),
            Some(Backup::Last(days)) => cards.push(Card {
                fine: days <= BACKUP_DAYS,
                title: tr("Backups").to_owned(),
                caption: if days == 0 {
                    tr("Last backup today.").to_owned()
                } else {
                    fill(tr("Last backup {} days ago."), &days.to_string())
                },
                share: None,
                button: "Open",
                suggested: false,
                message: Message::Backups,
            }),
            None => {}
        }
        if let Some(disk) = &facts.disk {
            let share = if disk.total > 0 {
                disk.used as f32 / disk.total as f32
            } else {
                0.0
            };
            let fine = share < 0.9;
            cards.push(Card {
                fine,
                title: tr("Disk space").to_owned(),
                caption: if fine {
                    fill(tr("{} free"), &maintenance::size(disk.free))
                } else {
                    fill(tr("Almost full: {} free."), &maintenance::size(disk.free))
                },
                share: Some(share),
                button: "Maintenance",
                suggested: !fine,
                message: Message::Go(PageId::Maintenance),
            });
        }
        if let Some(devices) = facts.firmware {
            cards.push(Card {
                fine: devices == 0,
                title: tr("Firmware updates").to_owned(),
                caption: match devices {
                    0 => tr("Nothing new from the makers.").to_owned(),
                    1 => tr("One device has an update.").to_owned(),
                    n => fill(tr("{} devices have an update."), &n.to_string()),
                },
                share: None,
                button: "Maintenance",
                suggested: false,
                message: Message::Go(PageId::Maintenance),
            });
        }
        // Fewer than four: help fills the grid
        if cards.len() < 4 {
            cards.push(Card {
                fine: true,
                title: tr("Help").to_owned(),
                caption: tr("Find answers, ask a question, or tell us what goes wrong.").to_owned(),
                share: None,
                button: "Open",
                suggested: false,
                message: Message::Go(PageId::Help),
            });
        }
        cards
    }

    fn card(card: Card) -> Element<'static, Message> {
        let cosmic_theme::Spacing {
            space_xxs,
            space_xs,
            space_s,
            space_m,
            ..
        } = theme::spacing();
        let fine = card.fine;
        let symbol = widget::icon::from_name(if fine {
            "object-select-symbolic"
        } else {
            "dialog-warning-symbolic"
        })
        .size(20)
        .apply(widget::container)
        .class(theme::Container::custom(move |theme| {
            let cosmic = theme.cosmic();
            cosmic::iced::widget::container::Style {
                icon_color: Some(
                    if fine {
                        cosmic.accent_color()
                    } else {
                        cosmic.warning_color()
                    }
                    .into(),
                ),
                ..Default::default()
            }
        }));
        let button = if card.suggested {
            widget::button::suggested(tr(card.button))
        } else {
            widget::button::standard(tr(card.button))
        }
        .on_press(card.message);
        widget::column::with_capacity(5)
            .spacing(space_xs)
            .push(
                widget::row::with_capacity(2)
                    .spacing(space_xs)
                    .align_y(Alignment::Center)
                    .push(symbol)
                    .push(row_title(card.title)),
            )
            .push_maybe(card.share.map(|share| {
                widget::progress_bar::determinate_linear(share)
                    .width(Length::Fill)
                    .girth(6)
            }))
            .push(
                dim_caption(card.caption)
                    .wrapping(cosmic::iced::core::text::Wrapping::Word)
                    .height(Length::Fill),
            )
            .push(button)
            .apply(widget::container)
            .padding([space_s, space_m])
            .width(Length::Fill)
            .height(168)
            .class(theme::Container::Card)
            .apply(widget::container)
            .padding([0, 0, space_xxs, 0])
            .into()
    }

    fn results(&self) -> Element<'_, Message> {
        let found = search(&self.query);
        if found.is_empty() {
            return text::body(tr("Nothing found.")).into();
        }
        let mut section = settings::section();
        for (title, page) in found {
            section = section.add(
                settings::item_row(vec![
                    label(title, tr(page_name(page)).to_owned()),
                    widget::button::standard(tr("Open"))
                        .on_press(Message::Go(page))
                        .into(),
                ])
                .align_y(Alignment::Center),
            );
        }
        section.into()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let cosmic_theme::Spacing {
            space_xxs,
            space_s,
            space_m,
            ..
        } = theme::spacing();
        let cards = self.facts.as_ref().map(Self::cards).unwrap_or_default();
        let attention: Vec<&Card> = cards.iter().filter(|card| !card.fine).collect();
        let title = match attention.len() {
            0 => tr("All is well").to_owned(),
            1 => tr("One thing needs your attention").to_owned(),
            n => fill(tr("{} things need your attention"), &n.to_string()),
        };
        // What needs attention, by name (each card says why)
        let caption = if attention.is_empty() {
            tr("cosmic-blue looks after itself. Here is where things stand.").to_owned()
        } else {
            attention
                .iter()
                .map(|card| card.title.as_str())
                .collect::<Vec<_>>()
                .join(" · ")
        };
        let search = widget::search_input(tr("Search a setting"), self.query.as_str())
            .on_input(Message::Search)
            .on_clear(Message::Search(String::new()))
            .width(Length::Fill);
        let body: Element<'_, Message> = if self.query.trim().is_empty() {
            let mut grid = widget::column::with_capacity(2).spacing(space_s);
            let mut cards = cards.into_iter();
            while let Some(first) = cards.next() {
                let second = cards.next();
                grid = grid.push(
                    widget::row::with_capacity(2)
                        .spacing(space_s)
                        .push(Self::card(first))
                        .push_maybe(second.map(Self::card)),
                );
            }
            grid.into()
        } else {
            self.results()
        };
        widget::column::with_capacity(6)
            .push(widget::space::vertical().height(space_xxs))
            .push(crate::style::page_header(
                widget::icon::from_name(ICON).size(44).into(),
                if self.facts.is_some() {
                    title
                } else {
                    tr("Overview").to_owned()
                },
                self.facts.as_ref().map(|_| caption),
            ))
            .push(widget::space::vertical().height(space_m))
            .push(search)
            .push(widget::space::vertical().height(space_m))
            .push(body)
            .max_width(640.0)
            .width(640.0)
            .apply(widget::container)
            .center_x(Length::Fill)
            .apply(widget::scrollable)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_accents() {
        assert_eq!(fold("Thème Sombre"), "theme sombre");
    }

    #[test]
    fn finds_settings() {
        let pages = |query| {
            search(query)
                .into_iter()
                .map(|(_, page)| page)
                .collect::<Vec<_>>()
        };
        assert_eq!(pages("steam"), [PageId::Software]);
        assert!(pages("dark").contains(&PageId::Appearance));
        assert!(pages("BIOS").contains(&PageId::Computer));
        assert!(pages("nothing like this").is_empty());
        // Every word counts
        assert_eq!(search("battery limit").len(), 1);
    }
}
