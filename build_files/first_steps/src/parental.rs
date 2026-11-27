// Celenn OS Parental controls: how long and when children can use the
// computer, and a family filter for the web.
//
//   - the parent's window: the child accounts (accounts without
//     administrator rights that have rules) with their picture and their
//     screen time today, a button that creates one, the other accounts
//     (a standard account becomes a child's with "This is a child"), and
//     for each child, in tabs: a daily limit (weekdays, weekend) and
//     bedtime hours, the applications (allowed one by one, or all those a
//     search finds at once, with Undo), the web filter, the screen time of
//     the last seven days and the applications most used in front;
//   - in a child's session, a small agent (autostart) that warns 10, 5 and
//     1 minutes before the session closes, with an "Ask for more time"
//     button, and tells the service which application is in front
//     (parental_focus.rs);
//   - that button's window: a parent gives some more time (a few clicks,
//     or a typed duration, up to 4 h) or no limit for the day, with their
//     password, there and then; it shows when the session will close.
//
// The rules are applied by a service (parental_daemon.rs: screen time,
// closing the session, the web filter) and at login (PAM). Changing them
// needs an administrator password: the app runs
// /usr/libexec/celenn-parental-helper (polkit action
// io.github.celenn_os.parental.manage, remembered a few minutes), and
// more time /usr/libexec/celenn-parental-extend (action
// io.github.celenn_os.parental.extend, asked every time, so that a
// child cannot give itself more).
//
//   celenn-parental-controls               the parent's window
//   celenn-parental-controls --user NAME   the same, on a child's page
//   celenn-parental-controls --more-time   the "more time" window
//   celenn-parental-controls --agent       the warnings (autostart)
//   celenn-parental-controls --check       image build test: every text
//                                               has its translations
//
// The look is that of Celenn OS Tools: a compact page header (the picture,
// the title and a caption, style.rs), then COSMIC settings sections.

use cosmic::app::{Core, Settings, Task};
use cosmic::iced::core::text::Wrapping;
use cosmic::iced::{Alignment, Length, Limits};
use cosmic::widget::{self, segmented_button, settings, text, toaster::Toasts};
use cosmic::{Application, ApplicationExt, Apply, Element, cosmic_theme, executor, theme};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

mod failure;
mod i18n;
mod network;
mod parental_accounts;
mod parental_agent;
mod parental_apps;
mod parental_focus;
mod parental_rules;
mod parental_view;
mod process;
mod style;
mod units;
use failure::{Card, CardAction, CardMessage, Outcome};
use i18n::{fill, tr};
use parental_accounts::{Account, accounts, user_from_name};
use parental_agent::*;
use parental_rules::{Now, Rules, Usage};
use parental_view::*;

pub(crate) const LANGUAGES: &[(&str, &str)] = &[
    ("de", include_str!("../po/parental/de.po")),
    ("es", include_str!("../po/parental/es.po")),
    ("fr", include_str!("../po/parental/fr.po")),
    ("it", include_str!("../po/parental/it.po")),
    ("pt_BR", include_str!("../po/parental/pt_BR.po")),
];
pub(crate) const TEMPLATE: &str = include_str!("../po/parental/celenn-parental-controls.pot");
pub(crate) const PO_DIR: &str = "po/parental";
pub(crate) const TEMPLATE_NAME: &str = "celenn-parental-controls.pot";

const HELPER: &str = "/usr/libexec/celenn-parental-helper";
const EXTEND: &str = "/usr/libexec/celenn-parental-extend";
const ICON: &str = "celenn-parental-controls";
/// The child's launcher (see more_time_launcher)
const MORE_TIME_ID: &str = "io.github.celenn_os.ParentalControls.MoreTime";
const AVATARS: &str = "/var/lib/AccountsService/icons";
/// Celenn OS's profile pictures (branding/avatars), one to pick for a new
/// child account
const FACES: &str = "/usr/share/pixmaps/faces";
/// COSMIC's picture for accounts without one (a link to our planet)
const DEFAULT_FACE: &str = "pop-robot.png";
/// Warnings before the session closes, in seconds
const WARNINGS: [u64; 3] = [600, 300, 60];
/// Steps of the - and + buttons, in minutes (any value can be typed)
const STEP: u32 = 15;
/// Bounds of a daily screen time, in minutes
const LIMIT_MIN: u32 = 5;
const LIMIT_MAX: u32 = 12 * 60;
/// More time a parent can give at once, in minutes, and its - and + step
/// (more: "No limit today")
const EXTRA_MIN: u32 = 5;
const EXTRA_MAX: u32 = 4 * 60;
const EXTRA_STEP: u32 = 5;
/// The ready-made amounts of more time, in minutes
const EXTRA_CHOICES: [u32; 4] = [15, 30, 60, 120];

