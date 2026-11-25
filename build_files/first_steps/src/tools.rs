// cosmic-blue Tools: what people would otherwise do with commands (`ujust`,
// bootc...), as buttons, in one window with a sidebar like COSMIC Settings.
//
//   Overview      where things stand, in four cards, and a search through
//                 every setting (overview.rs; the window's first page)
//   Updates       the system version, check now, go back, channel
//                 (updates_page.rs; the Updates launcher opens this page)
//   Maintenance   disk space and cleaning up, the apps that come with
//                 cosmic-blue, firmware (maintenance.rs)
//   Computer      printers and scanners, battery charge limit, NVIDIA
//                 edition, firmware settings
//   Extra software  the kinds of apps of the first steps tour (apps.rs),
//                 virtual machines, Microsoft fonts, FlatPark
//   Appearance    dark theme, the cosmic-blue look, reset, change
//                 computers (these three: more.rs)
//   Help          documentation, questions, bug report (help.rs)
//
// Each button runs the same program as the matching `ujust` command, so the
// terminal and the window do the same thing. Changes that need root go
// through small helpers run with pkexec (an administrator password).
// No notifications about updates: the system looks after itself, this is
// for when people want to look or act. The one notification is a disk almost
// full (housekeeping.rs), which people have to act on.
//
//   cosmic-blue-tools               the window, on Overview
//   cosmic-blue-tools --updates     the window, on Updates (also when run as
//                                   /usr/libexec/cosmic-blue-updates)
//   cosmic-blue-tools --maintenance / --computer / --software /
//                     --appearance / --help-page   the window, on that page
//   cosmic-blue-tools --restore     the window, on Appearance, choosing a
//                                   settings file to take back (transfer.rs)
//   cosmic-blue-tools --housekeeping  empties the old trash, warns when the
//                                   disk is almost full (no window; run by
//                                   cosmic-blue-housekeeping.timer)
//   cosmic-blue-tools --status / --older   see updates_page.rs (tests)
//   cosmic-blue-tools --check       image build test: every text has its
//                                   translations (no window)
//
// Text is in English, translated from the PO files of po/tools/.

use cosmic::app::{Core, Settings, Task};
use cosmic::iced::Limits;
use cosmic::widget::nav_bar;
use cosmic::{Application, ApplicationExt, Element, executor};

mod apps;
mod common;
mod devtools;
mod disks;
mod edition;
mod failure;
mod hardware;
mod help;
mod housekeeping;
mod i18n;
mod layout;
mod maintenance;
mod more;
mod network;
mod overview;
mod style;
// What the tour reads from the system; Tools uses the settings folder and
// the first-boot apps (transfer.rs)
#[allow(dead_code)]
mod system;
mod transfer;
mod update_progress;
mod updates_page;
use i18n::tr;

pub(crate) const LANGUAGES: &[(&str, &str)] = &[
    ("de", include_str!("../po/tools/de.po")),
    ("es", include_str!("../po/tools/es.po")),
    ("fr", include_str!("../po/tools/fr.po")),
    ("it", include_str!("../po/tools/it.po")),
    ("pt_BR", include_str!("../po/tools/pt_BR.po")),
];
pub(crate) const TEMPLATE: &str = include_str!("../po/tools/cosmic-blue-tools.pot");
pub(crate) const PO_DIR: &str = "po/tools";
pub(crate) const TEMPLATE_NAME: &str = "cosmic-blue-tools.pot";

