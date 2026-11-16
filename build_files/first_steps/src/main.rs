// cosmic-blue tours, built with libcosmic so they look exactly like COSMIC's
// own first-run assistant (cosmic-initial-setup): same window, title, text,
// lists and buttons, frosted glass and accent color from the COSMIC theme.
//
//   cosmic-blue-tour             "First steps": the installed system, opened once
//                                at the first login of each new account
//                                (/usr/libexec/cosmic-blue-first-steps-autostart)
//                                and from the "First steps" launcher
//   cosmic-blue-tour --live      Welcome tour of the live ISO, with a button
//                                that starts the installer
//   cosmic-blue-tour --check     Image build test: every text has its
//                                translations (no window)
//
// Text is in English, with a French translation picked from the session
// language (see TRANSLATIONS). The layout follows cosmic-initial-setup's
// App::view (src/main.rs of its epoch-<version> tag).

use cosmic::app::{Core, Settings, Task};
use cosmic::iced::core::text::Wrapping;
use cosmic::iced::keyboard::{self, Key, key::Named};
use cosmic::iced::{Alignment, Event, Length, Limits, Subscription, event};
use cosmic::widget::{self, settings, text, toaster::Toasts};
use cosmic::{Application, ApplicationExt, Apply, Element, cosmic_theme, executor, theme};
use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

const SITE: &str = "https://cosmic-blue-os.github.io/cosmic-blue/";
const ISSUES: &str = "https://github.com/cosmic-blue-os/cosmic-blue/issues";
const LOGO: &str = "/usr/share/icons/hicolor/scalable/apps/cosmic-blue-logo.svg";
const INSTALLERS: [&str; 2] = [
    "/usr/share/applications/liveinst.desktop",
    "/usr/share/applications/anaconda.desktop",
];