/// More time one step up or down: the next multiple of the step
fn extra_step(value: u32, up: bool) -> u32 {
    let next = if up {
        value / EXTRA_STEP * EXTRA_STEP + EXTRA_STEP
    } else {
        (value.div_ceil(EXTRA_STEP) * EXTRA_STEP).saturating_sub(EXTRA_STEP)
    };
    next.clamp(EXTRA_MIN, EXTRA_MAX)
}

/// A time setting of a child account: a daily screen time or a bedtime end
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Field {
    Weekdays,
    Weekend,
    BedtimeStart,
    BedtimeEnd,
}

impl Field {
    fn is_clock(self) -> bool {
        matches!(self, Field::BedtimeStart | Field::BedtimeEnd)
    }

    fn get(self, rules: &Rules) -> u32 {
        match self {
            Field::Weekdays => rules.weekdays,
            Field::Weekend => rules.weekend,
            Field::BedtimeStart => rules.bedtime_start,
            Field::BedtimeEnd => rules.bedtime_end,
        }
    }

    fn set(self, rules: &mut Rules, value: u32) {
        match self {
            Field::Weekdays => rules.weekdays = value,
            Field::Weekend => rules.weekend = value,
            Field::BedtimeStart => rules.bedtime_start = value,
            Field::BedtimeEnd => rules.bedtime_end = value,
        }
    }

    fn show(self, value: u32) -> String {
        if self.is_clock() {
            parental_rules::clock(value)
        } else {
            duration(u64::from(value) * 60)
        }
    }

    /// The value one step up or down: the next multiple of the step, so that
    /// a typed 50 min goes to 45 or 1 h; times of day go round midnight
    fn step(self, value: u32, up: bool) -> u32 {
        let next = if up {
            value / STEP * STEP + STEP
        } else {
            value.div_ceil(STEP) * STEP - STEP.min(value.div_ceil(STEP) * STEP)
        };
        if self.is_clock() {
            if !up && value == 0 {
                24 * 60 - STEP
            } else {
                next % (24 * 60)
            }
        } else {
            next.clamp(LIMIT_MIN, LIMIT_MAX)
        }
    }

    /// What was typed: a duration (45, 45 min, 1 h, 1h20, 1:20) or a time
    /// of day (20:45, 20h45, 20.45, 2045, 20), whatever the language
    fn parse(self, typed: &str) -> Option<u32> {
        let typed = typed.trim().to_lowercase();
        // Groups of digits, of letters, and separators, in order
        let mut tokens: Vec<String> = Vec::new();
        for c in typed.chars() {
            let kind = |c: char| (c.is_ascii_digit(), c.is_alphabetic());
            match tokens.last_mut() {
                Some(last)
                    if !c.is_whitespace()
                        && last.chars().next().map(kind) == Some(kind(c))
                        && (c.is_ascii_digit() || c.is_alphabetic()) =>
                {
                    last.push(c)
                }
                _ if c.is_whitespace() => {}
                _ => tokens.push(c.to_string()),
            }
        }
        let mut numbers: Vec<String> = Vec::new();
        let mut hours_mark = false;
        for token in &tokens {
            let first = token.chars().next()?;
            if first.is_ascii_digit() {
                numbers.push(token.clone());
            } else if numbers.len() == 1 && (matches!(first, ':' | '.' | ',' | 'h' | 's' | 'o')) {
                // 1:20, 1.20, h, hours, heures, Std, Stunden, ore
                hours_mark = true;
            } else if !first.is_alphabetic() || !matches!(first, 'h' | 's' | 'o' | 'm') {
                return None;
            }
        }
        let values: Vec<u32> = numbers
            .iter()
            .map(|n| n.parse().ok())
            .collect::<Option<_>>()?;
        if self.is_clock() {
            let (h, m) = match values.as_slice() {
                [n] if numbers[0].len() >= 3 => (n / 100, n % 100),
                [h] => (*h, 0),
                [h, m] => (*h, *m),
                _ => return None,
            };
            (h <= 24 && m < 60 && h * 60 + m <= 24 * 60).then_some((h * 60 + m) % (24 * 60))
        } else {
            let minutes = match values.as_slice() {
                [m] if !hours_mark => *m,
                [h] => h.checked_mul(60)?,
                [h, m] if hours_mark && *m < 60 => h.checked_mul(60)? + m,
                _ => return None,
            };
            (minutes > 0).then_some(minutes.clamp(LIMIT_MIN, LIMIT_MAX))
        }
    }
}