/// The window's own texts; each page lists its own
const TEXTS: &[&str] = &[
    "cosmic-blue Tools",
    "Some changes ask for the password of a parent or an administrator.",
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PageId {
    Overview,
    Updates,
    Maintenance,
    Computer,
    Software,
    Appearance,
    Help,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let options = &args[1..];
    if options.iter().any(|arg| arg == "--check") {
        let texts: Vec<&str> = [
            TEXTS,
            updates_page::TEXTS,
            update_progress::TEXTS,
            maintenance::TEXTS,
            more::TEXTS,
            help::TEXTS,
            housekeeping::TEXTS,
            failure::TEXTS,
            overview::TEXTS,
            transfer::TEXTS,
            disks::TEXTS,
        ]
        .concat();
        let mut sorted = texts.clone();
        sorted.extend(apps::texts());
        sorted.sort_unstable();
        sorted.dedup();
        std::process::exit(if i18n::check(&sorted) { 0 } else { 1 });
    }
    if options.iter().any(|arg| arg == "--housekeeping") {
        std::process::exit(housekeeping::run());
    }
    if let Some(code) = updates_page::command_line(options) {
        std::process::exit(code);
    }
    // Run as /usr/libexec/cosmic-blue-updates (a link): the Updates page
    let as_updates = args
        .first()
        .is_some_and(|name| name.ends_with("cosmic-blue-updates"));
    let page = if as_updates || options.iter().any(|arg| arg == "--updates") {
        PageId::Updates
    } else if options.iter().any(|arg| arg == "--maintenance") {
        PageId::Maintenance
    } else if options.iter().any(|arg| arg == "--computer") {
        PageId::Computer
    } else if options.iter().any(|arg| arg == "--software") {
        PageId::Software
    } else if options
        .iter()
        .any(|arg| arg == "--appearance" || arg == "--restore")
    {
        PageId::Appearance
    } else if options.iter().any(|arg| arg == "--help-page") {
        PageId::Help
    } else {
        PageId::Overview
    };
    let settings = Settings::default()
        .size(cosmic::iced::Size::new(1000.0, 760.0))
        .size_limits(Limits::NONE.min_width(520.0).min_height(480.0));
    cosmic::app::run::<App>(settings, page)?;
    Ok(())
}

#[derive(Clone, Debug)]
pub enum Message {
    Overview(overview::Message),
    Updates(updates_page::Message),
    Maintenance(maintenance::Message),
    More(more::Message),
    Help(help::Message),
}

struct App {
    core: Core,
    nav: nav_bar::Model,
    overview: overview::Page,
    updates: updates_page::Page,
    maintenance: maintenance::Page,
    more: more::Page,
    help: help::Page,
}

impl App {
    /// A page was opened: the Overview reads its facts again
    fn shown(&mut self) -> Task<Message> {
        if self.page() == PageId::Overview {
            overview::refresh().map(|action| action.map(Message::Overview))
        } else {
            Task::none()
        }
    }

    fn page(&self) -> PageId {
        self.nav
            .active_data::<PageId>()
            .copied()
            .unwrap_or(PageId::Overview)
    }
}

impl Application for App {
    type Executor = executor::Default;
    type Flags = PageId;
    type Message = Message;
    const APP_ID: &'static str = "io.github.cosmic_blue_os.Tools";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(mut core: Core, first: PageId) -> (Self, Task<Message>) {
        core.nav_bar_set_toggled(true);
        let mut nav = nav_bar::Model::default();
        for (id, icon) in [
            // The glyph of each page's icon (branding/icons/build.py,
            // TOOLS_SYMBOLIC), so that the sidebar matches the pages
            (PageId::Overview, "cosmic-blue-tools-overview-symbolic"),
            (PageId::Updates, "cosmic-blue-tools-updates-symbolic"),
            (
                PageId::Maintenance,
                "cosmic-blue-tools-maintenance-symbolic",
            ),
            (PageId::Computer, "cosmic-blue-tools-computer-symbolic"),
            (PageId::Software, "cosmic-blue-tools-software-symbolic"),
            (PageId::Appearance, "cosmic-blue-tools-appearance-symbolic"),
            (PageId::Help, "cosmic-blue-tools-help-symbolic"),
        ] {
            let entity = nav
                .insert()
                .text(tr(overview::page_name(id)))
                .icon(cosmic::widget::icon::from_name(icon))
                .data(id);
            if id == first {
                entity.activate();
            }
        }
        let (overview, overview_task) = overview::Page::new();
        let (updates, updates_task) = updates_page::Page::new();
        let (maintenance, maintenance_task) = maintenance::Page::new();
        let (more, more_task) = more::Page::new();
        let (help, help_task) = help::Page::new();
        let mut app = App {
            core,
            nav,
            overview,
            updates,
            maintenance,
            more,
            help,
        };
        let title = match app.core.main_window_id() {
            Some(id) => app.set_window_title(tr("cosmic-blue Tools").to_owned(), id),
            None => Task::none(),
        };
        (
            app,
            Task::batch([
                title,
                overview_task.map(|action| action.map(Message::Overview)),
                updates_task.map(|action| action.map(Message::Updates)),
                maintenance_task.map(|action| action.map(Message::Maintenance)),
                more_task.map(|action| action.map(Message::More)),
                help_task.map(|action| action.map(Message::Help)),
            ]),
        )
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Message> {
        self.nav.activate(id);
        self.shown()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Overview(overview::Message::Go(page)) => {
                let entity = self
                    .nav
                    .iter()
                    .find(|&entity| self.nav.data::<PageId>(entity) == Some(&page));
                if let Some(entity) = entity {
                    self.nav.activate(entity);
                }
                self.shown()
            }
            Message::Overview(message) => self
                .overview
                .update(message)
                .map(|action| action.map(Message::Overview)),
            Message::Updates(message) => self
                .updates
                .update(message)
                .map(|action| action.map(Message::Updates)),
            Message::Maintenance(message) => self
                .maintenance
                .update(message)
                .map(|action| action.map(Message::Maintenance)),
            Message::More(message) => self
                .more
                .update(message)
                .map(|action| action.map(Message::More)),
            Message::Help(message) => self
                .help
                .update(message)
                .map(|action| action.map(Message::Help)),
        }
    }

    /// Change computers: the dialog that takes a settings file back
    fn dialog(&self) -> Option<Element<'_, Message>> {
        self.more.dialog().map(|dialog| dialog.map(Message::More))
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Message> {
        self.updates.subscription().map(Message::Updates)
    }

    fn view(&self) -> Element<'_, Message> {
        match self.page() {
            PageId::Overview => self.overview.view().map(Message::Overview),
            PageId::Updates => self.updates.view().map(Message::Updates),
            PageId::Maintenance => self.maintenance.view().map(Message::Maintenance),
            PageId::Computer => self.more.view(more::Kind::Computer).map(Message::More),
            PageId::Software => self.more.view(more::Kind::Software).map(Message::More),
            PageId::Appearance => self.more.view(more::Kind::Appearance).map(Message::More),
            PageId::Help => self.help.view().map(Message::Help),
        }
    }
}