/// English text -> French text
const FRENCH: &[(&str, &str)] = &[
    // Shared
    ("Back", "Précédent"),
    ("Next", "Suivant"),
    ("Close", "Fermer"),
    ("Copy", "Copier"),
    ("Copied", "Copié"),
    ("Welcome to cosmic-blue", "Bienvenue dans cosmic-blue"),
    // First steps
    ("First steps", "Premiers pas"),
    (
        "Your system is installed and ready. Here are a few things worth knowing to get started.",
        "Votre système est installé et prêt. Voici quelques points utiles pour bien démarrer.",
    ),
    ("Install apps", "Installer des applications"),
    (
        "Apps come from COSMIC Store, which offers Flathub and the COSMIC apps. They run apart from the system and update by themselves.",
        "Les applications viennent de COSMIC Store, qui propose Flathub et les applications COSMIC. Elles tournent à part du système et se mettent à jour seules.",
    ),
    ("Open COSMIC Store", "Ouvrir COSMIC Store"),
    (
        "Command line tools, installed in your home folder with Homebrew",
        "Outils en ligne de commande, installés dans votre dossier personnel avec Homebrew",
    ),
    (
        "Packages from other distributions, in a container that shares your home folder",
        "Paquets d’autres distributions, dans un conteneur qui partage votre dossier personnel",
    ),
    ("Updates and rollback", "Mises à jour et retour en arrière"),
    (
        "The system, your apps and Homebrew packages update by themselves in the background. A system update applies at the next restart, without interrupting you.",
        "Le système, vos applications et les paquets Homebrew se mettent à jour seuls en arrière-plan. Une mise à jour du système s’applique au redémarrage suivant, sans vous interrompre.",
    ),
    (
        "Update everything now, without waiting",
        "Tout mettre à jour maintenant, sans attendre",
    ),
    (
        "An update broke something? Go back to the previous version, then restart",
        "Une mise à jour a cassé quelque chose ? Revenez à la version précédente, puis redémarrez",
    ),
    (
        "Show the current and previous versions",
        "Afficher la version actuelle et la précédente",
    ),
    ("Handy commands", "Commandes utiles"),
    (
        "ujust runs the shortcuts that come with the system. Type these in a terminal.",
        "ujust lance les raccourcis fournis avec le système. Tapez-les dans un terminal.",
    ),
    ("List all the shortcuts", "Lister tous les raccourcis"),
    (
        "Developer tools: Docker, VS Code, Podman Desktop…",
        "Outils de développement : Docker, VS Code, Podman Desktop…",
    ),
    (
        "Switch between the dark and light themes",
        "Passer du thème sombre au thème clair",
    ),
    (
        "Bring the cosmic-blue look back: colors, icons, wallpaper",
        "Retrouver le style cosmic-blue : couleurs, icônes, fond d’écran",
    ),
    ("Open a terminal", "Ouvrir un terminal"),
    ("Tile your windows", "Disposez vos fenêtres en mosaïque"),
    (
        "COSMIC can arrange your windows side by side, without overlap, as you open them. It is off by default: turn it on with the tiling button in the panel, or with the keyboard.",
        "COSMIC peut placer vos fenêtres côte à côte, sans chevauchement, à mesure que vous les ouvrez. C’est désactivé par défaut : activez-le avec le bouton de mosaïque du panneau, ou au clavier.",
    ),
    ("Turn tiling on or off", "Activer ou désactiver la mosaïque"),
    (
        "Go to the window on the left, right, above or below",
        "Aller à la fenêtre de gauche, de droite, du dessus ou du dessous",
    ),
    (
        "Move the window to the left, right, up or down",
        "Déplacer la fenêtre à gauche, à droite, en haut ou en bas",
    ),
    (
        "Let a window float above the others",
        "Laisser une fenêtre flotter au-dessus des autres",
    ),
    (
        "COSMIC can be customized much further: Settings → Desktop.",
        "COSMIC se personnalise bien plus encore : Paramètres → Bureau.",
    ),
    ("Help and feedback", "Aide et retours"),
    (
        "Everything else is in the cosmic-blue guide. You can open this tour again from the app library: look for First steps.",
        "Tout le reste est dans le guide de cosmic-blue. Vous pouvez rouvrir cette visite depuis la bibliothèque d’applications : cherchez Premiers pas.",
    ),
    ("Documentation", "Documentation"),
    (
        "Updates, software, commands and tips",
        "Mises à jour, logiciels, commandes et astuces",
    ),
    ("Report a problem", "Signaler un problème"),
    (
        "Found a bug or something unclear? Tell us on GitHub",
        "Un bug ou quelque chose de pas clair ? Dites-le-nous sur GitHub",
    ),
    ("Open", "Ouvrir"),
    ("Get started", "C’est parti"),
    // Live welcome tour
    (
        "cosmic-blue is Fedora Atomic with the COSMIC desktop, made in the spirit of Bluefin: a desktop that stays out of your way and keeps itself up to date.",
        "cosmic-blue, c’est Fedora Atomic avec le bureau COSMIC, dans l’esprit de Bluefin : un bureau qui se fait oublier et se tient à jour tout seul.",
    ),
    ("Try it live", "Essayez-le en live"),
    (
        "cosmic-blue is running from the live media. Nothing is written to your disk, and your changes are lost when you shut down. You can start the installer at any time from the desktop or the dock.",
        "cosmic-blue fonctionne depuis le support live. Rien n’est écrit sur votre disque, et vos modifications sont perdues à l’extinction. Vous pouvez lancer l’installateur à tout moment depuis le bureau ou le dock.",
    ),
    ("What you get", "Ce qui vous attend"),
    ("Automatic updates", "Mises à jour automatiques"),
    (
        "The whole system updates in the background and applies at the next restart. If something goes wrong, boot the previous version.",
        "Le système entier se met à jour en arrière-plan et s’applique au redémarrage suivant. En cas de souci, démarrez la version précédente.",
    ),
    (
        "Apps from Flathub and COSMIC",
        "Applications de Flathub et COSMIC",
    ),
    (
        "Install apps with COSMIC Store, from Flathub and the COSMIC repository. They run sandboxed and update on their own.",
        "Installez vos applications avec COSMIC Store, depuis Flathub et le dépôt COSMIC. Elles tournent isolées et se mettent à jour seules.",
    ),
    ("Ready for the terminal", "Prêt pour le terminal"),
    (
        "Homebrew for command line tools, distrobox for other distributions, and ujust for common tasks.",
        "Homebrew pour les outils en ligne de commande, distrobox pour les autres distributions, et ujust pour les tâches courantes.",
    ),
    ("Ready to install?", "Prêt à installer ?"),
    (
        "The installer copies cosmic-blue to your disk. After that, it updates itself from the cosmic-blue image registry.",
        "L’installateur copie cosmic-blue sur votre disque. Ensuite, il se met à jour depuis le registre d’images de cosmic-blue.",
    ),
    ("Install cosmic-blue", "Installer cosmic-blue"),
    ("Keep trying", "Continuer l’essai"),
];

