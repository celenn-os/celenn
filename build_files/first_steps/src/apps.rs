// The kinds of apps that cosmic-blue offers to install without a terminal,
// the same in the first steps tour ("Your apps", main.rs) and in cosmic-blue
// Tools (Extra software, more.rs): the same categories in the same order,
// shown as tiles in two columns. A tile opens its list of apps to tick, with
// the recommended ones already ticked the first time: in every category, the
// apps most used on Linux for that need, all free, so that a beginner sees
// which ones Linux people reach for. Everything comes from
// Flathub, for everyone, except the developer tools (Homebrew, devtools.rs).
//
// The order goes from everyday use to the more specialised: office,
// messaging, passwords, then creating (graphics, videos), then games and
// school, then development.

// (Each program uses its part)
#![allow(dead_code)]

use crate::devtools::{self, BITWARDEN, DEV_TOOLS, DevTool, ToolKind};
use crate::i18n::{fill, tr};
use crate::style::dim_caption;
use cosmic::iced::core::text::Wrapping;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, text};
use cosmic::{Apply, Element, theme};
use std::collections::BTreeSet;

const fn flatpak(name: &'static str, app: &'static str) -> DevTool {
    DevTool {
        name,
        kind: ToolKind::Flatpak(app),
    }
}

pub struct Category {
    pub title: &'static str,
    /// A few of its apps, under the title of its tile
    pub caption: &'static str,
    /// Symbolic icon of the icon theme
    pub icon: &'static str,
    /// Its apps, by group: a heading, then the apps
    pub groups: &'static [(&'static str, &'static [DevTool])],
    /// The apps ticked the first time the list opens, when none of them is
    /// installed: the ones most used on Linux for that need, all free of
    /// charge (the tour's apps page and Tools' Extra software page say so,
    /// once, at the top)
    pub recommended: &'static [&'static str],
    /// Said under the list, when it has something to add (Tools says how
    /// the list works first)
    pub note: &'static str,
    /// Said once the apps are installed (Tools)
    pub done: &'static str,
}

/// How the lists of Tools work (the tour only installs)
pub const TICK_NOTE: &str = "Tick the ones you want, untick the ones to remove.";

