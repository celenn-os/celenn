// Three pages of cosmic-blue Tools (tools.rs) that turn `ujust` commands
// into buttons. Each button runs the matching command (`ujust <recipe>`,
// without a terminal: the recipes then skip their questions), so the
// terminal and the window do exactly the same thing:
//
//   Computer       the battery charge limit (toggle-battery-limit), the
//                  NVIDIA edition for NVIDIA graphics cards (the Updates
//                  helper, as the first steps tour) and, with Secure Boot,
//                  its driver key (Tools helper, as `ujust
//                  enroll-secure-boot-key`), restarting into the firmware
//                  settings (reboot-to-bios)
//   Extra software Microsoft fonts, Steam and Heroic, a password manager,
//                  virtual machines,
//                  the FlatPark app source, developer tools (ticked in a
//                  list, installed as the first steps tour does:
//                  devtools.rs)
//   Appearance     dark or light, the cosmic-blue look back, every COSMIC
//                  setting back to the defaults, settings saved to a file
//
// Changes that cannot be undone in one click (restart, reset, the NVIDIA
// edition, a license) ask first, in the row itself. A change that does not
// work shows a card at the top of its page (failure.rs).

use crate::common::{label, spawn, state_icon};
use crate::devtools::{self, DEV_TOOLS, DevTool, ToolKind};
use crate::failure::{self, Card, CardAction, CardMessage, Outcome};
use crate::i18n::tr;
use cosmic::app::Task;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, settings, text, toaster::Toasts};
use cosmic::{Apply, Element, cosmic_theme, theme};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const UPDATES_HELPER: &str = "/usr/libexec/cosmic-blue-updates-helper";
const TOOLS_HELPER: &str = "/usr/libexec/cosmic-blue-tools-helper";
const IMAGE_INFO: &str = "/usr/share/ublue-os/image-info.json";
const FONTS_LICENSE: &str = "https://corefonts.sourceforge.net/eula.htm";
const VMS: &str = "org.virt_manager.virt-manager";

pub const TEXTS: &[&str] = &[
    // Pages
    "Computer",
    "Battery, graphics card and firmware settings.",
    "Extra software",
    "What cosmic-blue does not install by itself, ready in one click.",
    "Appearance",
    "The look of the desktop and your COSMIC settings.",
    // Common
    "Install",
    "Remove",
    "Cancel",
    "Working…",
    // Computer
    "Battery charge limit",
    "Charging stops at 80 %: the battery lasts longer when the laptop stays plugged in.",
    "NVIDIA graphics card",
    "Use the NVIDIA edition of cosmic-blue, with NVIDIA's own driver, for games and graphics apps.",
    "You use the NVIDIA edition, with NVIDIA's own driver.",
    "Use it",
    "The NVIDIA edition is downloaded now and starts at the next restart. With Secure Boot on, a blue screen asks for the password cosmicblue once.",
    "Switch",
    "The NVIDIA edition starts at the next restart.",
    "Secure Boot does not let NVIDIA's driver load yet. Allow it once: at the next restart, a blue screen asks for the password cosmicblue.",
    "Allow",
    "At the next restart, a blue screen asks for the password cosmicblue: choose Enroll MOK, Continue, Yes, type cosmicblue, then Reboot. The keyboard may be in QWERTY.",
    "Firmware settings (BIOS/UEFI)",
    "Restart into the computer's own settings: boot order, virtualization, Secure Boot…",
    "Restart",
    "The computer restarts now. Save your work first.",
    "Restart now",
    // Extra software
    "Microsoft fonts",
    "Arial, Times New Roman, Verdana… for documents made on Windows.",
    "Installed. Restart open apps to see them.",
    "Microsoft's license applies to these fonts.",
    "Read it",
    "Accept and install",
    "Games",
    "Steam, Heroic, Lutris, Discord…",
    "Stores",
    "Windows games",
    "And also",
    "Your game apps are ready.",
    "Passwords",
    "Bitwarden (synced with your phone), Secrets, KeePassXC.",
    "Synced",
    "On this computer",
    "Your password manager is ready.",
    "Virtual machines",
    "virt-manager, to run Windows or another Linux in a window.",
    "FlatPark",
    "More apps in COSMIC Store, from outside Flathub. Less checked.",
    "Developer tools",
    "Docker, VS Code, Neovim…",
    "Choose…",
    "Hide",
    "Containers",
    "Code editors",
    "In the terminal",
    "Apply",
    "Tick the ones you want, untick the ones to remove.",
    "Tick the ones you want, untick the ones to remove. Your games and saves stay.",
    "Homebrew is not set up yet: only Podman Desktop can be installed now.",
    "The developer tools are ready.",
    // Appearance
    "Dark theme",
    "Light text on dark backgrounds. Off: the light theme.",
    "Light theme",
    "Undo",
    "Layout",
    "Top bar and dock: like the Mac. One bar at the bottom: like Windows.",
    "Top bar and dock",
    "Bar at the bottom",
    "Dock always visible",
    "Off: it hides when a window covers it.",
    "The dock hides when a window covers it.",
    "This did not work.",
    "More in Settings",
    "Accent color, wallpaper, panel, dock, fonts…",
    "Open",
    "The cosmic-blue look",
    "Back to the cosmic-blue colors, dark theme, icons, fonts and wallpaper. Panel, dock, keyboard and shortcuts stay.",
    "Restore",
    "All COSMIC settings",
    "Back to the cosmic-blue defaults, for every setting. A copy of the current ones is kept.",
    "Reset",
    "Every COSMIC setting goes back to the defaults. A copy is kept in your settings folder.",
    "Log out and back in to finish.",
    "Save my settings",
    "A text file in your home folder, to keep or to compare.",
    "Save",
    "Saved in your home folder.",
];