/// The session language, when it has a translation: "fr" or None
fn language() -> Option<&'static str> {
    static LANGUAGE: OnceLock<Option<&'static str>> = OnceLock::new();
    *LANGUAGE.get_or_init(|| {
        // Same order as gettext: LANGUAGE (a list), LC_ALL, LC_MESSAGES, LANG
        let mut names = Vec::new();
        if let Ok(list) = std::env::var("LANGUAGE") {
            names.extend(list.split(':').map(str::to_owned));
        }
        for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(value) = std::env::var(var) {
                names.push(value);
            }
        }
        for name in names.iter().filter(|name| !name.is_empty()) {
            let lang = name.split(['_', '.', '@']).next().unwrap_or_default();
            match lang {
                "fr" => return Some("fr"),
                "en" | "C" | "POSIX" => return None,
                _ => {}
            }
        }
        None
    })
}

/// Translates an English text to the session language
fn tr(text: &'static str) -> &'static str {
    match language() {
        Some("fr") => FRENCH
            .iter()
            .find(|(english, _)| *english == text)
            .map_or(text, |(_, french)| french),
        _ => text,
    }
}

#[derive(Clone, Debug)]
enum Action {
    Launch(&'static str),
    OpenUri(String),
    Install,
}

/// What a page shows under its description
enum Body {
    Logo,
    /// A large symbolic icon, as an illustration
    Icon(&'static str),
    /// Terminal commands, each with a button that copies it
    Commands(Vec<(&'static str, &'static str)>),
    /// Keyboard shortcuts
    Shortcuts(Vec<(&'static str, &'static str)>),
    /// Title and subtitle rows, without action
    Features(Vec<(&'static str, &'static str)>),
    /// Rows that open a web page
    Links(Vec<(&'static str, &'static str, String)>),
}

struct Page {
    title: &'static str,
    description: Option<&'static str>,
    /// Button above the rows
    button_before: Option<(&'static str, Action)>,
    body: Body,
    /// Button under the rows
    button_after: Option<(&'static str, Action)>,
    /// Short dimmed line at the bottom
    note: Option<&'static str>,
}

impl Page {
    fn new(title: &'static str, description: Option<&'static str>, body: Body) -> Self {
        Self {
            title,
            description,
            button_before: None,
            body,
            button_after: None,
            note: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    FirstSteps,
    Live,
}

fn docs_url() -> String {
    format!(
        "{SITE}{}docs/",
        if language() == Some("fr") { "fr/" } else { "" }
    )
}

fn pages(mode: Mode) -> Vec<Page> {
    match mode {
        Mode::FirstSteps => vec![
            Page::new(
                "Welcome to cosmic-blue",
                Some(
                    "Your system is installed and ready. Here are a few things worth knowing to get started.",
                ),
                Body::Logo,
            ),
            Page {
                button_before: Some(("Open COSMIC Store", Action::Launch("cosmic-store"))),
                ..Page::new(
                    "Install apps",
                    Some(
                        "Apps come from COSMIC Store, which offers Flathub and the COSMIC apps. They run apart from the system and update by themselves.",
                    ),
                    Body::Commands(vec![
                        (
                            "brew install",
                            "Command line tools, installed in your home folder with Homebrew",
                        ),
                        (
                            "distrobox create",
                            "Packages from other distributions, in a container that shares your home folder",
                        ),
                    ]),
                )
            },
            Page::new(
                "Updates and rollback",
                Some(
                    "The system, your apps and Homebrew packages update by themselves in the background. A system update applies at the next restart, without interrupting you.",
                ),
                Body::Commands(vec![
                    ("ujust update", "Update everything now, without waiting"),
                    (
                        "sudo bootc rollback",
                        "An update broke something? Go back to the previous version, then restart",
                    ),
                    (
                        "sudo bootc status",
                        "Show the current and previous versions",
                    ),
                ]),
            ),
            Page {
                button_after: Some(("Open a terminal", Action::Launch("cosmic-term"))),
                ..Page::new(
                    "Handy commands",
                    Some(
                        "ujust runs the shortcuts that come with the system. Type these in a terminal.",
                    ),
                    Body::Commands(vec![
                        ("ujust", "List all the shortcuts"),
                        (
                            "ujust devmode",
                            "Developer tools: Docker, VS Code, Podman Desktop…",
                        ),
                        (
                            "ujust toggle-dark-mode",
                            "Switch between the dark and light themes",
                        ),
                        (
                            "ujust reset-cosmic-theme",
                            "Bring the cosmic-blue look back: colors, icons, wallpaper",
                        ),
                    ]),
                )
            },
            Page {
                note: Some("COSMIC can be customized much further: Settings → Desktop."),
                ..Page::new(
                    "Tile your windows",
                    Some(
                        "COSMIC can arrange your windows side by side, without overlap, as you open them. It is off by default: turn it on with the tiling button in the panel, or with the keyboard.",
                    ),
                    Body::Shortcuts(vec![
                        ("Super + Y", "Turn tiling on or off"),
                        (
                            "Super + ← ↑ ↓ →",
                            "Go to the window on the left, right, above or below",
                        ),
                        (
                            "Super + Shift + ← ↑ ↓ →",
                            "Move the window to the left, right, up or down",
                        ),
                        ("Super + G", "Let a window float above the others"),
                    ]),
                )
            },
            Page::new(
                "Help and feedback",
                Some(
                    "Everything else is in the cosmic-blue guide. You can open this tour again from the app library: look for First steps.",
                ),
                Body::Links(vec![
                    (
                        "Documentation",
                        "Updates, software, commands and tips",
                        docs_url(),
                    ),
                    (
                        "Report a problem",
                        "Found a bug or something unclear? Tell us on GitHub",
                        ISSUES.to_owned(),
                    ),
                ]),
            ),
        ],
        Mode::Live => vec![
            Page::new(
                "Welcome to cosmic-blue",
                Some(
                    "cosmic-blue is Fedora Atomic with the COSMIC desktop, made in the spirit of Bluefin: a desktop that stays out of your way and keeps itself up to date.",
                ),
                Body::Logo,
            ),
            Page::new(
                "Try it live",
                Some(
                    "cosmic-blue is running from the live media. Nothing is written to your disk, and your changes are lost when you shut down. You can start the installer at any time from the desktop or the dock.",
                ),
                Body::Icon("media-optical-symbolic"),
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
                    "The installer copies cosmic-blue to your disk. After that, it updates itself from the cosmic-blue image registry.",
                ),
                Body::Icon("drive-harddisk-symbolic"),
            ),
        ],
    }
}

/// Every text of both tours, for the translation check
fn all_texts() -> Vec<&'static str> {
    let mut texts = vec![
        "Back",
        "Next",
        "Close",
        "Copy",
        "Copied",
        "First steps",
        "Open",
        "Get started",
        "Install cosmic-blue",
        "Keep trying",
    ];
    for mode in [Mode::FirstSteps, Mode::Live] {
        for page in pages(mode) {
            texts.push(page.title);
            texts.extend(page.description);
            texts.extend(page.note);
            texts.extend(page.button_before.map(|(label, _)| label));
            texts.extend(page.button_after.map(|(label, _)| label));
            match page.body {
                Body::Commands(rows) | Body::Shortcuts(rows) => {
                    texts.extend(rows.into_iter().map(|(_, description)| description));
                }
                Body::Features(rows) => {
                    texts.extend(
                        rows.into_iter()
                            .flat_map(|(title, subtitle)| [title, subtitle]),
                    );
                }
                Body::Links(rows) => {
                    texts.extend(
                        rows.into_iter()
                            .flat_map(|(title, subtitle, _)| [title, subtitle]),
                    );
                }
                Body::Logo | Body::Icon(_) => {}
            }
        }
    }
    texts
}

fn check_translations() -> bool {
    let texts = all_texts();
    let mut missing = 0;
    for text in &texts {
        if !FRENCH.iter().any(|(english, _)| english == text) {
            eprintln!("No fr translation for: {text}");
            missing += 1;
        }
    }
    println!("{} texts, {missing} missing translations", texts.len());
    missing == 0
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--check") {
        std::process::exit(if check_translations() { 0 } else { 1 });
    }
    // Same window as cosmic-initial-setup
    let settings = Settings::default().size_limits(Limits::NONE.max_width(900.0).max_height(650.0));
    if args.iter().any(|arg| arg == "--live") {
        cosmic::app::run::<App<LiveTour>>(settings, Mode::Live)?;
    } else {
        cosmic::app::run::<App<FirstStepsTour>>(settings, Mode::FirstSteps)?;
    }
    Ok(())
}

#[derive(Clone, Debug)]
enum Message {
    PageOpen(usize),
    /// Next (1) or previous (-1) page
    Turn(isize),
    Run(Action),
    Copy(&'static str),
    CloseToast(widget::ToastId),
    Exit,
}

/// Application ID of each tour: the name of its launcher (.desktop file),
/// which gives the window its name and icon in the dock
trait Tour: 'static {
    const APP_ID: &'static str;
}
struct FirstStepsTour;
impl Tour for FirstStepsTour {
    const APP_ID: &'static str = "io.github.cosmic_blue_os.FirstSteps";
}
struct LiveTour;
impl Tour for LiveTour {
    const APP_ID: &'static str = "io.github.cosmic_blue_os.Welcome";
}

struct App<T: Tour> {
    tour: std::marker::PhantomData<T>,
    core: Core,
    mode: Mode,
    pages: Vec<Page>,
    page_i: usize,
    toasts: Toasts<Message>,
    logo: Option<widget::svg::Handle>,
}

impl<T: Tour> Application for App<T> {
    type Executor = executor::Default;
    type Flags = Mode;
    type Message = Message;
    const APP_ID: &'static str = T::APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(mut core: Core, mode: Mode) -> (Self, Task<Message>) {
        core.window.show_headerbar = false;
        core.window.show_close = false;
        core.window.show_maximize = false;
        core.window.show_minimize = false;

        let mut app = App {
            tour: std::marker::PhantomData,
            core,
            mode,
            pages: pages(mode),
            page_i: 0,
            toasts: Toasts::new(Message::CloseToast),
            logo: Path::new(LOGO)
                .exists()
                .then(|| widget::svg::Handle::from_path(LOGO)),
        };
        let title = if mode == Mode::Live {
            "Welcome to cosmic-blue"
        } else {
            "First steps"
        };
        let task = match app.core.main_window_id() {
            Some(id) => app.set_window_title(tr(title).to_owned(), id),
            None => Task::none(),
        };
        (app, task)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PageOpen(page_i) => {
                if page_i < self.pages.len() {
                    self.page_i = page_i;
                }
            }
            Message::Turn(delta) => {
                if let Some(page_i) = self.page_i.checked_add_signed(delta) {
                    return self.update(Message::PageOpen(page_i));
                }
            }
            Message::Run(action) => run(&action),
            Message::Copy(command) => {
                return Task::batch([
                    cosmic::iced::clipboard::write(command.to_owned()),
                    self.toasts
                        .push(widget::toaster::Toast::new(tr("Copied")))
                        .map(cosmic::Action::App),
                ]);
            }
            Message::CloseToast(id) => self.toasts.remove(id),
            Message::Exit => return cosmic::iced::exit(),
        }
        Task::none()
    }

    fn on_escape(&mut self) -> Task<Message> {
        cosmic::iced::exit()
    }

    fn subscription(&self) -> Subscription<Message> {
        // Alt+Left and Alt+Right turn the pages (also used by the CI boot test)
        event::listen_with(|event, _status, _window| match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(named),
                modifiers,
                ..
            }) if modifiers.alt() => match named {
                Named::ArrowRight => Some(Message::Turn(1)),
                Named::ArrowLeft => Some(Message::Turn(-1)),
                _ => None,
            },
            _ => None,
        })
    }

    fn view(&self) -> Element<'_, Message> {
        let cosmic_theme::Spacing {
            space_xxs,
            space_s,
            space_m,
            space_l,
            space_xl,
            ..
        } = theme::spacing();
        let page = &self.pages[self.page_i];
        let last = self.page_i + 1 == self.pages.len();

        // Buttons, as in cosmic-initial-setup: a link on the left, Back and
        // the suggested action on the right
        let close_label = if self.mode == Mode::Live {
            "Keep trying"
        } else {
            "Close"
        };
        let mut button_row = widget::row::with_capacity(4)
            .spacing(space_xxs)
            .push(widget::button::link(tr(close_label)).on_press(Message::Exit))
            .push(widget::space::horizontal());
        if let Some(page_i) = self.page_i.checked_sub(1) {
            button_row = button_row
                .push(widget::button::standard(tr("Back")).on_press(Message::PageOpen(page_i)));
        }
        button_row = button_row.push(if !last {
            widget::button::suggested(tr("Next")).on_press(Message::PageOpen(self.page_i + 1))
        } else if self.mode == Mode::Live {
            let install = widget::button::suggested(tr("Install cosmic-blue"));
            if installer().is_some() {
                install.on_press(Message::Run(Action::Install))
            } else {
                install
            }
        } else {
            widget::button::suggested(tr("Get started")).on_press(Message::Exit)
        });

        let title = text::title2(tr(page.title)).center().width(Length::Fill);

        let mut content = widget::column::with_capacity(6)
            .spacing(space_m)
            .align_x(Alignment::Center);
        if let Some(description) = page.description {
            content = content.push(
                text::body(tr(description))
                    .align_x(Alignment::Center)
                    .width(Length::Fill),
            );
        }
        if let Some((label, action)) = &page.button_before {
            content = content.push(action_button(label, action));
        }
        content = content.push_maybe(self.body(&page.body, space_s));
        if let Some((label, action)) = &page.button_after {
            content = content.push(action_button(label, action));
        }
        if let Some(note) = page.note {
            content = content.push(
                text::caption(tr(note))
                    .align_x(Alignment::Center)
                    .width(Length::Fill),
            );
        }
        let content = content
            .apply(widget::scrollable)
            .apply(widget::container)
            .height(Length::Fill);

        let page_view = widget::column::with_capacity(7)
            .push(widget::space::vertical().height(space_xl))
            .push(title)
            .push(widget::space::vertical().height(space_l))
            .push(content)
            .push(widget::space::vertical().height(space_m))
            .push(button_row)
            .push(widget::space::vertical().height(space_l))
            .max_width(640.0)
            .width(640.0)
            .align_x(Alignment::Center)
            .apply(widget::container)
            .center_x(Length::Fill);

        widget::toaster(&self.toasts, page_view)
    }
}