pub const CATEGORIES: &[Category] = &[
    Category {
        title: "Office",
        caption: "LibreOffice, OnlyOffice…",
        icon: "x-office-document-symbolic",
        groups: &[
            (
                "Office suites",
                &[
                    flatpak("LibreOffice", "org.libreoffice.LibreOffice"),
                    flatpak("OnlyOffice", "org.onlyoffice.desktopeditors"),
                ],
            ),
            (
                "PDF files",
                &[flatpak(
                    "PDF Arranger",
                    "com.github.jeromerobert.pdfarranger",
                )],
            ),
        ],
        recommended: &["LibreOffice"],
        note: "LibreOffice and OnlyOffice open and save Word, Excel and PowerPoint files.",
        done: "Your office apps are ready.",
    },
    Category {
        title: "Messaging and video calls",
        caption: "Signal, Telegram, Element, Zoom…",
        icon: "user-available-symbolic",
        groups: &[
            (
                "Messaging",
                &[
                    flatpak("Signal", "org.signal.Signal"),
                    flatpak("Telegram", "org.telegram.desktop"),
                    flatpak("Element", "im.riot.Riot"),
                ],
            ),
            ("Video calls", &[flatpak("Zoom", "us.zoom.Zoom")]),
        ],
        recommended: &["Signal"],
        note: "",
        done: "Your messaging apps are ready.",
    },
    Category {
        title: "Passwords",
        caption: "Bitwarden, Secrets, KeePassXC",
        icon: "dialog-password-symbolic",
        groups: &[
            ("Synced with your phone", &[flatpak("Bitwarden", BITWARDEN)]),
            (
                "Only on this computer",
                &[
                    flatpak("Secrets", "org.gnome.World.Secrets"),
                    flatpak("KeePassXC", "org.keepassxc.KeePassXC"),
                ],
            ),
        ],
        recommended: &["Bitwarden"],
        note: "Bitwarden: a free account, on this computer, your phone and in Firefox. Secrets and KeePassXC: an encrypted file, no account.",
        done: "Your password manager is ready.",
    },
    Category {
        title: "Graphics and photos",
        caption: "GIMP, Inkscape, Krita, darktable…",
        icon: "applications-graphics-symbolic",
        groups: &[
            (
                "Drawing and editing",
                &[
                    flatpak("GIMP", "org.gimp.GIMP"),
                    flatpak("Inkscape", "org.inkscape.Inkscape"),
                    flatpak("Krita", "org.kde.krita"),
                ],
            ),
            (
                "Photos",
                &[
                    flatpak("darktable", "org.darktable.Darktable"),
                    flatpak("Shotwell", "org.gnome.Shotwell"),
                ],
            ),
            (
                "And also",
                &[
                    flatpak("Blender", "org.blender.Blender"),
                    flatpak("Scribus", "net.scribus.Scribus"),
                ],
            ),
        ],
        recommended: &["GIMP", "Inkscape", "Krita"],
        note: "",
        done: "Your graphics apps are ready.",
    },
    Category {
        title: "Videos and streaming",
        caption: "OBS Studio, Kdenlive, Audacity…",
        icon: "camera-video-symbolic",
        groups: &[
            (
                "Video",
                &[
                    flatpak("OBS Studio", "com.obsproject.Studio"),
                    flatpak("Kdenlive", "org.kde.kdenlive"),
                    flatpak("Shotcut", "org.shotcut.Shotcut"),
                    flatpak("HandBrake", "fr.handbrake.ghb"),
                ],
            ),
            ("Sound", &[flatpak("Audacity", "org.audacityteam.Audacity")]),
        ],
        recommended: &["OBS Studio", "Kdenlive", "Audacity"],
        note: "",
        done: "Your video apps are ready.",
    },
    Category {
        title: "Games",
        caption: "Steam, Heroic, Lutris, Discord…",
        icon: "input-gaming-symbolic",
        groups: &[
            (
                "Stores",
                &[
                    flatpak("Steam", "com.valvesoftware.Steam"),
                    flatpak("Heroic", "com.heroicgameslauncher.hgl"),
                    flatpak("Lutris", "net.lutris.Lutris"),
                ],
            ),
            (
                "Windows games",
                &[
                    flatpak("Bottles", "com.usebottles.bottles"),
                    flatpak("ProtonPlus", "com.vysp3r.ProtonPlus"),
                ],
            ),
            (
                "And also",
                &[
                    flatpak("RetroArch", "org.libretro.RetroArch"),
                    flatpak("Prism Launcher", "org.prismlauncher.PrismLauncher"),
                    flatpak("Discord", "com.discordapp.Discord"),
                ],
            ),
        ],
        recommended: &["Steam", "Heroic"],
        note: "Removing an app keeps your games and saves.",
        done: "Your game apps are ready.",
    },
    Category {
        title: "School and science",
        caption: "GCompris, Stellarium, GeoGebra…",
        icon: "applications-science-symbolic",
        groups: &[
            (
                "For children",
                &[
                    flatpak("GCompris", "org.kde.gcompris"),
                    flatpak("Tux Paint", "org.tuxpaint.Tuxpaint"),
                ],
            ),
            (
                "Science",
                &[
                    flatpak("Stellarium", "org.stellarium.Stellarium"),
                    flatpak("GeoGebra", "org.geogebra.GeoGebra"),
                    flatpak("Kalzium", "org.kde.kalzium"),
                ],
            ),
            ("Studying", &[flatpak("Anki", "net.ankiweb.Anki")]),
        ],
        recommended: &["GCompris", "Stellarium", "GeoGebra"],
        note: "",
        done: "Your school apps are ready.",
    },
    Category {
        title: "Development",
        caption: "VS Code, Docker, Zed, Neovim…",
        icon: "utilities-terminal-symbolic",
        groups: DEV_TOOLS,
        recommended: &["VS Code"],
        note: "Lima: ujust install-dev-tools, in a terminal.",
        done: "The developer tools are ready.",
    },
];