/// The game apps, from Flathub, by group. Steam and Heroic are ticked
/// the first time the list opens
const GAMES: &[(&str, &[DevTool])] = &[
    (
        "Stores",
        &[
            DevTool {
                name: "Steam",
                kind: ToolKind::Flatpak("com.valvesoftware.Steam"),
            },
            DevTool {
                name: "Heroic",
                kind: ToolKind::Flatpak("com.heroicgameslauncher.hgl"),
            },
            DevTool {
                name: "Lutris",
                kind: ToolKind::Flatpak("net.lutris.Lutris"),
            },
        ],
    ),
    (
        "Windows games",
        &[
            DevTool {
                name: "Bottles",
                kind: ToolKind::Flatpak("com.usebottles.bottles"),
            },
            DevTool {
                name: "ProtonPlus",
                kind: ToolKind::Flatpak("com.vysp3r.ProtonPlus"),
            },
        ],
    ),
    (
        "And also",
        &[
            DevTool {
                name: "RetroArch",
                kind: ToolKind::Flatpak("org.libretro.RetroArch"),
            },
            DevTool {
                name: "Prism Launcher",
                kind: ToolKind::Flatpak("org.prismlauncher.PrismLauncher"),
            },
            DevTool {
                name: "Discord",
                kind: ToolKind::Flatpak("com.discordapp.Discord"),
            },
        ],
    ),
];

/// The password managers, from Flathub: the ones of the first steps tour's
/// "Your passwords" page (main.rs), and KeePassXC
const VAULT_APPS: &[(&str, &[DevTool])] = &[
    (
        "Synced",
        &[DevTool {
            name: "Bitwarden",
            kind: ToolKind::Flatpak(devtools::BITWARDEN),
        }],
    ),
    (
        "On this computer",
        &[
            DevTool {
                name: "Secrets",
                kind: ToolKind::Flatpak("org.gnome.World.Secrets"),
            },
            DevTool {
                name: "KeePassXC",
                kind: ToolKind::Flatpak("org.keepassxc.KeePassXC"),
            },
        ],
    ),
];

/// The lists of apps picked with checkboxes
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum List {
    Dev = 0,
    Games = 1,
    Vault = 2,
}

impl List {
    fn groups(self) -> &'static [(&'static str, &'static [DevTool])] {
        match self {
            List::Dev => DEV_TOOLS,
            List::Games => GAMES,
            List::Vault => VAULT_APPS,
        }
    }

    fn tools(self) -> impl Iterator<Item = &'static DevTool> {
        self.groups().iter().flat_map(|(_, tools)| tools.iter())
    }

    fn title(self) -> &'static str {
        match self {
            List::Dev => "Developer tools",
            List::Games => "Games",
            List::Vault => "Passwords",
        }
    }

    fn done(self) -> &'static str {
        match self {
            List::Dev => "The developer tools are ready.",
            List::Games => "Your game apps are ready.",
            List::Vault => "Your password manager is ready.",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Computer,
    Software,
    Appearance,
}

/// What the rows show: read in the background, again after each change
#[derive(Clone, Debug, Default)]
pub struct State {
    /// None: no laptop battery whose charge can be limited
    battery_limit: Option<bool>,
    /// None: no NVIDIA graphics card. Some(true): the NVIDIA edition runs
    nvidia: Option<bool>,
    /// The NVIDIA edition with Secure Boot on: its driver key, when it is
    /// not enrolled yet
    driver_key: Option<DriverKey>,
    efi: bool,
    fonts: bool,
    vms: bool,
    flatpark: bool,
    dark: bool,
    bottom_bar: bool,
    dock_visible: bool,
    /// The apps of each list installed (numbers of List::tools())
    installed: [BTreeSet<usize>; 3],
    brew: bool,
}

/// The key that signs NVIDIA's driver, when Secure Boot does not know it yet
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DriverKey {
    Missing,
    /// Queued: the firmware asks for it at the next restart
    Queued,
}

/// Every change a button makes
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Job {
    BatteryLimit(bool),
    Nvidia,
    DriverKey,
    Firmware,
    Fonts(bool),
    Vms(bool),
    FlatPark(bool),
    Dark(bool),
    ResetTheme,
    ResetConfig,
    Export,
}