/// Every text, for the translation check
const TEXTS: &[&str] = &[
    "Parental controls",
    "Choose how long and when your children can use this computer.",
    "Child accounts",
    "Other accounts",
    "Administrator · no limits",
    "Standard account · no limits",
    "This is a child",
    "Picture",
    "No limits",
    "Today: {} of {}",
    "Today: {}",
    "Add a child account",
    "An account without administrator rights, ready in one click",
    "Add…",
    "Only a parent (an administrator account) can change these settings.",
    "Back",
    "Screen time",
    "Limit screen time",
    "Only time in front of the screen counts. Warnings 10, 5 and 1 minutes before the end.",
    "Monday to Friday",
    "Saturday and Sunday",
    "Bedtime",
    "No computer at night",
    "The session closes and logging in is refused during these hours.",
    "From",
    "To",
    "Internet",
    "Block unsuitable websites",
    "Adult content, gambling and malicious sites, in every browser. Safe search is forced. Uses the CleanBrowsing family filter.",
    "This week",
    "Average: {} a day",
    "Mon",
    "Tue",
    "Wed",
    "Thu",
    "Fri",
    "Sat",
    "Sun",
    "{h} h {m} min",
    "{h} h",
    "{m} min",
    "Name",
    "Password",
    "The child logs in with this name and password.",
    "Create",
    "Cancel",
    "Saving the settings",
    "Recommended",
    "More time for {}",
    "A parent confirms with their password.",
    "Extra time",
    "No limit today",
    "Bedtime included, until tomorrow morning",
    "The session closes in {}.",
    "The session will close at {new} instead of {old}.",
    "The session will close at {}.",
    "No limit until tomorrow at {}.",
    "No limit for the rest of the day.",
    "Give {}",
    "Remove the limit",
    "At most 4 h. For more, turn on No limit today.",
    "Keep to bedtime",
    "The session closes at {} all the same.",
    "Bedtime ({}) does not apply to this time.",
    "{} minutes left",
    "1 minute left",
    "Your session closes at {}. Save your work.",
    "Ask for more time",
    "It is bedtime.",
    "Screen time is over for today.",
    "You can log in again at {}.",
    "You can log in again tomorrow.",
    "You can log in again tomorrow at {}.",
    "Type a duration, for example 45 or 1:20.",
    "Type a time, for example 20:45.",
    "Parental controls do not allow logging in now. Ask a parent.",
    "Applications",
    "Applications validated by a parent",
    "{} only uses the applications you ticked. Installing an application needs a parent's password.",
    "Allowed applications",
    "{} of {}",
    "Choose…",
    "Downloaded programs (AppImages, executable files) do not run on this account.",
    "Applications of {}",
    "Search for an application",
    "Today",
    "Most used applications",
    "Allow all",
    "Block all",
    "Undo",
    "{allowed} allowed out of {total}",
    "One application found",
    "{} applications found",
    "One application allowed",
    "One application blocked",
    "{} applications allowed",
    "{} applications blocked",
    "Allowed",
    "Blocked",
    "Applications installed later are blocked until you tick them.",
    "{} is not allowed",
    "A parent must allow it first in Parental controls.",
    "Program blocked",
    "«{}» cannot run on this account: downloaded programs are not allowed.",
    // The - and + buttons of times (their tooltip and name for screen readers)
    "Less time",
    "More time",
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--check") {
        let texts: Vec<&str> = [TEXTS, failure::TEXTS, units::TEXTS].concat();
        std::process::exit(if i18n::check(&texts) { 0 } else { 1 });
    }
    if args.iter().any(|arg| arg == "--agent") {
        agent();
        return Ok(());
    }
    let mode = if args.iter().any(|arg| arg == "--more-time") {
        Mode::MoreTime
    } else {
        Mode::Parent(
            args.iter()
                .position(|arg| arg == "--user")
                .and_then(|i| args.get(i + 1).cloned()),
        )
    };
    let size = match mode {
        // The child's picture, the title, the amount, ready-made amounts,
        // bedtime, no limit, when the session closes and the buttons
        Mode::MoreTime => cosmic::iced::Size::new(440.0, 620.0),
        // At most the height a 1366x768 screen or a VM window leaves
        // (about 690 px): the child's page is in tabs so that it fits
        Mode::Parent(_) => cosmic::iced::Size::new(760.0, 680.0),
    };
    let settings = Settings::default()
        .size(size)
        .size_limits(Limits::NONE.min_width(440.0).min_height(340.0));
    cosmic::app::run::<App>(settings, mode)?;
    Ok(())
}