/// The category of the developer tools: its apps are numbered as
/// devtools::dev_tools()
pub const DEVELOPMENT: usize = 7;
/// The password managers: Bitwarden's Firefox extension follows Bitwarden
pub const PASSWORDS: usize = 2;

/// Said under the developer tools while Homebrew is not there
pub const NO_BREW: &str = "Homebrew is not set up yet: only Podman Desktop can be installed now.";

impl Category {
    /// Its apps, numbered in the order of the groups
    pub fn tools(&self) -> impl Iterator<Item = &'static DevTool> + use<> {
        self.groups.iter().flat_map(|(_, tools)| tools.iter())
    }

    /// The numbers of the recommended apps that can be installed now
    /// (Homebrew's need Homebrew)
    pub fn recommended(&self) -> BTreeSet<usize> {
        self.tools()
            .enumerate()
            .filter(|(_, tool)| self.recommended.contains(&tool.name) && available(tool))
            .map(|(i, _)| i)
            .collect()
    }

    /// The names of some of its apps (numbers of tools())
    pub fn names(&self, apps: &BTreeSet<usize>) -> String {
        self.tools()
            .enumerate()
            .filter(|(i, _)| apps.contains(i))
            .map(|(_, tool)| tool.name)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Every text of the categories, for the translation checks
pub fn texts() -> Vec<&'static str> {
    let mut texts = vec!["Chosen: {}", "Installed: {}", NO_BREW];
    for category in CATEGORIES {
        texts.extend([category.title, category.caption, category.done]);
        texts.extend(Some(category.note).filter(|note| !note.is_empty()));
        texts.extend(category.groups.iter().map(|(heading, _)| *heading));
    }
    let mut seen = BTreeSet::new();
    texts.retain(|text| seen.insert(*text));
    texts
}

/// What a tile says under its title: the apps chosen, else the apps
/// installed, else a few of its apps
pub fn tile_caption(
    category: &Category,
    chosen: &BTreeSet<usize>,
    installed: &BTreeSet<usize>,
) -> String {
    let new: BTreeSet<usize> = chosen.difference(installed).copied().collect();
    if !new.is_empty() {
        fill(tr("Chosen: {}"), &category.names(&new))
    } else if !installed.is_empty() {
        fill(tr("Installed: {}"), &category.names(installed))
    } else {
        tr(category.caption).to_owned()
    }
}

/// A category's tile: its symbol, title and caption, and an arrow, as one
/// button that opens its list
pub fn tile<'a, M: Clone + 'static>(
    category: &'static Category,
    caption: String,
    on_press: Option<M>,
) -> Element<'a, M> {
    let cosmic::cosmic_theme::Spacing {
        space_s, space_m, ..
    } = theme::spacing();
    let lines = widget::column::with_capacity(2)
        .spacing(2)
        .push(text::heading(tr(category.title)))
        .push(dim_caption(caption).wrapping(Wrapping::Word))
        .width(Length::Fill);
    widget::row::with_capacity(3)
        .spacing(space_s)
        .align_y(Alignment::Center)
        .push(widget::icon::from_name(category.icon).size(24))
        .push(lines)
        .push(widget::icon::from_name("go-next-symbolic").size(16))
        // Aligned with the icons and buttons of the settings rows
        .padding([0, space_m])
        .height(Length::Fill)
        .apply(widget::button::custom)
        .class(crate::style::tile_style(false))
        .padding(0)
        .width(Length::Fill)
        .height(Length::Fixed(56.0))
        .on_press_maybe(on_press)
        .into()
}