impl Job {
    /// The program that does it, and its arguments
    fn command(self) -> (&'static str, Vec<&'static str>) {
        let ujust = |args: &[&'static str]| ("ujust", args.to_vec());
        let on_off = |on: bool| if on { "on" } else { "off" };
        match self {
            Job::BatteryLimit(on) => ujust(&["toggle-battery-limit", on_off(on)]),
            Job::Nvidia => ("pkexec", vec![UPDATES_HELPER, "edition", "nvidia"]),
            Job::DriverKey => ("pkexec", vec![TOOLS_HELPER, "driver-key"]),
            Job::Firmware => ("systemctl", vec!["reboot", "--firmware-setup"]),
            Job::Fonts(true) => ujust(&["install-microsoft-fonts", "yes"]),
            Job::Fonts(false) => ujust(&["remove-microsoft-fonts"]),
            Job::Vms(true) => ujust(&["install-vms"]),
            Job::Vms(false) => ujust(&["remove-vms"]),
            Job::FlatPark(on) => ujust(&["toggle-flatpark", on_off(on)]),
            Job::Dark(true) => ujust(&["toggle-dark-mode", "dark"]),
            Job::Dark(false) => ujust(&["toggle-dark-mode", "light"]),
            Job::ResetTheme => ujust(&["reset-cosmic-theme", "yes"]),
            Job::ResetConfig => ujust(&["reset-cosmic-config", "yes"]),
            Job::Export => ujust(&["export-cosmic-config"]),
        }
    }

    /// The title of its row, for the card when it does not work
    fn title(self) -> &'static str {
        match self {
            Job::BatteryLimit(_) => "Battery charge limit",
            Job::Nvidia | Job::DriverKey => "NVIDIA graphics card",
            Job::Firmware => "Firmware settings (BIOS/UEFI)",
            Job::Fonts(_) => "Microsoft fonts",
            Job::Vms(_) => "Virtual machines",
            Job::FlatPark(_) => "FlatPark",
            Job::Dark(_) => "Dark theme",
            Job::ResetTheme => "The cosmic-blue look",
            Job::ResetConfig => "All COSMIC settings",
            Job::Export => "Save my settings",
        }
    }

    fn kind(self) -> Kind {
        match self {
            Job::BatteryLimit(_) | Job::Nvidia | Job::DriverKey | Job::Firmware => Kind::Computer,
            Job::Fonts(_) | Job::Vms(_) | Job::FlatPark(_) => Kind::Software,
            Job::Dark(_) | Job::ResetTheme | Job::ResetConfig | Job::Export => Kind::Appearance,
        }
    }

    /// Asked first, in the row: what happens
    fn question(self) -> Option<&'static str> {
        match self {
            Job::Nvidia => Some(
                "The NVIDIA edition is downloaded now and starts at the next restart. With Secure Boot on, a blue screen asks for the password cosmicblue once.",
            ),
            Job::Firmware => Some("The computer restarts now. Save your work first."),
            Job::Fonts(true) => Some("Microsoft's license applies to these fonts."),
            Job::ResetConfig => Some(
                "Every COSMIC setting goes back to the defaults. A copy is kept in your settings folder.",
            ),
            _ => None,
        }
    }

    /// The button that confirms the question
    fn confirm(self) -> &'static str {
        match self {
            Job::Nvidia => "Switch",
            Job::Firmware => "Restart now",
            Job::Fonts(true) => "Accept and install",
            _ => "Reset",
        }
    }

    /// Said when it worked (the rows show the rest)
    fn done(self) -> Option<&'static str> {
        match self {
            Job::Nvidia => Some("The NVIDIA edition starts at the next restart."),
            Job::Fonts(true) => Some("Installed. Restart open apps to see them."),
            Job::ResetConfig => Some("Log out and back in to finish."),
            Job::Export => Some("Saved in your home folder."),
            Job::ResetTheme => Some("Log out and back in to finish."),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    State(State),
    /// A button: asks first when the change needs it
    Ask(Job),
    Cancel,
    Run(Job),
    Done(Job, Outcome),
    OpenSettings,
    /// One bar at the bottom (true), or the top bar and the dock
    Layout(bool),
    DockVisible(bool),
    FontsLicense,
    /// Show or hide a list of apps
    ListOpen(List),
    Tick(List, usize, bool),
    ListApply(List),
    ListDone(List, Outcome),
    Card(CardMessage),
    CloseToast(widget::ToastId),
    /// A toast's Undo: closes it, then takes the change back
    Undo(widget::ToastId, Box<Message>),
}

/// What a card's "Try again" runs again
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Retry {
    Job(Job),
    List(List),
}

pub struct Page {
    state: Option<State>,
    busy: Option<Job>,
    asking: Option<Job>,
    /// The lists shown, the apps ticked in them, the list being applied
    open: [bool; 3],
    chosen: [BTreeSet<usize>; 3],
    applying: Option<List>,
    /// What did not work, until closed or done again
    card: Option<Card<Retry>>,
    toasts: Toasts<Message>,
}

fn output(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
        .unwrap_or_default()
}

fn home() -> PathBuf {
    std::env::var_os("HOME").map_or_else(|| PathBuf::from("."), PathBuf::from)
}