impl<T: Tour> App<T> {
    fn body<'a>(&'a self, body: &'a Body, space_s: u16) -> Option<Element<'a, Message>> {
        let rows = match body {
            Body::Logo => {
                let logo: Element<'a, Message> = match &self.logo {
                    Some(handle) => widget::svg(handle.clone()).width(160).height(160).into(),
                    None => widget::icon::from_name("computer-symbolic")
                        .size(128)
                        .into(),
                };
                return Some(
                    widget::container(logo)
                        .padding([space_s, 0])
                        .center_x(Length::Fill)
                        .into(),
                );
            }
            Body::Icon(name) => {
                return Some(
                    widget::container(widget::icon::from_name(*name).size(128))
                        .padding([space_s, 0])
                        .center_x(Length::Fill)
                        .into(),
                );
            }
            Body::Commands(rows) => rows
                .iter()
                .map(|(command, description)| {
                    settings::item_row(vec![
                        two_lines(text::monotext(*command).into(), tr(description)),
                        widget::tooltip(
                            widget::button::icon(widget::icon::from_name("edit-copy-symbolic"))
                                .on_press(Message::Copy(command)),
                            text::body(tr("Copy")),
                            widget::tooltip::Position::Left,
                        )
                        .into(),
                    ])
                    .into()
                })
                .collect::<Vec<Element<'a, Message>>>(),
            Body::Shortcuts(rows) => rows
                .iter()
                .map(|(keys, description)| {
                    settings::item_row(vec![
                        text::body(tr(description))
                            .wrapping(Wrapping::Word)
                            .width(Length::Fill)
                            .into(),
                        text::caption(*keys).into(),
                    ])
                    .into()
                })
                .collect(),
            Body::Features(rows) => rows
                .iter()
                .map(|(title, subtitle)| {
                    settings::item_row(vec![two_lines(text::body(tr(title)).into(), tr(subtitle))])
                        .into()
                })
                .collect(),
            Body::Links(rows) => rows
                .iter()
                .map(|(title, subtitle, uri)| {
                    settings::item_row(vec![
                        two_lines(text::body(tr(title)).into(), tr(subtitle)),
                        widget::button::standard(tr("Open"))
                            .trailing_icon(widget::button::link::icon())
                            .on_press(Message::Run(Action::OpenUri(uri.clone())))
                            .into(),
                    ])
                    .into()
                })
                .collect(),
        };
        let mut section = settings::section();
        for row in rows {
            section = section.add(row);
        }
        Some(section.into())
    }
}