fn user_name() -> String {
    std::env::var("USER").unwrap_or_default()
}

/// A Wayland app id as a name: its last part, capitalized
/// (org.kde.krita: Krita)
fn pretty_id(id: &str) -> String {
    let last = id.rsplit('.').next().unwrap_or(id);
    let mut chars = last.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

/// "1 h 20 min", "2 h", "25 min"
fn duration(seconds: u64) -> String {
    let minutes = seconds / 60;
    let (h, m) = (minutes / 60, minutes % 60);
    if h == 0 {
        tr("{m} min").replace("{m}", &m.to_string())
    } else if m == 0 {
        tr("{h} h").replace("{h}", &h.to_string())
    } else {
        tr("{h} h {m} min")
            .replace("{h}", &h.to_string())
            .replace("{m}", &format!("{m:02}"))
    }
}

// ---------------------------------------------------------------------------
// The windows
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum Mode {
    /// The parent's window, maybe on a child's page
    Parent(Option<String>),
    /// The "more time" window, in a child's session
    MoreTime,
}

#[derive(Clone, Debug, PartialEq)]
enum Page {
    Home,
    Child(String),
    /// The applications a child may use
    Apps(String),
    Add,
}

/// The tabs of a child's page
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tab {
    /// Screen time and bedtime
    Time,
    Apps,
    Internet,
    Week,
}

#[derive(Clone, Debug)]
enum Message {
    Accounts(Vec<Account>),
    Open(String),
    /// A tab of the child's page
    Tab(segmented_button::Entity),
    /// A picture picked for the new child account
    Face(PathBuf),
    Back,
    ShowAdd,
    /// A change to the rules of the account shown
    Rules(Rules),
    /// End of saving the rules: whether it worked (None: cancelled)
    Saved(String, Rules, Outcome),
    Name(String),
    Password(String),
    TogglePassword,
    Create,
    Created(String, Outcome),
    /// More time: a ready-made amount, - or +, typed (then done with it)
    Extra(u32),
    ExtraStep(bool),
    ExtraEdit(String),
    ExtraCommit,
    /// "No limit today" turned on or off
    NoLimit(bool),
    /// "Keep to bedtime" turned on or off
    KeepBedtime(bool),
    Give,
    Given(Outcome),
    Card(CardMessage),
    Close,
    CloseToast(widget::ToastId),
    /// - or + next to a time setting
    Step(Field, bool),
    /// Typing in a time setting, then done with it (Enter or elsewhere)
    Edit(Field, String),
    Commit,
    /// The installed applications, read
    AppsLoaded(Vec<parental_apps::App>),
    OpenApps(String),
    Search(String),
    /// "Applications validated by a parent" turned on or off
    AppsEnabled(bool),
    /// An application ticked or unticked
    Allow(String, bool),
    /// "Allow all" (true) or "Block all": the applications the search finds
    AllowAll(bool),
    /// A toast's Undo: the allowed applications as they were
    Undo(widget::ToastId, BTreeSet<String>),
}

struct App {
    core: Core,
    mode: Mode,
    page: Page,
    accounts: Vec<Account>,
    /// The rules being saved, and the next ones to save once that is done
    saving: bool,
    pending: Option<(String, Rules)>,
    name: String,
    password: String,
    password_hidden: bool,
    /// The pictures to pick from, and the one picked, for a new child
    faces: Vec<(PathBuf, widget::image::Handle)>,
    face: Option<PathBuf>,
    /// The tabs of a child's page
    tabs: segmented_button::SingleSelectModel,
    busy: bool,
    /// The more time to give, in minutes, what is typed in its field, and
    /// "No limit today"
    extra: u32,
    extra_typed: Option<String>,
    no_limit: bool,
    /// "Keep to bedtime": the more time does not go past it
    keep_bedtime: bool,
    /// The child's rules and time, for the "more time" window (read again
    /// after each change in it)
    child: Option<ChildTime>,
    /// The time setting being typed in, and what is typed
    editing: Option<(Field, String)>,
    /// The applications installed (read when a child's page opens)
    apps: Option<Vec<parental_apps::App>>,
    search: String,
    /// What did not work, until closed or done again
    card: Option<Card<Retry>>,
    toasts: Toasts<Message>,
}

/// What a card's "Try again" does again
#[derive(Clone, Debug)]
enum Retry {
    Save(String, Rules),
    Create,
    Give,
}

fn load() -> Task<Message> {
    Task::perform(
        async {
            tokio::task::spawn_blocking(accounts)
                .await
                .unwrap_or_default()
        },
        |accounts| cosmic::Action::App(Message::Accounts(accounts)),
    )
}

fn load_apps() -> Task<Message> {
    Task::perform(
        async {
            tokio::task::spawn_blocking(|| {
                parental_apps::installed(&std::env::var("LANG").unwrap_or_default())
            })
            .await
            .unwrap_or_default()
        },
        |apps| cosmic::Action::App(Message::AppsLoaded(apps)),
    )
}

fn save(user: String, rules: Rules) -> Task<Message> {
    let user_shown = user.clone();
    let args = vec![HELPER.to_owned(), "set".to_owned(), user];
    let json = rules.to_json();
    Task::perform(failure::run("pkexec", args, Some(json)), move |result| {
        cosmic::Action::App(Message::Saved(user_shown.clone(), rules.clone(), result))
    })
}

impl App {
    /// The more time typed, taken; Some: it was wrong (a toast says why)
    fn commit_extra(&mut self) -> Option<Task<Message>> {
        let typed = self.extra_typed.take()?;
        match Field::Weekdays.parse(&typed) {
            Some(minutes) if minutes > EXTRA_MAX => {
                self.extra = EXTRA_MAX;
                Some(self.toast("At most 4 h. For more, turn on No limit today."))
            }
            Some(minutes) => {
                self.extra = minutes.max(EXTRA_MIN);
                None
            }
            None => Some(self.toast("Type a duration, for example 45 or 1:20.")),
        }
    }

    fn toast(&mut self, message: &'static str) -> Task<Message> {
        self.toasts
            .push(widget::toaster::Toast::new(tr(message)))
            .map(cosmic::Action::App)
    }

    /// The applications the search finds (all of them without a search)
    fn found_apps(&self) -> Vec<&parental_apps::App> {
        let search = self.search.trim().to_lowercase();
        self.apps
            .iter()
            .flatten()
            .filter(|app| {
                search.is_empty()
                    || app.name.to_lowercase().contains(&search)
                    || app.comment.to_lowercase().contains(&search)
            })
            .collect()
    }

    /// The child account shown
    fn shown_user(&self) -> Option<&str> {
        match &self.page {
            Page::Child(user) | Page::Apps(user) => Some(user),
            _ => None,
        }
    }

    /// The rules of the child account shown
    fn shown_rules(&self) -> Option<Rules> {
        let user = self.shown_user()?;
        Some(
            self.account(user)?
                .rules
                .clone()
                .unwrap_or_else(Rules::none),
        )
    }

    fn account(&self, user: &str) -> Option<&Account> {
        self.accounts.iter().find(|account| account.user == user)
    }
}

impl Application for App {
    type Executor = executor::Default;
    type Flags = Mode;
    type Message = Message;
    const APP_ID: &'static str = "io.github.celenn_os.ParentalControls";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// "Back" in the header bar, on every page but the first
    fn header_start(&self) -> Vec<Element<'_, Message>> {
        if matches!(self.mode, Mode::Parent(_)) && self.page != Page::Home {
            vec![
                widget::button::text(tr("Back"))
                    .leading_icon(widget::icon::from_name("go-previous-symbolic"))
                    .on_press(Message::Back)
                    .into(),
            ]
        } else {
            Vec::new()
        }
    }

    fn init(core: Core, mode: Mode) -> (Self, Task<Message>) {
        let page = match &mode {
            Mode::Parent(Some(user)) => Page::Child(user.clone()),
            _ => Page::Home,
        };
        let mut app = App {
            core,
            mode,
            page,
            accounts: Vec::new(),
            saving: false,
            pending: None,
            name: String::new(),
            password: String::new(),
            password_hidden: true,
            faces: Vec::new(),
            face: None,
            tabs: segmented_button::Model::builder()
                .insert(|b| b.text(tr("Screen time")).data(Tab::Time).activate())
                .insert(|b| b.text(tr("Applications")).data(Tab::Apps))
                .insert(|b| b.text(tr("Internet")).data(Tab::Internet))
                .insert(|b| b.text(tr("This week")).data(Tab::Week))
                .build(),
            busy: false,
            extra: 30,
            extra_typed: None,
            no_limit: false,
            keep_bedtime: true,
            child: None,
            editing: None,
            apps: None,
            search: String::new(),
            card: None,
            toasts: Toasts::new(Message::CloseToast),
        };
        if matches!(app.mode, Mode::MoreTime) {
            app.child = ChildTime::read();
            // A small dialog: only its close button
            app.core.window.show_maximize = false;
            app.core.window.show_minimize = false;
        }
        let title = match app.core.main_window_id() {
            Some(id) => app.set_window_title(tr("Parental controls").to_owned(), id),
            None => Task::none(),
        };
        let apps = match &app.page {
            Page::Child(_) => load_apps(),
            _ => Task::none(),
        };
        (app, Task::batch([title, load(), apps]))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Accounts(accounts) => self.accounts = accounts,
            Message::Open(user) => {
                self.page = Page::Child(user);
                self.tabs.activate_position(0);
                if self.apps.is_none() {
                    return load_apps();
                }
            }
            Message::Back => {
                self.editing = None;
                // From the applications, back to the child's page
                if let Page::Apps(user) = &self.page {
                    self.page = Page::Child(user.clone());
                    return Task::none();
                }
                self.page = Page::Home;
                return load();
            }
            Message::Tab(entity) => {
                self.tabs.activate(entity);
                // The week's applications are shown by name
                if self.tabs.active_data::<Tab>() == Some(&Tab::Week) && self.apps.is_none() {
                    return load_apps();
                }
            }
            Message::Face(face) => self.face = Some(face),
            Message::ShowAdd => {
                self.name.clear();
                self.password.clear();
                self.faces = faces();
                // A different picture for each new child, to change at will
                self.face = self
                    .faces
                    .get(self.accounts.len() * 7 % self.faces.len().max(1))
                    .map(|(face, _)| face.clone());
                self.page = Page::Add;
            }
            Message::Rules(rules) => {
                let Some(user) = self.shown_user().map(str::to_owned) else {
                    return Task::none();
                };
                // Shown at once; saved one change at a time (the password
                // is asked once, then remembered for a few minutes)
                if let Some(account) = self.accounts.iter_mut().find(|a| a.user == user) {
                    account.rules = Some(rules.clone());
                }
                if self.saving {
                    self.pending = Some((user, rules));
                } else {
                    self.saving = true;
                    return save(user, rules);
                }
            }
            Message::Saved(user, rules, result) => {
                self.saving = false;
                if matches!(result, Some(Ok(()))) {
                    if let Some((user, rules)) = self.pending.take() {
                        self.saving = true;
                        return save(user, rules);
                    }
                    return Task::none();
                }
                // Cancelled or failed: show what is saved
                self.pending = None;
                if let Some(Err(failure)) = result {
                    self.card = Some(Card::new(
                        tr("Saving the settings"),
                        failure,
                        Retry::Save(user, rules),
                    ));
                }
                return load();
            }
            Message::Name(name) => self.name = name,
            Message::Password(password) => self.password = password,
            Message::TogglePassword => self.password_hidden = !self.password_hidden,
            Message::Create => {
                let name = self.name.trim().to_owned();
                if name.is_empty() || self.password.is_empty() || self.busy {
                    return Task::none();
                }
                let taken: Vec<String> = self.accounts.iter().map(|a| a.user.clone()).collect();
                let user = user_from_name(&name, &taken);
                self.busy = true;
                let mut args = vec![
                    HELPER.to_owned(),
                    "add-child".to_owned(),
                    user.clone(),
                    name,
                ];
                if let Some(face) = &self.face {
                    args.push(face.to_string_lossy().into_owned());
                }
                let input = format!("{}\n{}", self.password, Rules::default().to_json());
                self.card = None;
                return Task::perform(failure::run("pkexec", args, Some(input)), move |result| {
                    cosmic::Action::App(Message::Created(user.clone(), result))
                });
            }
            Message::Created(user, result) => {
                self.busy = false;
                match result {
                    Some(Ok(())) => {
                        self.password.clear();
                        self.page = Page::Child(user);
                        self.tabs.activate_position(0);
                        return load();
                    }
                    Some(Err(failure)) => {
                        self.card =
                            Some(Card::new(tr("Add a child account"), failure, Retry::Create));
                    }
                    None => {}
                }
            }
            Message::Extra(minutes) => {
                self.extra_typed = None;
                self.extra = minutes;
                self.child = ChildTime::read();
            }
            Message::ExtraStep(up) => {
                self.extra_typed = None;
                self.extra = extra_step(self.extra, up);
                self.child = ChildTime::read();
            }
            Message::ExtraEdit(typed) => self.extra_typed = Some(typed),
            Message::ExtraCommit => {
                self.child = ChildTime::read();
                return self.commit_extra().unwrap_or_else(Task::none);
            }
            Message::KeepBedtime(on) => {
                self.keep_bedtime = on;
                self.child = ChildTime::read();
            }
            Message::NoLimit(on) => {
                self.no_limit = on;
                self.child = ChildTime::read();
            }
            Message::Give => {
                // What was typed and not done with yet counts
                if let Some(wrong) = self.commit_extra() {
                    return wrong;
                }
                self.busy = true;
                let mut args = vec![EXTEND.to_owned(), user_name()];
                if self.no_limit {
                    args.push("day".to_owned());
                } else {
                    args.push(self.extra.to_string());
                    args.push(if self.keep_bedtime { "bedtime" } else { "late" }.to_owned());
                }
                self.card = None;
                return Task::perform(failure::run("pkexec", args, None), |result| {
                    cosmic::Action::App(Message::Given(result))
                });
            }
            Message::Given(result) => {
                self.busy = false;
                match result {
                    Some(Ok(())) => std::process::exit(0),
                    Some(Err(failure)) => {
                        self.card = Some(Card::new(tr("Ask for more time"), failure, Retry::Give));
                    }
                    None => {}
                }
            }
            Message::Card(message) => {
                let Some(card) = &mut self.card else {
                    return Task::none();
                };
                match card.update(message) {
                    CardAction::None => {}
                    CardAction::Close => self.card = None,
                    CardAction::Retry(retry) => {
                        self.card = None;
                        match retry {
                            Retry::Save(user, rules) => {
                                self.saving = true;
                                return save(user, rules);
                            }
                            Retry::Create => return self.update(Message::Create),
                            Retry::Give => return self.update(Message::Give),
                        }
                    }
                }
            }
            Message::Step(field, up) => {
                self.editing = None;
                if let Some(mut rules) = self.shown_rules() {
                    let value = field.step(field.get(&rules), up);
                    field.set(&mut rules, value);
                    return self.update(Message::Rules(rules));
                }
            }
            Message::Edit(field, typed) => self.editing = Some((field, typed)),
            Message::Commit => {
                let Some((field, typed)) = self.editing.take() else {
                    return Task::none();
                };
                let Some(mut rules) = self.shown_rules() else {
                    return Task::none();
                };
                match field.parse(&typed) {
                    Some(value) if value == field.get(&rules) => {}
                    Some(value) => {
                        field.set(&mut rules, value);
                        return self.update(Message::Rules(rules));
                    }
                    None => {
                        return self.toast(if field.is_clock() {
                            "Type a time, for example 20:45."
                        } else {
                            "Type a duration, for example 45 or 1:20."
                        });
                    }
                }
            }
            Message::AppsLoaded(apps) => self.apps = Some(apps),
            Message::OpenApps(user) => {
                self.search.clear();
                self.page = Page::Apps(user);
                if self.apps.is_none() {
                    return load_apps();
                }
            }
            Message::Search(search) => self.search = search,
            Message::AppsEnabled(on) => {
                let Some(mut rules) = self.shown_rules() else {
                    return Task::none();
                };
                rules.apps_enabled = on;
                // The first time: the usual applications ticked, not the
                // terminal, system tools and the like
                if on && rules.allowed_apps.is_empty() {
                    rules.allowed_apps = self
                        .apps
                        .iter()
                        .flatten()
                        .filter(|app| app.ticked_first())
                        .map(|app| app.id.clone())
                        .collect();
                }
                return self.update(Message::Rules(rules));
            }
            Message::Allow(id, on) => {
                let Some(mut rules) = self.shown_rules() else {
                    return Task::none();
                };
                if on {
                    rules.allowed_apps.insert(id);
                } else {
                    rules.allowed_apps.remove(&id);
                }
                return self.update(Message::Rules(rules));
            }
            Message::AllowAll(on) => {
                let Some(mut rules) = self.shown_rules() else {
                    return Task::none();
                };
                let before = rules.allowed_apps.clone();
                let ids: Vec<String> = self
                    .found_apps()
                    .into_iter()
                    .map(|app| app.id.clone())
                    .collect();
                for id in ids {
                    if on {
                        rules.allowed_apps.insert(id);
                    } else {
                        rules.allowed_apps.remove(&id);
                    }
                }
                let changed = before.symmetric_difference(&rules.allowed_apps).count();
                if changed == 0 {
                    return Task::none();
                }
                let text = match (on, changed) {
                    (true, 1) => tr("One application allowed").to_owned(),
                    (false, 1) => tr("One application blocked").to_owned(),
                    (true, n) => fill(tr("{} applications allowed"), &n.to_string()),
                    (false, n) => fill(tr("{} applications blocked"), &n.to_string()),
                };
                let toast = self
                    .toasts
                    .push(
                        widget::toaster::Toast::new(text)
                            .action(tr("Undo").to_owned(), move |id| {
                                Message::Undo(id, before.clone())
                            }),
                    )
                    .map(cosmic::Action::App);
                return Task::batch([toast, self.update(Message::Rules(rules))]);
            }
            Message::Undo(id, allowed) => {
                self.toasts.remove(id);
                let Some(mut rules) = self.shown_rules() else {
                    return Task::none();
                };
                rules.allowed_apps = allowed;
                return self.update(Message::Rules(rules));
            }
            Message::Close => std::process::exit(0),
            Message::CloseToast(id) => self.toasts.remove(id),
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let page = match (&self.mode, &self.page) {
            (Mode::MoreTime, _) => self.more_time(),
            (Mode::Parent(_), Page::Home) => self.home(),
            (Mode::Parent(_), Page::Child(user)) => self.child(user),
            (Mode::Parent(_), Page::Apps(user)) => self.apps_page(user),
            (Mode::Parent(_), Page::Add) => self.add(),
        };
        let page = match &self.card {
            None => page,
            Some(card) => widget::column::with_capacity(2)
                .push(
                    widget::container(card.view(Message::Card))
                        .max_width(640)
                        .padding(theme::spacing().space_s)
                        .apply(widget::container)
                        .center_x(Length::Fill),
                )
                .push(page)
                .into(),
        };
        widget::toaster(&self.toasts, page)
    }
}
#[cfg(test)]
mod tests {
    use super::Field;