/// An NVIDIA graphics card: Some(whether the NVIDIA edition runs)
fn nvidia() -> Option<bool> {
    let read = |dir: &Path, name: &str| {
        std::fs::read_to_string(dir.join(name))
            .map(|value| value.trim().to_owned())
            .unwrap_or_default()
    };
    let found = std::fs::read_dir("/sys/bus/pci/devices")
        .ok()?
        .flatten()
        .any(|entry| {
            let dir = entry.path();
            // Class 0x03xxxx: display controller (VGA, 3D)
            read(&dir, "vendor") == "0x10de" && read(&dir, "class").starts_with("0x03")
        });
    let image = std::fs::read_to_string(IMAGE_INFO)
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .and_then(|info| info.get("image-name")?.as_str().map(str::to_owned))
        .unwrap_or_default();
    found.then(|| image.ends_with("-nvidia"))
}

/// With Secure Boot on, whether the key that signs NVIDIA's driver is
/// enrolled (mokutil reads the firmware's lists, no root needed)
fn driver_key() -> Option<DriverKey> {
    const KEY: &str = "/etc/pki/akmods/certs/akmods-ublue.der";
    if !Path::new(KEY).exists()
        || !output("mokutil", &["--sb-state"]).contains("SecureBoot enabled")
    {
        return None;
    }
    if output("mokutil", &["--test-key", KEY]).contains("already enrolled") {
        return None;
    }
    // Queued for the next restart: listed among the keys to enroll
    let queued = Command::new("mokutil")
        .args(["--list-new"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success());
    Some(if queued {
        DriverKey::Queued
    } else {
        DriverKey::Missing
    })
}

/// The COSMIC dark theme setting: the person's, else the system default
fn dark() -> bool {
    let key = "cosmic/com.system76.CosmicTheme.Mode/v1/is_dark";
    let config =
        std::env::var_os("XDG_CONFIG_HOME").map_or_else(|| home().join(".config"), PathBuf::from);
    std::fs::read_to_string(config.join(key))
        .or_else(|_| std::fs::read_to_string(Path::new("/usr/share").join(key)))
        .map_or(true, |value| value.trim() == "true")
}

pub fn read_state() -> State {
    // `ujust toggle-battery-limit status` says whether the limit is on, or
    // why there is none
    let battery = output("ujust", &["toggle-battery-limit", "status"]);
    let battery_limit = if battery.contains("Battery limit on") {
        Some(true)
    } else if battery.contains("Battery limit off") {
        Some(false)
    } else {
        None
    };
    let apps = output(
        "flatpak",
        &["list", "--system", "--app", "--columns=application"],
    );
    let has = |id: &str| apps.lines().any(|line| line.trim() == id);
    let data = std::env::var_os("XDG_DATA_HOME")
        .map_or_else(|| home().join(".local/share"), PathBuf::from);
    let nvidia = nvidia();
    State {
        battery_limit,
        nvidia,
        driver_key: if nvidia == Some(true) {
            driver_key()
        } else {
            None
        },
        efi: Path::new("/sys/firmware/efi").exists(),
        fonts: data.join("fonts/microsoft-core-fonts").is_dir(),
        vms: has(VMS),
        flatpark: output("flatpak", &["remotes", "--user", "--columns=name"])
            .lines()
            .any(|line| line.trim() == "flatpark"),
        dark: dark(),
        bottom_bar: crate::layout::bottom_bar(),
        dock_visible: crate::layout::dock_always_visible(),
        installed: [List::Dev, List::Games, List::Vault].map(|list| {
            let mut installed = match list {
                List::Dev => devtools::brew_installed(),
                List::Games | List::Vault => BTreeSet::new(),
            };
            installed.extend(
                list.tools()
                    .enumerate()
                    .filter_map(|(i, tool)| match tool.kind {
                        ToolKind::Flatpak(app) if has(app) => Some(i),
                        _ => None,
                    }),
            );
            installed
        }),
        brew: Path::new(devtools::BREW).exists(),
    }
}

fn refresh() -> Task<Message> {
    Task::perform(
        async {
            tokio::task::spawn_blocking(read_state)
                .await
                .unwrap_or_default()
        },
        |state| cosmic::Action::App(Message::State(state)),
    )
}

impl Page {
    pub fn new() -> (Self, Task<Message>) {
        let page = Page {
            state: None,
            busy: None,
            asking: None,
            open: [false; 3],
            chosen: Default::default(),
            applying: None,
            card: None,
            toasts: Toasts::new(Message::CloseToast),
        };
        (page, refresh())
    }

    /// A change of the desktop's look, said with a button to take it back
    fn undo_toast(&mut self, message: &'static str, undo: Message) -> Task<Message> {
        self.toasts
            .push(
                widget::toaster::Toast::new(tr(message)).action(tr("Undo").to_owned(), move |id| {
                    Message::Undo(id, Box::new(undo.clone()))
                }),
            )
            .map(cosmic::Action::App)
    }

    fn toast(&mut self, message: &'static str) -> Task<Message> {
        self.toasts
            .push(widget::toaster::Toast::new(tr(message)))
            .map(cosmic::Action::App)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::State(state) => self.state = Some(state),
            Message::Ask(job) => {
                if job.question().is_some() {
                    self.asking = Some(job);
                } else {
                    return self.update(Message::Run(job));
                }
            }
            Message::Cancel => self.asking = None,
            Message::Run(job) => {
                self.asking = None;
                self.card = None;
                self.busy = Some(job);
                // The switch and the theme show the change at once
                if let (Job::Dark(on), Some(state)) = (job, self.state.as_mut()) {
                    state.dark = on;
                }
                let (program, args) = job.command();
                let args = args.into_iter().map(str::to_owned).collect();
                return Task::perform(failure::run(program, args, None), move |result| {
                    cosmic::Action::App(Message::Done(job, result))
                });
            }
            Message::Done(job, result) => {
                self.busy = None;
                let toast = match result {
                    None => Task::none(),
                    Some(Err(failure)) => {
                        self.card = Some(Card::new(tr(job.title()), failure, Retry::Job(job)));
                        Task::none()
                    }
                    Some(Ok(())) => match (job, job.done()) {
                        (Job::Dark(on), _) => self.undo_toast(
                            if on { "Dark theme" } else { "Light theme" },
                            Message::Run(Job::Dark(!on)),
                        ),
                        (_, Some(text)) => self.toast(text),
                        (_, None) => Task::none(),
                    },
                };
                return Task::batch([toast, refresh()]);
            }
            Message::OpenSettings => spawn("cosmic-settings", &["appearance"]),
            Message::Layout(bottom) => {
                if !crate::layout::set_bottom_bar(bottom) {
                    return self.toast("This did not work.");
                }
                if let Some(state) = &mut self.state {
                    state.bottom_bar = bottom;
                }
                return self.undo_toast(
                    if bottom {
                        "Bar at the bottom"
                    } else {
                        "Top bar and dock"
                    },
                    Message::Layout(!bottom),
                );
            }
            Message::DockVisible(visible) => {
                if !crate::layout::set_dock_always_visible(visible) {
                    return self.toast("This did not work.");
                }
                if let Some(state) = &mut self.state {
                    state.dock_visible = visible;
                }
                return self.undo_toast(
                    if visible {
                        "Dock always visible"
                    } else {
                        "The dock hides when a window covers it."
                    },
                    Message::DockVisible(!visible),
                );
            }
            Message::Undo(id, message) => {
                self.toasts.remove(id);
                return self.update(*message);
            }
            Message::FontsLicense => spawn("xdg-open", &[FONTS_LICENSE]),
            Message::ListOpen(list) => {
                let n = list as usize;
                self.open[n] = !self.open[n];
                if let Some(state) = &self.state {
                    self.chosen[n] = state.installed[n].clone();
                    // Nothing installed yet: Steam and Heroic ticked
                    if list == List::Games && self.chosen[n].is_empty() {
                        self.chosen[n].extend([0, 1]);
                    }
                }
            }
            Message::Tick(list, i, on) => {
                let chosen = &mut self.chosen[list as usize];
                if on {
                    chosen.insert(i);
                } else {
                    chosen.remove(&i);
                }
            }
            Message::ListApply(list) => {
                let Some(state) = &self.state else {
                    return Task::none();
                };
                let n = list as usize;
                self.applying = Some(list);
                self.card = None;
                return Task::perform(
                    apply_list(list, state.installed[n].clone(), self.chosen[n].clone()),
                    move |result| cosmic::Action::App(Message::ListDone(list, result)),
                );
            }
            Message::ListDone(list, result) => {
                self.applying = None;
                let toast = match result {
                    None => Task::none(),
                    Some(Err(failure)) => {
                        self.card = Some(Card::new(tr(list.title()), failure, Retry::List(list)));
                        Task::none()
                    }
                    Some(Ok(())) => {
                        let n = list as usize;
                        self.open[n] = false;
                        // Bitwarden just installed: its Firefox extension
                        // too (devtools.rs)
                        if list == List::Vault
                            && let Some(state) = &self.state
                        {
                            let bitwarden = list.tools().position(|tool| {
                                tool.kind == ToolKind::Flatpak(devtools::BITWARDEN)
                            });
                            if bitwarden.is_some_and(|i| {
                                self.chosen[n].contains(&i) && !state.installed[n].contains(&i)
                            }) {
                                let command = devtools::BITWARDEN_EXTENSION;
                                spawn(command[0], &command[1..]);
                            }
                        }
                        self.toast(list.done())
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
                    CardAction::Retry(Retry::Job(job)) => return self.update(Message::Run(job)),
                    CardAction::Retry(Retry::List(list)) => {
                        return self.update(Message::ListApply(list));
                    }
                }
            }
            Message::CloseToast(id) => self.toasts.remove(id),
        }
        Task::none()
    }

    /// The button of a row: a spinner while its change runs, else the
    /// button (disabled while another change runs)
    fn button<'a>(&self, job: Job, name: &'static str, suggested: bool) -> Element<'a, Message> {
        if self.busy == Some(job) {
            return widget::row::with_capacity(2)
                .spacing(theme::spacing().space_xxs)
                .align_y(Alignment::Center)
                .push(widget::progress_bar::indeterminate_circular().size(20.0))
                .push(text::body(tr("Working…")))
                .into();
        }
        let message = self.busy.is_none().then_some(Message::Ask(job));
        if suggested {
            widget::button::suggested(tr(name))
                .on_press_maybe(message)
                .into()
        } else {
            widget::button::standard(tr(name))
                .on_press_maybe(message)
                .into()
        }
    }

    /// Install or Remove, after what is there
    fn install<'a>(&self, installed: bool, job: fn(bool) -> Job) -> Element<'a, Message> {
        if installed {
            self.button(job(false), "Remove", false)
        } else {
            self.button(job(true), "Install", false)
        }
    }

    fn toggler<'a>(&self, on: bool, job: fn(bool) -> Job) -> Element<'a, Message> {
        let toggler = widget::toggler(on);
        if self.busy.is_none() {
            toggler.on_toggle(move |on| Message::Run(job(on))).into()
        } else {
            toggler.into()
        }
    }

    /// A row: its symbol, its text, its button. While its change waits for
    /// an answer, the question and Cancel / confirm under it
    fn row<'a>(
        &self,
        icon: &'static str,
        title: &'static str,
        caption: &'static str,
        control: Element<'a, Message>,
        job: Option<Job>,
    ) -> Element<'a, Message> {
        let space_s = theme::spacing().space_s;
        let main = settings::item_row(vec![
            state_icon(icon, false),
            label(tr(title), tr(caption).to_owned()),
            control,
        ])
        .spacing(space_s);
        let Some(job) = job.filter(|job| self.asking == Some(*job)) else {
            return main.into();
        };
        let mut actions = widget::row::with_capacity(4)
            .spacing(theme::spacing().space_xxs)
            .align_y(Alignment::Center)
            .push(text::caption(tr(job.question().unwrap_or_default())).width(Length::Fill));
        if job == Job::Fonts(true) {
            actions = actions.push(
                widget::button::link(tr("Read it"))
                    .trailing_icon(true)
                    .on_press(Message::FontsLicense),
            );
        }
        actions = actions
            .push(widget::button::standard(tr("Cancel")).on_press(Message::Cancel))
            .push(if job == Job::ResetConfig {
                // Settings lost (a copy kept): the warning style
                widget::button::destructive(tr(job.confirm())).on_press(Message::Run(job))
            } else {
                widget::button::suggested(tr(job.confirm())).on_press(Message::Run(job))
            });
        widget::column::with_capacity(2)
            .push(main)
            .push(widget::container(actions).padding([0, space_s, space_s, space_s + 36]))
            .into()
    }

    pub fn view(&self, kind: Kind) -> Element<'_, Message> {
        let cosmic_theme::Spacing { space_s, .. } = theme::spacing();
        let (title, intro, icon) = match kind {
            Kind::Computer => (
                "Computer",
                "Battery, graphics card and firmware settings.",
                // The tour's screen, in the style of the other pages' icons
                "cosmic-blue-tour-computer",
            ),
            Kind::Software => (
                "Extra software",
                "What cosmic-blue does not install by itself, ready in one click.",
                "cosmic-blue-tour-store",
            ),
            Kind::Appearance => (
                "Appearance",
                "The look of the desktop and your COSMIC settings.",
                "cosmic-blue-tools-appearance",
            ),
        };
        let mut content = widget::column::with_capacity(3)
            .spacing(space_s)
            .align_x(Alignment::Center);
        let card_kind = self.card.as_ref().map(|card| match card.job {
            Retry::Job(job) => job.kind(),
            Retry::List(_) => Kind::Software,
        });
        if let Some(card) = self.card.as_ref().filter(|_| card_kind == Some(kind)) {
            content = content.push(card.view(Message::Card));
        }
        if let Some(state) = &self.state {
            content = content.push(match kind {
                Kind::Computer => self.computer(state),
                Kind::Software => self.software(state),
                Kind::Appearance => self.appearance(state),
            });
        }
        let page = crate::common::header(icon, tr(title), tr(intro), content.into());
        widget::toaster(&self.toasts, page)
    }

    fn computer(&self, state: &State) -> Element<'_, Message> {
        let mut section = settings::section();
        if let Some(on) = state.battery_limit {
            section = section.add(self.row(
                "battery-good-symbolic",
                "Battery charge limit",
                "Charging stops at 80 %: the battery lasts longer when the laptop stays plugged in.",
                self.toggler(on, Job::BatteryLimit),
                None,
            ));
        }
        match state.nvidia {
            Some(true) => {
                section = section.add(match state.driver_key {
                    None => self.row(
                        "video-display-symbolic",
                        "NVIDIA graphics card",
                        "You use the NVIDIA edition, with NVIDIA's own driver.",
                        widget::space::horizontal().width(0).into(),
                        None,
                    ),
                    Some(DriverKey::Missing) => self.row(
                        "video-display-symbolic",
                        "NVIDIA graphics card",
                        "Secure Boot does not let NVIDIA's driver load yet. Allow it once: at the next restart, a blue screen asks for the password cosmicblue.",
                        self.button(Job::DriverKey, "Allow", true),
                        None,
                    ),
                    Some(DriverKey::Queued) => self.row(
                        "video-display-symbolic",
                        "NVIDIA graphics card",
                        "At the next restart, a blue screen asks for the password cosmicblue: choose Enroll MOK, Continue, Yes, type cosmicblue, then Reboot. The keyboard may be in QWERTY.",
                        widget::space::horizontal().width(0).into(),
                        None,
                    ),
                });
            }
            Some(false) => {
                section = section.add(self.row(
                    "video-display-symbolic",
                    "NVIDIA graphics card",
                    "Use the NVIDIA edition of cosmic-blue, with NVIDIA's own driver, for games and graphics apps.",
                    self.button(Job::Nvidia, "Use it", true),
                    Some(Job::Nvidia),
                ));
            }
            None => {}
        }
        if state.efi {
            section = section.add(self.row(
                "preferences-system-symbolic",
                "Firmware settings (BIOS/UEFI)",
                "Restart into the computer's own settings: boot order, virtualization, Secure Boot…",
                self.button(Job::Firmware, "Restart", false),
                Some(Job::Firmware),
            ));
        }
        section.into()
    }

    fn software<'a>(&'a self, state: &'a State) -> Element<'a, Message> {
        let mut section = settings::section().add(self.row(
            "font-x-generic-symbolic",
            "Microsoft fonts",
            "Arial, Times New Roman, Verdana… for documents made on Windows.",
            self.install(state.fonts, Job::Fonts),
            Some(Job::Fonts(true)),
        ));
        section = self.list(
            state,
            section,
            List::Games,
            "input-gaming-symbolic",
            "Games",
            "Steam, Heroic, Lutris, Discord…",
        );
        section = self.list(
            state,
            section,
            List::Vault,
            "dialog-password-symbolic",
            "Passwords",
            "Bitwarden (synced with your phone), Secrets, KeePassXC.",
        );
        section = section
            .add(self.row(
                "computer-symbolic",
                "Virtual machines",
                "virt-manager, to run Windows or another Linux in a window.",
                self.install(state.vms, Job::Vms),
                None,
            ))
            .add(self.row(
                "system-software-install-symbolic",
                "FlatPark",
                "More apps in COSMIC Store, from outside Flathub. Less checked.",
                self.toggler(state.flatpark, Job::FlatPark),
                None,
            ));
        section = self.list(
            state,
            section,
            List::Dev,
            "utilities-terminal-symbolic",
            "Developer tools",
            "Docker, VS Code, Neovim…",
        );
        section.into()
    }

    /// A row with Choose…, then when open the apps by group, each with its
    /// checkbox, then Apply
    fn list<'a>(
        &'a self,
        state: &'a State,
        section: settings::Section<'a, Message>,
        list: List,
        icon: &'static str,
        title: &'static str,
        caption: &'static str,
    ) -> settings::Section<'a, Message> {
        let n = list as usize;
        let mut section = section.add(
            self.row(
                icon,
                title,
                caption,
                widget::button::standard(tr(if self.open[n] { "Hide" } else { "Choose…" }))
                    .on_press_maybe(
                        (self.applying != Some(list)).then_some(Message::ListOpen(list)),
                    )
                    .into(),
                None,
            ),
        );
        if !self.open[n] {
            return section;
        }
        let cosmic_theme::Spacing {
            space_xxs, space_s, ..
        } = theme::spacing();
        let mut next = 0;
        for (heading, tools) in list.groups() {
            let mut cells: Vec<Element<'a, Message>> = Vec::new();
            for tool in tools.iter() {
                let i = next;
                next += 1;
                let available = state.brew || matches!(tool.kind, ToolKind::Flatpak(_));
                let on = self.chosen[n].contains(&i);
                let enabled = available && self.applying.is_none();
                let checkbox = widget::checkbox(on)
                    .on_toggle_maybe(enabled.then_some(move |on| Message::Tick(list, i, on)));
                // The tool's name ticks the box too
                let label = text::body(tool.name).apply(widget::mouse_area);
                let label = if enabled {
                    label.on_press(Message::Tick(list, i, !on))
                } else {
                    label
                };
                cells.push(
                    widget::row::with_capacity(2)
                        .spacing(space_xxs)
                        .align_y(Alignment::Center)
                        .push(checkbox)
                        .push(label)
                        .width(Length::Fixed(150.0))
                        .into(),
                );
            }
            // Three columns
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
            section = section.add(
                widget::row::with_capacity(2)
                    .spacing(space_s)
                    .push(text::heading(tr(heading)).width(Length::Fixed(130.0)))
                    .push(lines)
                    .padding([space_xxs, space_s, space_xxs, space_s + 36]),
            );
        }
        let changed = self.chosen[n] != state.installed[n];
        let apply: Element<'a, Message> = if self.applying == Some(list) {
            widget::row::with_capacity(2)
                .spacing(space_xxs)
                .align_y(Alignment::Center)
                .push(widget::progress_bar::indeterminate_circular().size(20.0))
                .push(text::body(tr("Working…")))
                .into()
        } else {
            widget::button::suggested(tr("Apply"))
                .on_press_maybe(
                    (changed && self.busy.is_none() && self.applying.is_none())
                        .then_some(Message::ListApply(list)),
                )
                .into()
        };
        // How the list works, said here rather than in the row's caption so
        // that the page stays short when the lists are closed
        let note = if list == List::Dev && !state.brew {
            tr("Homebrew is not set up yet: only Podman Desktop can be installed now.")
        } else if list == List::Games {
            tr("Tick the ones you want, untick the ones to remove. Your games and saves stay.")
        } else {
            tr("Tick the ones you want, untick the ones to remove.")
        };
        let note: Element<'a, Message> = text::caption(note).width(Length::Fill).into();
        section.add(
            widget::row::with_capacity(2)
                .spacing(space_s)
                .align_y(Alignment::Center)
                .push(note)
                .push(apply)
                .padding([space_xxs, space_s, space_s, space_s + 36]),
        )
    }

    fn appearance(&self, state: &State) -> Element<'_, Message> {
        let space_s = theme::spacing().space_s;
        let layout_buttons = [(false, "Top bar and dock"), (true, "Bar at the bottom")]
            .into_iter()
            .fold(
                widget::row::with_capacity(2).spacing(space_s),
                |row, (bottom, label)| {
                    let button = if state.bottom_bar == bottom {
                        widget::button::suggested(tr(label))
                    } else {
                        widget::button::standard(tr(label))
                    };
                    row.push(button.on_press(Message::Layout(bottom)))
                },
            );
        let mut look = settings::section()
            .add(self.row(
                "weather-clear-night-symbolic",
                "Dark theme",
                "Light text on dark backgrounds. Off: the light theme.",
                self.toggler(state.dark, Job::Dark),
                None,
            ))
            .add(self.row(
                "video-display-symbolic",
                "Layout",
                "Top bar and dock: like the Mac. One bar at the bottom: like Windows.",
                layout_buttons.into(),
                None,
            ));
        // The dock only exists with the top bar
        if !state.bottom_bar {
            look = look.add(
                self.row(
                    "view-app-grid-symbolic",
                    "Dock always visible",
                    "Off: it hides when a window covers it.",
                    widget::toggler(state.dock_visible)
                        .on_toggle(Message::DockVisible)
                        .into(),
                    None,
                ),
            );
        }
        let look = look.add(
            self.row(
                "preferences-desktop-appearance-symbolic",
                "More in Settings",
                "Accent color, wallpaper, panel, dock, fonts…",
                widget::button::standard(tr("Open"))
                    .on_press(Message::OpenSettings)
                    .into(),
                None,
            ),
        );
        let settings_section = settings::section()
            .add(self.row(
                "edit-undo-symbolic",
                "The cosmic-blue look",
                "Back to the cosmic-blue colors, dark theme, icons, fonts and wallpaper. Panel, dock, keyboard and shortcuts stay.",
                self.button(Job::ResetTheme, "Restore", false),
                None,
            ))
            .add(self.row(
                "document-revert-symbolic",
                "All COSMIC settings",
                "Back to the cosmic-blue defaults, for every setting. A copy of the current ones is kept.",
                self.button(Job::ResetConfig, "Reset", false),
                Some(Job::ResetConfig),
            ))
            .add(self.row(
                "document-save-symbolic",
                "Save my settings",
                "A text file in your home folder, to keep or to compare.",
                self.button(Job::Export, "Save", false),
                None,
            ));
        widget::column::with_capacity(2)
            .spacing(theme::spacing().space_s)
            .push(look)
            .push(settings_section)
            .into()
    }
}