/// Tiles in two columns
pub fn grid<'a, M: 'static>(tiles: Vec<Element<'a, M>>) -> Element<'a, M> {
    let space_xs = theme::spacing().space_xs;
    let mut lines = widget::column::with_capacity(tiles.len().div_ceil(2)).spacing(space_xs);
    let mut tiles = tiles.into_iter().peekable();
    while tiles.peek().is_some() {
        let mut line = widget::row::with_capacity(2).spacing(space_xs);
        let mut count = 0;
        for tile in tiles.by_ref().take(2) {
            line = line.push(tile);
            count += 1;
        }
        if count == 1 {
            line = line.push(widget::space::horizontal().width(Length::Fill));
        }
        lines = lines.push(line);
    }
    lines.width(Length::Fill).into()
}

/// A category's apps by group, each with its checkbox: the group's name on
/// the left, its apps in three columns on the right. `can_tick` says
/// whether an app's box can change (numbers of Category::tools()).
pub fn checklist<'a, M: Clone + 'static>(
    category: &'static Category,
    ticked: &BTreeSet<usize>,
    can_tick: impl Fn(usize, &DevTool) -> bool,
    on_tick: impl Fn(usize, bool) -> M,
) -> Vec<Element<'a, M>> {
    let cosmic::cosmic_theme::Spacing {
        space_xxs, space_s, ..
    } = theme::spacing();
    let mut rows = Vec::with_capacity(category.groups.len());
    let mut next = 0;
    for (heading, tools) in category.groups {
        let mut cells: Vec<Element<'a, M>> = Vec::new();
        for tool in tools.iter() {
            let i = next;
            next += 1;
            let on = ticked.contains(&i);
            let enabled = can_tick(i, tool);
            let checkbox = widget::checkbox(on).on_toggle_maybe(enabled.then(|| {
                let tick = on_tick(i, !on);
                move |_| tick.clone()
            }));
            // The app's name ticks the box too
            let label = text::body(tool.name).apply(widget::mouse_area);
            let label = if enabled {
                label.on_press(on_tick(i, !on))
            } else {
                label
            };
            cells.push(
                widget::row::with_capacity(2)
                    .spacing(space_xxs)
                    .align_y(Alignment::Center)
                    .push(checkbox)
                    .push(label)
                    .width(Length::Fixed(140.0))
                    .into(),
            );
        }
        let mut lines = widget::column::with_capacity(cells.len().div_ceil(3))
            .spacing(space_xxs)
            .width(Length::Fill);
        let mut cells = cells.into_iter().peekable();
        while cells.peek().is_some() {
            let mut line = widget::row::with_capacity(3).spacing(space_s);
            for cell in cells.by_ref().take(3) {
                line = line.push(cell);
            }
            lines = lines.push(line);
        }
        rows.push(
            widget::row::with_capacity(2)
                .spacing(space_s)
                .push(
                    text::heading(tr(heading))
                        .wrapping(Wrapping::Word)
                        .width(Length::Fixed(150.0)),
                )
                .push(lines)
                .padding([space_xxs, space_s])
                .into(),
        );
    }
    rows
}

/// Whether an app can be installed now: Homebrew's need Homebrew
pub fn available(tool: &DevTool) -> bool {
    matches!(tool.kind, ToolKind::Flatpak(_)) || std::path::Path::new(devtools::BREW).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categories_hold_together() {
        assert_eq!(CATEGORIES[DEVELOPMENT].title, "Development");
        assert!(std::ptr::eq(CATEGORIES[DEVELOPMENT].groups, DEV_TOOLS));
        assert!(
            CATEGORIES[PASSWORDS]
                .tools()
                .any(|tool| tool.kind == ToolKind::Flatpak(BITWARDEN))
        );
        let mut apps = BTreeSet::new();
        for category in CATEGORIES {
            // Every recommended app is in its list
            assert!(!category.recommended.is_empty(), "{}", category.title);
            for name in category.recommended {
                assert!(category.tools().any(|tool| tool.name == *name), "{name}");
            }
            // An app is in one list only
            for tool in category.tools() {
                assert!(apps.insert(tool.name), "{}", tool.name);
            }
        }
    }
}