/// A row's text: a first line, and a caption under it (like the screen
/// reader row of cosmic-initial-setup's accessibility page)
fn two_lines<'a>(first: Element<'a, Message>, caption: &'static str) -> Element<'a, Message> {
    widget::column::with_capacity(2)
        .spacing(2)
        .push(first)
        .push(text::caption(caption).wrapping(Wrapping::Word))
        .width(Length::Fill)
        .into()
}

fn action_button<'a>(label: &'static str, action: &Action) -> Element<'a, Message> {
    let available = match action {
        Action::Launch(program) => in_path(program),
        _ => true,
    };
    let button = widget::button::standard(tr(label));
    if available {
        button.on_press(Message::Run(action.clone()))
    } else {
        button
    }
    .into()
}

fn in_path(program: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|path| std::env::split_paths(&path).any(|dir| dir.join(program).is_file()))
}

fn installer() -> Option<&'static str> {
    INSTALLERS.into_iter().find(|path| Path::new(path).exists())
}

fn spawn(program: &str, args: &[&str]) {
    if let Err(error) = Command::new(program).args(args).spawn() {
        eprintln!("Could not start {program}: {error}");
    }
}

fn run(action: &Action) {
    match action {
        Action::Launch(program) => spawn(program, &[]),
        Action::OpenUri(uri) => spawn("xdg-open", &[uri]),
        Action::Install => {
            if let Some(desktop) = installer() {
                spawn("gio", &["launch", desktop]);
                std::process::exit(0);
            }
        }
    }
}
