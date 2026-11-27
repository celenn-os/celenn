// The first steps tour's content: where the person comes from, what each
// page says and offers (rows, buttons, lists), for the tour, the live ISO's
// welcome tour and the small dialogs (main.rs runs them, tour_view.rs
// draws them).

use super::*;

/// Where the person comes from (first page of the first steps tour), kept
/// in ~/.config/celenn/tour-origin for the next time
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Origin {
    Windows,
    Mac,
    Linux,
}

impl Origin {
    pub(super) const ALL: [Origin; 3] = [Origin::Windows, Origin::Mac, Origin::Linux];

    pub(super) fn key(self) -> &'static str {
        match self {
            Origin::Windows => "windows",
            Origin::Mac => "macos",
            Origin::Linux => "linux",
        }
    }

    pub(super) fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|origin| origin.key() == key)
    }

    /// Icon, name and what the tour does for them
    pub(super) fn card(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Origin::Windows => ("computer-symbolic", "Windows", "Your bearings from Windows"),
            Origin::Mac => (
                "computer-laptop-symbolic",
                "macOS",
                "Your bearings from the Mac",
            ),
            Origin::Linux => (
                "utilities-terminal-symbolic",
                "Linux",
                "What is different from your distribution",
            ),
        }
    }
}

/// tour-level's value for the short tour
pub(super) const EXPERIENCED: &str = "experienced";

/// Things to try on the "Try it" page: each button does the real thing
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Try {
    Launcher,
    AppLibrary,
    Workspaces,
    Tiling,
    Screenshot,
}

impl Try {
    pub(super) const ALL: [Try; 5] = [
        Try::Launcher,
        Try::AppLibrary,
        Try::Workspaces,
        Try::Tiling,
        Try::Screenshot,
    ];

    pub(super) fn texts(self) -> (&'static str, &'static str) {
        match self {
            Try::Launcher => (
                "Open the launcher",
                "Press Super, type the name of an app, then Enter.",
            ),
            Try::AppLibrary => ("See all your apps", "Super + A"),
            Try::Workspaces => (
                "See your workspaces",
                "Super + W: one desktop per activity. Super + Ctrl + arrows goes from one to the next.",
            ),
            Try::Tiling => (
                "Tile your windows",
                "Windows side by side, without overlap. Super + Y turns it on or off.",
            ),
            Try::Screenshot => (
                "Take a screenshot",
                "Super + Shift + S, or the Print Screen key",
            ),
        }
    }

    /// The COSMIC program behind the shortcut (cosmic-settings-daemon's
    /// system actions)
    pub(super) fn program(self) -> Option<&'static str> {
        match self {
            Try::Launcher => Some("cosmic-launcher"),
            Try::AppLibrary => Some("cosmic-app-library"),
            Try::Workspaces => Some("cosmic-workspaces"),
            Try::Screenshot => Some("cosmic-screenshot"),
            Try::Tiling => None,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) enum Action {
    /// A program and its arguments
    Launch(&'static [&'static str]),
    OpenUri(String),
    Install,
    /// A "Try it" button in a row: does the real thing (Message::Try)
    Try(Try),
}