/// Installs the apps ticked and removes the ones unticked, as the first
/// steps tour installs developer tools (devtools.rs): Homebrew for the user,
/// Flathub for everyone. None: the password prompt was dismissed; the
/// last failure otherwise
async fn apply_list(list: List, installed: BTreeSet<usize>, chosen: BTreeSet<usize>) -> Outcome {
    let (mut formulas, mut casks, mut apps) = (Vec::new(), Vec::new(), Vec::new());
    let (mut old_formulas, mut old_casks, mut old_apps) = (Vec::new(), Vec::new(), Vec::new());
    for (i, tool) in list.tools().enumerate() {
        let add = chosen.contains(&i) && !installed.contains(&i);
        let remove = !chosen.contains(&i) && installed.contains(&i);
        if !add && !remove {
            continue;
        }
        match tool.kind {
            ToolKind::Formula(names) if add => formulas.extend(names),
            ToolKind::Formula(names) => old_formulas.push(names[0]),
            ToolKind::Cask(name, tap) if add => casks.push(devtools::cask_name(name, tap)),
            ToolKind::Cask(name, tap) => old_casks.push(devtools::cask_name(name, tap)),
            ToolKind::Flatpak(app) if add => apps.push(app.to_owned()),
            ToolKind::Flatpak(app) => old_apps.push(app.to_owned()),
        }
    }
    let mut result = Ok(());
    if (!formulas.is_empty() || !casks.is_empty())
        && let Err(error) = devtools::brew_install(formulas, casks).await
    {
        result = Err(failure::failure(&error));
    }
    if (!old_formulas.is_empty() || !old_casks.is_empty())
        && let Err(error) = devtools::brew_remove(old_formulas, old_casks).await
    {
        result = Err(failure::failure(&error));
    }
    for (verb, apps) in [("install", apps), ("uninstall", old_apps)] {
        if apps.is_empty() {
            continue;
        }
        let mut args: Vec<String> = [verb, "--system", "--noninteractive", "-y"]
            .map(str::to_owned)
            .to_vec();
        if verb == "install" {
            args.push("flathub".to_owned());
        }
        args.extend(apps);
        match failure::run("flatpak", args, None).await {
            Some(Ok(())) => {}
            Some(Err(failure)) => result = Err(failure),
            None => return None,
        }
    }
    Some(result)
}