    #[test]
    fn typed_durations() {
        for (typed, minutes) in [
            ("45", 45),
            ("45 min", 45),
            ("1 h", 60),
            ("1h20", 80),
            ("1 h 20 min", 80),
            ("1:20", 80),
            ("2 Std", 120),
            ("2", 5),
            ("30h", 720),
        ] {
            assert_eq!(Field::Weekdays.parse(typed), Some(minutes), "{typed}");
        }
        for typed in ["", "0", "abc", "1:75", "-5", "1 x"] {
            assert_eq!(Field::Weekdays.parse(typed), None, "{typed}");
        }
    }

    #[test]
    fn typed_times() {
        for (typed, minute) in [
            ("20:45", 1245),
            ("20h45", 1245),
            ("20.45", 1245),
            ("2045", 1245),
            ("20", 1200),
            ("7", 420),
            ("0:00", 0),
            ("24:00", 0),
        ] {
            assert_eq!(Field::BedtimeStart.parse(typed), Some(minute), "{typed}");
        }
        for typed in ["25:00", "20:60", "abc", "2475"] {
            assert_eq!(Field::BedtimeStart.parse(typed), None, "{typed}");
        }
    }

    #[test]
    fn steps() {
        assert_eq!(Field::Weekdays.step(50, true), 60);
        assert_eq!(Field::Weekdays.step(50, false), 45);
        assert_eq!(Field::Weekdays.step(60, false), 45);
        assert_eq!(Field::Weekdays.step(15, false), 5);
        assert_eq!(Field::Weekdays.step(720, true), 720);
        assert_eq!(Field::BedtimeStart.step(1245, true), 1260);
        assert_eq!(Field::BedtimeStart.step(1435, true), 0);
        assert_eq!(Field::BedtimeStart.step(0, false), 1425);
        assert_eq!(Field::BedtimeStart.step(10, false), 0);
    }
}