/// A row: title, caption and maybe a button
pub(super) struct Row {
    pub(super) title: &'static str,
    pub(super) caption: &'static str,
    pub(super) button: Option<(&'static str, Action)>,
}

pub(super) const fn row(title: &'static str, caption: &'static str) -> Row {
    Row {
        title,
        caption,
        button: None,
    }
}

pub(super) fn row_button(
    title: &'static str,
    caption: &'static str,
    label: &'static str,
    action: Action,
) -> Row {
    Row {
        title,
        caption,
        button: Some((label, action)),
    }
}

/// What a page shows under its description
pub(super) enum Body {
    /// The welcome illustration (HERO, or the logo when it is missing),
    /// with the accessibility row under it (live tour)
    Welcome,
    /// Where the person comes from, the accessibility row and the progress
    /// of the first-boot apps (first steps tour)
    Origin,
    /// Theme, wallpaper, avatar, dock: applied at once
    Look,
    /// The kinds of apps, as tiles that open a list to tick (apps.rs)
    Profiles,
    /// The experienced tour's page: what is different here (rows), then
    /// the password managers and developer tools tiles
    Essentials(Vec<Row>),
    /// Buttons that do the real thing
    Try,
    /// What was done, what the computer needs, backups and help
    Ready,
    /// Title and subtitle rows, without action
    Features(Vec<(&'static str, &'static str)>),
    /// Title and subtitle rows, some with a button
    Rows(Vec<Row>),
    /// Live session: what this computer has, checked (readiness.rs)
    Readiness,
    /// Live session's last page: the checks to look at before installing,
    /// and the install guide
    Install,
}

pub(super) struct Page {
    /// Icon of the icon theme shown above the title on the BANNER sky, in
    /// color like the app icons (Celenn OS's Tela circle theme)
    pub(super) icon: Option<&'static str>,
    pub(super) title: &'static str,
    pub(super) description: Option<&'static str>,
    pub(super) body: Body,
    /// Button under the rows
    pub(super) button_after: Option<(&'static str, Action)>,
    /// Short dimmed line at the bottom
    pub(super) note: Option<&'static str>,
}

impl Page {
    pub(super) fn new(title: &'static str, description: Option<&'static str>, body: Body) -> Self {
        Self {
            icon: None,
            title,
            description,
            body,
            button_after: None,
            note: None,
        }
    }
}

/// The Celenn OS site in the session language
/// The website's folder of the session language ("" for English)
pub(super) fn site_language() -> &'static str {
    match language() {
        Some("pt_BR") => "pt-br/",
        Some("fr") => "fr/",
        Some("de") => "de/",
        Some("es") => "es/",
        Some("it") => "it/",
        _ => "",
    }
}

pub(super) fn docs_url() -> String {
    format!("{SITE}{}docs/", site_language())
}

/// The install guide, at one of its steps ("disk") or at the top ("")
pub(super) fn install_url(step: &str) -> String {
    let anchor = if step.is_empty() {
        String::new()
    } else {
        format!("#{step}")
    };
    format!("{SITE}{}install/{anchor}", site_language())
}

pub(super) const DISTROSHELF: &[&str] = &["flatpak", "run", "com.ranfdev.DistroShelf"];
pub(super) const FLATSEAL: &[&str] = &["flatpak", "run", "com.github.tchx84.Flatseal"];
pub(super) const FIREFOX: &[&str] = &["flatpak", "run", "org.mozilla.firefox"];
pub(super) const THUNDERBIRD: &[&str] = &["flatpak", "run", "org.mozilla.thunderbird"];
pub(super) const FIRMWARE: &[&str] = &["flatpak", "run", "org.gnome.Firmware"];

/// What is different on Celenn OS for someone who knows Linux: the
/// Linux bearings, and the first rows of the experienced tour
pub(super) fn linux_rows() -> Vec<Row> {
    vec![
        row_button(
            "The system is an image",
            "Signed and read-only, it updates as a whole: no dnf on the system, and the previous version is one click away.",
            "Updates",
            Action::Launch(&[UPDATES_APP]),
        ),
        row_button(
            "Apps → COSMIC Store",
            "Flatpak apps from Flathub, kept apart from the system",
            "Open COSMIC Store",
            Action::Launch(&["cosmic-store"]),
        ),
        row_button(
            "Packages from other distributions → DistroShelf",
            "Arch, Debian, Ubuntu… in a container that shares your home folder, set up in a few clicks",
            "Open DistroShelf",
            Action::Launch(DISTROSHELF),
        ),
        row_button(
            "If you like the terminal",
            "ujust lists the system's recipes, brew installs command line tools: the documentation has them all.",
            "Documentation",
            Action::OpenUri(docs_url()),
        ),
    ]
}

/// Where everything is, for someone new: each row has its "Try" button
/// when there is something to try (the "Try it" page of the tour when
/// nobody said where they come from)
pub(super) fn bearings(origin: Origin) -> Page {
    let rows = match origin {
        Origin::Linux => linux_rows(),
        Origin::Mac => vec![
            row(
                "Command key → Ctrl, most of the time",
                "Copy, paste, save: Ctrl + C, Ctrl + V, Ctrl + S",
            ),
            row_button(
                "Spotlight → the Super key",
                "Super opens the launcher: type the name of an app",
                "Try",
                Action::Try(Try::Launcher),
            ),
            row_button(
                "App Store → COSMIC Store",
                "Apps install and update from there",
                "Open",
                Action::Launch(&["cosmic-store"]),
            ),
            row_button(
                "Finder → Files",
                "Super + E opens it",
                "Try",
                Action::Launch(&["cosmic-files"]),
            ),
            row_button(
                "Mission Control → Super + W",
                "Your windows and workspaces at a glance",
                "Try",
                Action::Try(Try::Workspaces),
            ),
            row(
                "System Settings → Settings",
                "Everything in one place, Settings → Desktop for the look",
            ),
        ],
        Origin::Windows => vec![
            row_button(
                "Windows key → the Super key",
                "It opens the launcher, like the Start menu: type the name of an app",
                "Try",
                Action::Try(Try::Launcher),
            ),
            row_button(
                "Task View → Super + W",
                "Your windows and workspaces at a glance",
                "Try",
                Action::Try(Try::Workspaces),
            ),
            row_button(
                "Microsoft Store → COSMIC Store",
                "Apps install and update from there",
                "Open",
                Action::Launch(&["cosmic-store"]),
            ),
            row_button(
                "File Explorer → Files",
                "Super + E opens it, as on Windows",
                "Try",
                Action::Launch(&["cosmic-files"]),
            ),
            row(
                "Office → LibreOffice",
                "It opens and saves Word, Excel and PowerPoint files",
            ),
            row_button(
                "Same as on Windows",
                "Alt + Tab, Ctrl + C and Ctrl + V, Super + Shift + S for a screenshot, Ctrl + Shift + Esc for the task manager",
                "Try",
                Action::Try(Try::Screenshot),
            ),
        ],
    };
    // No banner, like the Look page: six rows and a note need the room
    Page {
        note: Some(SHORTCUTS_NOTE),
        ..Page::new(
            match origin {
                Origin::Mac => "Your bearings from the Mac",
                Origin::Windows => "Your bearings from Windows",
                Origin::Linux => "What is different here",
            },
            Some(match origin {
                Origin::Linux => "Another Linux, with its own habits, and an app for each of them.",
                _ => "What changes, and what stays the same. Try it, it is safe.",
            }),
            Body::Rows(rows),
        )
    }
}

/// Under the bearings: where the other shortcuts are
pub(super) const SHORTCUTS_NOTE: &str = "Every shortcut, tiling included: Super + F1.";

/// The page of the experienced Linux user's tour: what is different here,
/// then the developer tools and the password managers (Body::Essentials
/// adds them)
pub(super) fn essentials() -> Page {
    // No banner: the rows need the room
    Page {
        note: Some(
            "Office, games, graphics and more: COSMIC Store, or Celenn OS Tools → Install apps.",
        ),
        ..Page::new(
            "The essentials",
            Some("What is different here, and your tools."),
            Body::Essentials(linux_rows()),
        )
    }
}

/// The rows of the "Bring your things" window, made once
pub(super) static BELONGINGS: std::sync::LazyLock<Vec<Row>> =
    std::sync::LazyLock::new(belongings_rows);

/// "Bring your things" (--belongings, from the last page of the tour)
pub(super) fn belongings_rows() -> Vec<Row> {
    vec![
        row_button(
            "Files from an old disk",
            "Plug it in: it shows up in Files, on the left. A Windows partition of this computer too.",
            "Open Files",
            Action::Launch(&["cosmic-files"]),
        ),
        row_button(
            "Photos from a phone",
            "Without a cable, with LocalSend. Or plug it in and allow file transfer on the phone: it shows up in Files.",
            "Without a cable",
            Action::Launch(&[TOUR, "--phone"]),
        ),
        row_button(
            "From another Celenn OS computer",
            "Your settings and the list of your apps, with the file saved there in Celenn OS Tools → Change computers.",
            "Restore…",
            Action::Launch(&[TOOLS, "--restore"]),
        ),
        row_button(
            "Bookmarks and passwords",
            "Export them from your old browser (an HTML file for bookmarks, a CSV file for passwords), then import them in Firefox.",
            "Open Firefox",
            Action::Launch(FIREFOX),
        ),
        row_button(
            "Email",
            "Thunderbird sets up your account with just your address and password.",
            "Open Thunderbird",
            Action::Launch(THUNDERBIRD),
        ),
        row_button(
            "An app cannot see a folder or a USB drive?",
            "Apps run apart from the system and only see what they need. Flatseal gives them access to more.",
            "Open Flatseal",
            Action::Launch(FLATSEAL),
        ),
    ]
}

pub(super) fn pages(mode: &Mode, origin: Option<Origin>, experienced: bool) -> Vec<Page> {
    match mode {
        Mode::FirstSteps => {
            let welcome = Page::new(
                "Welcome to Celenn OS",
                Some(
                    "Your system is installed and ready. Where do you come from? The tour adapts to you.",
                ),
                Body::Origin,
            );
            let ready = Page::new(
                "You are ready",
                Some("What is done, and what is left for when you want."),
                Body::Ready,
            );
            // The short tour, for those who know Linux well (the choice
            // shows under the Linux card)
            if experienced && origin == Some(Origin::Linux) {
                return vec![welcome, essentials(), ready];
            }
            // No banner nor description: the wallpapers and pictures need
            // the room (each choice applies at once, which says it all)
            let look = Page::new("Make it yours", None, Body::Look);
            // No banner: the six rows need the room
            let apps = Page {
                note: Some("Thousands of other apps are in COSMIC Store."),
                ..Page::new(
                    "Your apps",
                    Some(
                        "Pick what concerns you. In each list, the apps most used on Linux are already ticked, all free. Nothing installs without your OK.",
                    ),
                    Body::Profiles,
                )
            };
            // Nobody said where they come from: the shortcuts to try
            let bearings = match origin {
                Some(origin) => bearings(origin),
                None => Page {
                    icon: Some("celenn-tour-try"),
                    note: Some(SHORTCUTS_NOTE),
                    ..Page::new(
                        "Try it, it is safe",
                        Some("Each button does the real thing."),
                        Body::Try,
                    )
                },
            };
            vec![welcome, look, apps, bearings, ready]
        }
        Mode::Live => vec![
            Page::new(
                "Welcome to Celenn OS",
                Some(
                    "Celenn OS is Fedora Atomic with the COSMIC desktop, made in the spirit of Bluefin: a desktop that stays out of your way and keeps itself up to date.",
                ),
                Body::Welcome,
            ),
            Page::new(
                "Is this computer ready?",
                Some(
                    "Celenn OS runs from the USB drive, without touching your disk. Here is what it found.",
                ),
                Body::Readiness,
            ),
            Page::new(
                "What you get",
                None,
                Body::Features(vec![
                    (
                        "Automatic updates",
                        "The whole system updates in the background and applies at the next restart. If something goes wrong, boot the previous version.",
                    ),
                    (
                        "Apps from Flathub and COSMIC",
                        "Install apps with COSMIC Store, from Flathub and the COSMIC repository. They run sandboxed and update on their own.",
                    ),
                    (
                        "Ready for the terminal",
                        "Homebrew for command line tools, distrobox for other distributions, and ujust for common tasks.",
                    ),
                ]),
            ),
            Page::new(
                "Ready to install?",
                Some(
                    "The installer copies Celenn OS to your disk. After that, it updates itself from the Celenn OS image registry.",
                ),
                Body::Install,
            ),
        ],
        Mode::Open(_) | Mode::Shortcuts | Mode::Phone | Mode::Belongings => Vec::new(),
    }
}

/// Texts that are not in a page (buttons, states, the other windows)
pub(super) const TEXTS: &[&str] = &[
    "Back",
    "Next",
    "Close",
    "Later",
    "Step {} of {}",
    "First steps",
    "Open",
    "Get started",
    "Install Celenn OS",
    "Keep trying",
    "Accessibility",
    ACCESSIBILITY,
    "Settings",
    // First page
    "Linux and you",
    "I am new to it",
    "I know it well",
    "I know it well: a shorter tour, in three steps.",
    "Installing your apps: {} of {}",
    "They show up in the app library as they arrive.",
    // Look
    "Theme",
    "Dark",
    "Light",
    "Wallpaper",
    "Your picture",
    "Layout",
    "Top bar and dock: like the Mac. One bar at the bottom: like Windows.",
    "Top bar and dock",
    "Bar at the bottom",
    "This did not work.",
    // Send without a cable (--phone)
    "Send without a cable",
    "With LocalSend, already on this computer. The files go straight through your Wi-Fi, encrypted, not through the internet.",
    "On the phone, scan this code and install LocalSend (Play Store or App Store).",
    "Open LocalSend here and on the phone, on the same Wi-Fi.",
    "On the phone, pick the photos, then this computer. They arrive in Downloads. It works the other way too.",
    "Open LocalSend",
    // Your apps (the categories: apps.rs)
    "Install and continue",
    "Installing: {} of {}",
    "Ticked and grey: already installed.",
    "Homebrew is still being set up: the code editors and terminal tools can be chosen in a few minutes.",
    "Back to the list",
    // What the computer needs (last page)
    "No internet connection",
    "Apps and updates need it.",
    "Network settings",
    "NVIDIA graphics card",
    "The NVIDIA edition of Celenn OS has its driver: about 4 GB to download, then it starts at the next restart.",
    "Switch",
    "Downloading the NVIDIA edition… This can take a while.",
    "The NVIDIA edition starts at the next restart.",
    "At the restart, a blue screen asks to enroll a key: choose Enroll MOK, Continue, Yes, then type the password celennos (the keyboard is QWERTY there).",
    "Firmware updates are available",
    "Open Firmware",
    "Take care of the battery",
    "Stop charging at 80 % when the laptop stays plugged in: the battery lasts longer.",
    "Screen scale",
    "Text and windows too small or too big? Change the scale of the screen.",
    "Display settings",
    // Try
    "Try",
    "Turn on",
    "Turn off",
    // Ready
    "Your look is set",
    "Your apps are installed",
    "Everything updates by itself",
    "In the background, applied at the next restart. The previous version stays one click away.",
    "Bring your things",
    "Files, photos from a phone, bookmarks and email from before",
    "Show me how",
    "Protect your files",
    "Plug in a USB drive and turn on automatic backups: Déjà Dup keeps a copy of your home folder.",
    "Set up backups",
    "Documentation",
    "Updates, software, commands and tips",
    "This tour and the Keyboard shortcuts app stay in the app library.",
    // Files that do not install here
    "“{}” does not install here",
    "This is a Windows program. On Celenn OS, apps come from COSMIC Store: most well-known apps are there, or a good alternative.",
    "This is a package for Debian and Ubuntu. On Celenn OS, apps come from COSMIC Store: most well-known apps are there, or a good alternative.",
    "This is a package for Fedora. Celenn OS's system is a read-only image: apps come from COSMIC Store instead. Most well-known apps are there, or a good alternative.",
    "Search for “{}” in COSMIC Store",
    "Open COSMIC Store",
    "A Windows program with no equivalent? Bottles, in COSMIC Store, runs some of them.",
    "Open with Bottles",
    "See the equivalents of Windows programs",
    "For advanced users: distrobox installs it in a container of another distribution.",
    // Keyboard shortcuts
    "Keyboard shortcuts",
    "Super is the key with the Windows logo on most keyboards.",
    "Change them in Settings",
    "Escape closes this window.",
];
