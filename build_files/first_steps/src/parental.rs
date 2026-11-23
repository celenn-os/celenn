// cosmic-blue Parental controls: how long and when children can use the
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
// /usr/libexec/cosmic-blue-parental-helper (polkit action
// io.github.cosmic_blue_os.parental.manage, remembered a few minutes), and
// more time /usr/libexec/cosmic-blue-parental-extend (action
// io.github.cosmic_blue_os.parental.extend, asked every time, so that a
// child cannot give itself more).
//
//   cosmic-blue-parental-controls               the parent's window
//   cosmic-blue-parental-controls --user NAME   the same, on a child's page
//   cosmic-blue-parental-controls --more-time   the "more time" window
//   cosmic-blue-parental-controls --agent       the warnings (autostart)
//   cosmic-blue-parental-controls --check       image build test: every text
//                                               has its translations
//
// The look is that of cosmic-blue Tools: a compact page header (the picture,
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
mod parental_apps;
mod parental_focus;
mod parental_rules;
mod style;
use failure::{Card, CardAction, CardMessage, Outcome};
use i18n::tr;
use parental_rules::{Now, Rules, Usage};

pub(crate) const LANGUAGES: &[(&str, &str)] = &[
    ("de", include_str!("../po/parental/de.po")),
    ("es", include_str!("../po/parental/es.po")),
    ("fr", include_str!("../po/parental/fr.po")),
    ("it", include_str!("../po/parental/it.po")),
    ("pt_BR", include_str!("../po/parental/pt_BR.po")),
];
pub(crate) const TEMPLATE: &str = include_str!("../po/parental/cosmic-blue-parental-controls.pot");
pub(crate) const PO_DIR: &str = "po/parental";
pub(crate) const TEMPLATE_NAME: &str = "cosmic-blue-parental-controls.pot";

const HELPER: &str = "/usr/libexec/cosmic-blue-parental-helper";
const EXTEND: &str = "/usr/libexec/cosmic-blue-parental-extend";
const ICON: &str = "cosmic-blue-parental-controls";
/// The child's launcher (see more_time_launcher)
const MORE_TIME_ID: &str = "io.github.cosmic_blue_os.ParentalControls.MoreTime";
const AVATARS: &str = "/var/lib/AccountsService/icons";
/// cosmic-blue's profile pictures (branding/avatars), one to pick for a new
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
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--check") {
        let texts: Vec<&str> = [TEXTS, failure::TEXTS].concat();
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

/// A translated text with its {} replaced
fn fill(text: &str, value: &str) -> String {
    text.replacen("{}", value, 1)
}

// ---------------------------------------------------------------------------
// The agent: warnings in a child's session
// ---------------------------------------------------------------------------

/// The app id in front, for the service (parental_rules::FOCUS); none: the
/// file goes (a new file, renamed over the old one, so that the service
/// never reads half of it)
fn write_focus(path: &Path, app: Option<&str>) {
    let Some(app) = app.and_then(parental_rules::clean_app_id) else {
        let _ = std::fs::remove_file(path);
        return;
    };
    let Some(dir) = path.parent() else {
        return;
    };
    let new = dir.join(".parental-focus.new");
    let done = std::fs::create_dir_all(dir)
        .and_then(|()| std::fs::write(&new, app))
        .and_then(|()| std::fs::rename(&new, path));
    if let Err(error) = done {
        eprintln!("Could not write {}: {error}", path.display());
    }
}

/// Runs for the whole session of an account with rules: warns 10, 5 and 1
/// minutes before the session closes (the service closes it). Granting more
/// time makes the warnings come again before the new end.
fn agent() {
    let user = user_name();
    // Administrators cannot have rules (see cosmic-blue-parental-helper)
    let groups = std::process::Command::new("id")
        .arg("-nG")
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
        .unwrap_or_default();
    if groups.split_whitespace().any(|group| group == "wheel") {
        more_time_launcher(false);
        return;
    }
    let mut warned: BTreeSet<u64> = BTreeSet::new();
    let mut launcher = None;
    let home = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
    let applications = home.join(".local/share/applications");
    let lang = std::env::var("LANG").unwrap_or_default();
    let mut hidden: Option<Option<BTreeSet<String>>> = None;
    let mut rounds: u32 = 0;
    // The application in front, for the parent's "Most used applications"
    let focus = parental_focus::Focus::default();
    {
        let focus = focus.clone();
        std::thread::spawn(move || parental_focus::watch(focus));
    }
    let focus_file = home.join(parental_rules::FOCUS);
    loop {
        let active = focus.lock().ok().and_then(|focus| focus.clone());
        write_focus(&focus_file, active.as_deref());
        // Rules can be added or removed while the session runs
        let rules = Rules::load(&user);
        // The applications not allowed, hidden (again every minute: some
        // may have been installed)
        let allowed = rules
            .as_ref()
            .filter(|r| r.apps_enabled)
            .map(|r| r.allowed_apps.clone());
        if hidden.as_ref() != Some(&allowed) || (allowed.is_some() && rounds.is_multiple_of(6)) {
            // With the Flatpaks installed for the child alone (never
            // allowed: the parent's list only has the system's)
            let mut dirs: Vec<PathBuf> =
                parental_apps::APP_DIRS.iter().map(PathBuf::from).collect();
            dirs.insert(
                0,
                home.join(".local/share/flatpak/exports/share/applications"),
            );
            parental_apps::hide(
                &applications,
                &parental_apps::installed_in(&dirs, &lang),
                allowed.as_ref(),
            );
            hidden = Some(allowed);
        }
        rounds = rounds.wrapping_add(1);
        let limited = rules.as_ref().is_some_and(|rules| rules.limit_enabled);
        if launcher != Some(limited) {
            more_time_launcher(limited);
            launcher = Some(limited);
        }
        if let (Some(rules), Some(now)) = (rules, Now::read()) {
            let usage = Usage::load(&user);
            let extra = parental_rules::load_extra(&user);
            let verdict = parental_rules::verdict(&rules, usage.used_today(&now), &extra, &now);
            if let Some(remaining) = verdict.remaining.filter(|r| *r > 0) {
                // More time given: warn again before the new end
                warned.retain(|threshold| remaining <= threshold + 30);
                let due: Vec<u64> = WARNINGS
                    .iter()
                    .copied()
                    .filter(|threshold| remaining <= *threshold && !warned.contains(threshold))
                    .collect();
                if !due.is_empty() {
                    warned.extend(due);
                    let end = (now.minute + remaining.div_ceil(60) as u32) % (24 * 60);
                    notify(remaining, &parental_rules::clock(end));
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}

/// The child's "Ask for more time" launcher, in its applications: present
/// while its screen time is limited, so that it can ask at any time, not only
/// from a warning
fn more_time_launcher(show: bool) {
    let Some(home) = std::env::var_os("HOME") else {
        return;
    };
    let path = Path::new(&home).join(format!(".local/share/applications/{MORE_TIME_ID}.desktop"));
    if !show {
        let _ = std::fs::remove_file(&path);
        return;
    }
    let entry = format!(
        "[Desktop Entry]\nType=Application\nName={}\nComment={}\nIcon={ICON}\nExec=/usr/libexec/cosmic-blue-parental-controls --more-time\nCategories=Utility;\nTerminal=false\n",
        tr("Ask for more time"),
        tr("A parent confirms with their password."),
    );
    if std::fs::read_to_string(&path).ok().as_deref() == Some(entry.as_str()) {
        return;
    }
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Err(error) = std::fs::write(&path, entry) {
        eprintln!("{}: {error}", path.display());
    }
}

/// A notification with an "Ask for more time" button, which opens the
/// window where a parent gives it
fn notify(remaining: u64, end: &str) {
    let minutes = remaining.div_ceil(60);
    let title = if minutes <= 1 {
        tr("1 minute left").to_owned()
    } else {
        fill(tr("{} minutes left"), &minutes.to_string())
    };
    let body = fill(tr("Your session closes at {}. Save your work."), end);
    let action = format!("more={}", tr("Ask for more time"));
    // The notification shown, if any: the next warning replaces it in place
    // (no pile of warnings), and its notify-send, which waits for a click on
    // the old one, is stopped
    let (id, waiting) = {
        let shown = SHOWN.lock().unwrap_or_else(|e| e.into_inner());
        *shown
    };
    if let Some(pid) = waiting {
        let _ = std::process::Command::new("kill")
            .arg(pid.to_string())
            .status();
    }
    let replace = id.map(|id| id.to_string());
    std::thread::spawn(move || {
        use std::io::BufRead;
        let mut args = vec![
            "--app-name",
            tr("Parental controls"),
            "--icon",
            ICON,
            "--urgency",
            "critical",
            // Until the child closes it: without a time, the COSMIC
            // notifications take 3 seconds, even for a critical one
            "--expire-time",
            "0",
            "--action",
            &action,
            "--print-id",
            "--wait",
        ];
        if let Some(replace) = &replace {
            args.extend(["--replace-id", replace]);
        }
        args.extend([title.as_str(), body.as_str()]);
        let child = std::process::Command::new("notify-send")
            .args(&args)
            .stdout(std::process::Stdio::piped())
            .spawn();
        let mut child = match child {
            Ok(child) => child,
            Err(error) => {
                eprintln!("notify-send: {error}");
                return;
            }
        };
        let pid = child.id();
        // First line: the notification's id; then the action clicked
        let mut lines = std::io::BufReader::new(child.stdout.take().expect("piped")).lines();
        let id = lines
            .next()
            .and_then(Result::ok)
            .and_then(|line| line.trim().parse().ok());
        *SHOWN.lock().unwrap_or_else(|e| e.into_inner()) = (id, Some(pid));
        let clicked = lines.next().and_then(Result::ok).unwrap_or_default();
        let _ = child.wait();
        {
            let mut shown = SHOWN.lock().unwrap_or_else(|e| e.into_inner());
            if shown.1 == Some(pid) {
                shown.1 = None;
            }
        }
        if clicked.trim() == "more"
            && let Err(error) = std::process::Command::new("/proc/self/exe")
                .arg("--more-time")
                .spawn()
                .or_else(|_| {
                    std::process::Command::new("/usr/libexec/cosmic-blue-parental-controls")
                        .arg("--more-time")
                        .spawn()
                })
        {
            eprintln!("Could not open the more time window: {error}");
        }
    });
}

/// The warning notification shown: its id, and the notify-send waiting on it
static SHOWN: std::sync::Mutex<(Option<u32>, Option<u32>)> = std::sync::Mutex::new((None, None));

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct Account {
    user: String,
    /// The full name, or the user name
    name: String,
    admin: bool,
    rules: Option<Rules>,
    usage: Usage,
    extra: Vec<parental_rules::Grant>,
    avatar: Option<widget::image::Handle>,
}

/// The people's accounts (user ids 1000 and up, with a login shell):
/// administrators (group wheel) first, then the others
fn accounts() -> Vec<Account> {
    let getent = |args: &[&str]| {
        std::process::Command::new("getent")
            .args(args)
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
            .unwrap_or_default()
    };
    let wheel: BTreeSet<String> = getent(&["group", "wheel"])
        .split(':')
        .nth(3)
        .unwrap_or_default()
        .trim()
        .split(',')
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect();
    let mut accounts: Vec<Account> = getent(&["passwd"])
        .lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split(':').collect();
            let (user, uid, gecos, shell) = (
                fields.first()?,
                fields.get(2)?,
                fields.get(4)?,
                fields.get(6)?,
            );
            let uid: u32 = uid.parse().ok()?;
            if !(1000..60000).contains(&uid)
                || shell.ends_with("nologin")
                || shell.ends_with("false")
            {
                return None;
            }
            let name = gecos.split(',').next().unwrap_or_default().trim();
            let avatar = Path::new(AVATARS).join(user);
            Some(Account {
                user: (*user).to_owned(),
                name: if name.is_empty() {
                    (*user).to_owned()
                } else {
                    name.to_owned()
                },
                admin: wheel.contains(*user),
                rules: Rules::load(user),
                usage: Usage::load(user),
                extra: parental_rules::load_extra(user),
                // No picture of their own: COSMIC's default one (our
                // planet), as on the login screen
                avatar: style::rounded_picture(&avatar, 128)
                    .or_else(|| style::rounded_picture(&Path::new(FACES).join(DEFAULT_FACE), 128)),
            })
        })
        .collect();
    accounts.sort_by_key(|a| a.name.to_lowercase());
    accounts
}

/// A user name from a person's name: "Léa Martin" -> "lea-martin"
fn user_from_name(name: &str, taken: &[String]) -> String {
    let mut base = String::new();
    for c in name.to_lowercase().chars() {
        let c = match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 'a',
            'ç' => 'c',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ñ' => 'n',
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ý' | 'ÿ' => 'y',
            'ß' => 's',
            c if c.is_ascii_alphanumeric() => c,
            _ => '-',
        };
        if c != '-' || !base.ends_with('-') {
            base.push(c);
        }
    }
    let mut base = base.trim_matches('-').to_owned();
    base.truncate(24);
    if !base.starts_with(|c: char| c.is_ascii_lowercase()) {
        base = format!("child{base}");
    }
    let base = base.trim_end_matches('-').to_owned();
    let mut user = base.clone();
    let mut n = 2;
    while taken.contains(&user)
        || std::process::Command::new("getent")
            .args(["passwd", &user])
            .status()
            .is_ok_and(|status| status.success())
    {
        user = format!("{base}{n}");
        n += 1;
    }
    user
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
    const APP_ID: &'static str = "io.github.cosmic_blue_os.ParentalControls";

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

/// The page around a content: its header (a picture, the title and a
/// caption), then the content; it scrolls only when the window is made
/// smaller than its default size. "Back" is in the header bar.
fn frame<'a>(
    picture: Element<'a, Message>,
    title: String,
    caption: Option<String>,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    let cosmic_theme::Spacing {
        space_xxs,
        space_m,
        space_l,
        ..
    } = theme::spacing();
    widget::column::with_capacity(5)
        .push(widget::space::vertical().height(space_xxs))
        .push(style::page_header(picture, title, caption))
        .push(widget::space::vertical().height(space_m))
        .push(content)
        .push(widget::space::vertical().height(space_l))
        .max_width(640.0)
        .width(640.0)
        .apply(widget::container)
        .center_x(Length::Fill)
        .apply(widget::scrollable)
        .into()
}

/// The app's icon, at the size of a page header's picture
fn app_icon<'a>() -> Element<'a, Message> {
    widget::icon::from_name(ICON).size(44).into()
}

/// cosmic-blue's profile pictures, to pick one for a new child account, cut
/// as rounded squares (the links, such as COSMIC's default picture, are not listed
/// twice)
fn faces() -> Vec<(PathBuf, widget::image::Handle)> {
    let mut faces: Vec<PathBuf> = std::fs::read_dir(FACES)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "png"))
        .collect();
    faces.sort();
    faces
        .into_iter()
        .filter_map(|face| {
            let handle = style::rounded_picture(&face, 72)?;
            Some((face, handle))
        })
        .collect()
}

/// A row's text: a first line, and a caption under it
fn two_lines<'a>(first: impl Into<String>, caption: String) -> Element<'a, Message> {
    widget::column::with_capacity(2)
        .spacing(2)
        .push(style::row_title(first.into()))
        .push(style::dim_caption(caption).wrapping(Wrapping::Word))
        .width(Length::Fill)
        .into()
}

fn avatar<'a>(account: &Account, size: u16) -> Element<'a, Message> {
    match &account.avatar {
        Some(handle) => widget::image(handle.clone())
            .width(size)
            .height(size)
            .into(),
        None => widget::icon::from_name("avatar-default-symbolic")
            .size(size)
            .into(),
    }
}

/// A rounded bar in the accent color (or a faint one)
fn bar<'a>(width: Length, height: f32, accent: bool) -> Element<'a, Message> {
    widget::container(widget::space::horizontal().width(Length::Fixed(0.0)))
        .width(width)
        .height(Length::Fixed(height))
        .class(theme::Container::custom(move |theme| {
            let cosmic = theme.cosmic();
            let color = if accent {
                cosmic.accent_color()
            } else {
                let mut color = cosmic.on_bg_color();
                color.alpha = 0.15;
                color
            };
            cosmic::iced::widget::container::Style {
                background: Some(cosmic::iced::Color::from(color).into()),
                border: cosmic::iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }))
        .into()
}

/// Today's screen time of an account, as a caption and a gauge (0 to 1)
fn today(account: &Account) -> (String, Option<f32>) {
    let Some(rules) = &account.rules else {
        return (tr("No limits").to_owned(), None);
    };
    let Some(now) = Now::read() else {
        return (String::new(), None);
    };
    let used = account.usage.used_today(&now);
    let verdict = parental_rules::verdict(rules, used, &account.extra, &now);
    match verdict.allowed_today {
        Some(allowed) => (
            tr("Today: {} of {}")
                .replacen("{}", &duration(used), 1)
                .replacen("{}", &duration(allowed), 1),
            Some(if allowed == 0 {
                1.0
            } else {
                (used as f32 / allowed as f32).min(1.0)
            }),
        ),
        None if rules.bedtime_enabled || rules.web_filter => {
            (fill(tr("Today: {}"), &duration(used)), None)
        }
        None => (tr("No limits").to_owned(), None),
    }
}

impl App {
    fn home(&self) -> Element<'_, Message> {
        let space_s = theme::spacing().space_s;
        let space_m = theme::spacing().space_m;
        let mut content = widget::column::with_capacity(4).spacing(space_m);

        // A child's account: no administrator rights, and rules (a standard
        // account without rules may be an adult's: it is in "Other accounts")
        let children: Vec<&Account> = self
            .accounts
            .iter()
            .filter(|a| !a.admin && a.rules.is_some())
            .collect();
        let mut section = settings::section().title(tr("Child accounts"));
        for account in children {
            let (caption, gauge) = today(account);
            let mut lines = widget::column::with_capacity(3)
                .spacing(2)
                .push(style::row_title(account.name.clone()))
                .push(style::dim_caption(caption))
                .width(Length::Fill);
            if let Some(gauge) = gauge {
                lines = lines.push(widget::space::vertical().height(4)).push(
                    widget::progress_bar::determinate_linear(gauge)
                        .width(Length::Fill)
                        .girth(6),
                );
            }
            section = section.add(
                widget::button::custom(
                    settings::item_row(vec![
                        avatar(account, 40),
                        lines.into(),
                        widget::icon::from_name("go-next-symbolic").size(16).into(),
                    ])
                    .spacing(space_s),
                )
                .class(theme::Button::MenuItem)
                .padding(0)
                .width(Length::Fill)
                .on_press(Message::Open(account.user.clone())),
            );
        }
        section = section.add(
            settings::item_row(vec![
                two_lines(
                    tr("Add a child account"),
                    tr("An account without administrator rights, ready in one click").to_owned(),
                ),
                widget::button::suggested(tr("Add…"))
                    .on_press(Message::ShowAdd)
                    .into(),
            ])
            .spacing(space_s),
        );
        content = content.push(section);

        let others: Vec<&Account> = self
            .accounts
            .iter()
            .filter(|a| a.admin || a.rules.is_none())
            .collect();
        if !others.is_empty() {
            let mut section = settings::section().title(tr("Other accounts"));
            for account in others {
                let mut row = vec![
                    avatar(account, 40),
                    two_lines(
                        account.name.clone(),
                        tr(if account.admin {
                            "Administrator · no limits"
                        } else {
                            "Standard account · no limits"
                        })
                        .to_owned(),
                    ),
                ];
                if !account.admin {
                    // Its page: the rules are saved at the first change
                    row.push(
                        widget::button::standard(tr("This is a child"))
                            .on_press(Message::Open(account.user.clone()))
                            .into(),
                    );
                }
                section = section.add(settings::item_row(row).spacing(space_s));
            }
            content = content.push(section);
        }
        content = content.push(
            style::dim_caption(tr(
                "Only a parent (an administrator account) can change these settings.",
            ))
            .align_x(Alignment::Center)
            .width(Length::Fill),
        );
        frame(
            app_icon(),
            tr("Parental controls").to_owned(),
            Some(tr("Choose how long and when your children can use this computer.").to_owned()),
            content.into(),
        )
    }

    fn child(&self, user: &str) -> Element<'_, Message> {
        let space_s = theme::spacing().space_s;
        let space_m = theme::spacing().space_m;
        let Some(account) = self.account(user) else {
            return frame(
                app_icon(),
                user.to_owned(),
                None,
                widget::column::with_capacity(0).into(),
            );
        };
        let rules = account.rules.clone().unwrap_or_else(Rules::none);
        let toggle = |on: bool, change: fn(&mut Rules, bool)| {
            let rules = rules.clone();
            widget::toggler(on).on_toggle(move |value| {
                let mut rules = rules.clone();
                change(&mut rules, value);
                Message::Rules(rules)
            })
        };
        // - and + by 15 minutes, and the value itself can be typed in
        let minutes_button = |field: Field| -> Element<'_, Message> {
            let shown = match &self.editing {
                Some((editing, typed)) if *editing == field => typed.clone(),
                _ => field.show(field.get(&rules)),
            };
            widget::row::with_capacity(3)
                .push(
                    widget::button::icon(widget::icon::from_name("list-remove-symbolic"))
                        .on_press(Message::Step(field, false)),
                )
                .push(
                    // A visible field, so that it shows it can be typed in
                    widget::text_input("", shown)
                        .select_on_focus(true)
                        .on_input(move |typed| Message::Edit(field, typed))
                        .on_submit(|_| Message::Commit)
                        .on_unfocus(Message::Commit)
                        .width(Length::Fixed(96.0)),
                )
                .push(
                    widget::button::icon(widget::icon::from_name("list-add-symbolic"))
                        .on_press(Message::Step(field, true)),
                )
                .align_y(Alignment::Center)
                .into()
        };

        let mut screen = settings::section().title(tr("Screen time")).add(
            settings::item_row(vec![
                two_lines(
                    tr("Limit screen time"),
                    tr("Only time in front of the screen counts. Warnings 10, 5 and 1 minutes before the end.").to_owned(),
                ),
                toggle(rules.limit_enabled, |r, v| r.limit_enabled = v).into(),
            ])
            .spacing(space_s),
        );
        if rules.limit_enabled {
            screen = screen
                .add(settings::item(
                    tr("Monday to Friday"),
                    minutes_button(Field::Weekdays),
                ))
                .add(settings::item(
                    tr("Saturday and Sunday"),
                    minutes_button(Field::Weekend),
                ));
        }

        let mut bedtime = settings::section().title(tr("Bedtime")).add(
            settings::item_row(vec![
                two_lines(
                    tr("No computer at night"),
                    tr("The session closes and logging in is refused during these hours.")
                        .to_owned(),
                ),
                toggle(rules.bedtime_enabled, |r, v| r.bedtime_enabled = v).into(),
            ])
            .spacing(space_s),
        );
        if rules.bedtime_enabled {
            bedtime = bedtime
                .add(settings::item(
                    tr("From"),
                    minutes_button(Field::BedtimeStart),
                ))
                .add(settings::item(tr("To"), minutes_button(Field::BedtimeEnd)));
        }

        let internet = settings::section().title(tr("Internet")).add(
            settings::item_row(vec![
                widget::column::with_capacity(2)
                    .spacing(2)
                    .push(
                        widget::row::with_capacity(2)
                            .spacing(space_s)
                            .align_y(Alignment::Center)
                            .push(style::row_title(tr("Block unsuitable websites")))
                            .push(
                                text::caption(tr("Recommended"))
                                    .apply(widget::container)
                                    .padding([1, 8])
                                    .class(theme::Container::custom(|theme| {
                                        let cosmic = theme.cosmic();
                                        cosmic::iced::widget::container::Style {
                                            background: Some(cosmic::iced::Color::from(cosmic.accent_color()).into()),
                                            text_color: Some(cosmic::iced::Color::from(cosmic.accent.on)),
                                            border: cosmic::iced::Border {
                                                radius: 10.0.into(),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        }
                                    })),
                            ),
                    )
                    .push(
                        style::dim_caption(tr("Adult content, gambling and malicious sites, in every browser. Safe search is forced. Uses the CleanBrowsing family filter."))
                            .wrapping(Wrapping::Word),
                    )
                    .width(Length::Fill)
                    .into(),
                toggle(rules.web_filter, |r, v| r.web_filter = v).into(),
            ])
            .spacing(space_s),
        );

        // One tab at a time, so that the page fits without scrolling
        let tabs = widget::segmented_control::horizontal(&self.tabs)
            .on_activate(Message::Tab)
            .width(Length::Fill);
        let mut content = widget::column::with_capacity(3).spacing(space_m).push(tabs);
        content = match self.tabs.active_data::<Tab>().copied().unwrap_or(Tab::Time) {
            Tab::Time => content.push(screen).push(bedtime),
            Tab::Apps => content.push(self.apps_section(account, &rules)),
            Tab::Internet => content.push(internet),
            Tab::Week => content.push(self.week(account, &rules)),
        };
        let (caption, _) = today(account);
        frame(
            avatar(account, 52),
            account.name.clone(),
            Some(caption),
            content.into(),
        )
    }

    /// The "Applications" section of a child's page
    fn apps_section<'a>(&'a self, account: &Account, rules: &Rules) -> Element<'a, Message> {
        let space_s = theme::spacing().space_s;
        let mut section = settings::section().title(tr("Applications")).add(
            settings::item_row(vec![
                two_lines(
                    tr("Applications validated by a parent"),
                    fill(
                        tr("{} only uses the applications you ticked. Installing an application needs a parent's password."),
                        &account.name,
                    ),
                ),
                widget::toggler(rules.apps_enabled)
                    .on_toggle(Message::AppsEnabled)
                    .into(),
            ])
            .spacing(space_s),
        );
        if rules.apps_enabled {
            let caption = match &self.apps {
                Some(apps) => tr("{} of {}")
                    .replacen(
                        "{}",
                        &apps
                            .iter()
                            .filter(|a| rules.allowed_apps.contains(&a.id))
                            .count()
                            .to_string(),
                        1,
                    )
                    .replacen("{}", &apps.len().to_string(), 1),
                None => String::new(),
            };
            section = section
                .add(
                    settings::item_row(vec![
                        two_lines(tr("Allowed applications"), caption),
                        widget::button::standard(tr("Choose…"))
                            .on_press(Message::OpenApps(account.user.clone()))
                            .into(),
                    ])
                    .spacing(space_s),
                )
                .add(settings::item_row(vec![
                    style::dim_caption(tr("Downloaded programs (AppImages, executable files) do not run on this account."))
                        .wrapping(Wrapping::Word)
                        .width(Length::Fill)
                        .into(),
                ]));
        }
        section.into()
    }

    /// The applications a child may use, to tick
    fn apps_page(&self, user: &str) -> Element<'_, Message> {
        let space_s = theme::spacing().space_s;
        let space_m = theme::spacing().space_m;
        let Some(account) = self.account(user) else {
            return frame(
                app_icon(),
                user.to_owned(),
                None,
                widget::column::with_capacity(0).into(),
            );
        };
        let rules = account.rules.clone().unwrap_or_else(Rules::none);
        let title = fill(tr("Applications of {}"), &account.name);
        let mut content = widget::column::with_capacity(4).spacing(space_m).push(
            widget::search_input(tr("Search for an application"), self.search.clone())
                .on_input(Message::Search)
                .on_clear(Message::Search(String::new()))
                .width(Length::Fill),
        );
        if self.apps.is_none() {
            return frame(avatar(account, 52), title, None, content.into());
        }
        let found = self.found_apps();
        // How many are allowed, or found; and both buttons for those
        let allowed_found = found
            .iter()
            .filter(|app| rules.allowed_apps.contains(&app.id))
            .count();
        let count = if self.search.trim().is_empty() {
            tr("{allowed} allowed out of {total}")
                .replace("{allowed}", &allowed_found.to_string())
                .replace("{total}", &found.len().to_string())
        } else if found.len() == 1 {
            tr("One application found").to_owned()
        } else {
            fill(tr("{} applications found"), &found.len().to_string())
        };
        content = content.push(
            widget::row::with_capacity(3)
                .spacing(space_s)
                .align_y(Alignment::Center)
                .push(
                    style::dim_caption(count)
                        .wrapping(Wrapping::Word)
                        .width(Length::Fill),
                )
                .push(widget::button::text(tr("Allow all")).on_press_maybe(
                    (allowed_found < found.len()).then_some(Message::AllowAll(true)),
                ))
                .push(
                    widget::button::text(tr("Block all"))
                        .on_press_maybe((allowed_found > 0).then_some(Message::AllowAll(false))),
                ),
        );
        for (heading, allowed) in [(tr("Allowed"), true), (tr("Blocked"), false)] {
            let mut section = settings::section().title(heading);
            let mut any = false;
            for app in found
                .iter()
                .filter(|a| rules.allowed_apps.contains(&a.id) == allowed)
            {
                any = true;
                let icon = if app.icon.starts_with('/') {
                    widget::icon::from_path(app.icon.clone().into())
                } else {
                    widget::icon::from_name(app.icon.clone()).size(32).handle()
                };
                let id = app.id.clone();
                section = section.add(
                    settings::item_row(vec![
                        widget::icon(icon).size(32).into(),
                        two_lines(app.name.clone(), app.comment.clone()),
                        widget::checkbox(allowed)
                            .on_toggle(move |on| Message::Allow(id.clone(), on))
                            .into(),
                    ])
                    .spacing(space_s),
                );
            }
            if any {
                content = content.push(section);
            }
        }
        content = content.push(
            text::caption(tr(
                "Applications installed later are blocked until you tick them.",
            ))
            .align_x(Alignment::Center)
            .width(Length::Fill),
        );
        frame(avatar(account, 52), title, None, content.into())
    }

    /// The screen time of the last seven days, as bars (always seven: a
    /// day without use has a flat bar), then the most used applications
    fn week<'a>(&'a self, account: &Account, rules: &Rules) -> Element<'a, Message> {
        const DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        const HEIGHT: f32 = 60.0;
        /// Room above a bar for its value
        const VALUE: f32 = 16.0;
        let space_s = theme::spacing().space_s;
        let Some(now) = Now::read() else {
            return widget::column::with_capacity(0).into();
        };
        let Some(today) = parental_rules::day_number(&now.date) else {
            return widget::column::with_capacity(0).into();
        };
        let mut known = account.usage.history.clone();
        if account.usage.today.date == now.date {
            known.push(account.usage.today.clone());
        }
        // The seven days that end today, oldest first
        let days: Vec<(u32, Option<&parental_rules::Day>)> = (0..7)
            .rev()
            .map(|back: i64| {
                let weekday = ((i64::from(now.weekday) - 1 - back).rem_euclid(7) + 1) as u32;
                let day = known
                    .iter()
                    .find(|d| parental_rules::day_number(&d.date) == Some(today - back));
                (weekday, day)
            })
            .collect();
        let seconds = |day: &Option<&parental_rules::Day>| day.map_or(0, |d| d.seconds);
        let most = days.iter().map(|(_, d)| seconds(d)).max().unwrap_or(0);
        // The limit line, when every day has the same one and a day came
        // near it (otherwise the bars would be tiny under it)
        let limit = rules
            .daily_limit(1)
            .filter(|_| rules.daily_limit(1) == rules.daily_limit(7))
            .map(|m| u64::from(m) * 60)
            .filter(|limit| most * 4 >= limit * 3);
        let highest = most.max(limit.unwrap_or(0)).max(60);
        let height_of = |secs: u64| secs as f32 / highest as f32 * HEIGHT;
        let mut bars = widget::row::with_capacity(7)
            .spacing(space_s)
            .align_y(Alignment::End)
            .height(Length::Fixed(HEIGHT + VALUE));
        let mut labels = widget::row::with_capacity(7).spacing(space_s);
        for (i, (weekday, day)) in days.iter().enumerate() {
            let used = seconds(day);
            let is_today = i == days.len() - 1;
            bars = bars.push(
                widget::column::with_capacity(2)
                    .spacing(2)
                    .align_x(Alignment::Center)
                    .width(Length::Fill)
                    .push_maybe((used >= 60).then(|| text::caption_heading(duration(used))))
                    .push(bar(Length::Fixed(26.0), height_of(used).max(3.0), used > 0)),
            );
            let label = if is_today {
                tr("Today").to_owned()
            } else {
                DAYS.get(weekday.saturating_sub(1) as usize)
                    .map_or(String::new(), |d| tr(d).to_owned())
            };
            labels = labels.push(
                widget::container(if is_today {
                    text::caption_heading(label)
                } else {
                    text::caption(label)
                })
                .center_x(Length::Fill),
            );
        }
        // The limit as a thin line across the bars, with its value
        let chart: Element<'a, Message> = match limit {
            Some(limit) => cosmic::iced::widget::stack![
                bars,
                widget::column::with_capacity(2)
                    .push(widget::space::vertical().height(Length::Fixed(
                        (HEIGHT + VALUE - height_of(limit) - 14.0).max(0.0)
                    )))
                    .push(
                        widget::container(text::caption(duration(limit)))
                            .align_x(Alignment::End)
                            .width(Length::Fill)
                            .height(Length::Fixed(14.0)),
                    )
                    .push(
                        widget::container(widget::space::horizontal())
                            .width(Length::Fill)
                            .height(Length::Fixed(1.0))
                            .class(theme::Container::custom(|theme| {
                                let mut color = theme.cosmic().on_bg_color();
                                color.alpha = 0.3;
                                cosmic::iced::widget::container::Style {
                                    background: Some(cosmic::iced::Color::from(color).into()),
                                    ..Default::default()
                                }
                            })),
                    ),
            ]
            .into(),
            None => bars.into(),
        };
        // The average of the days the computer was used
        let used: Vec<u64> = days
            .iter()
            .map(|(_, d)| seconds(d))
            .filter(|s| *s > 0)
            .collect();
        let average = used.iter().sum::<u64>() / used.len().max(1) as u64;
        let mut column = widget::column::with_capacity(2).push(
            // (the tab says "This week")
            settings::section().add(
                widget::column::with_capacity(3)
                    .spacing(theme::spacing().space_xxs)
                    .push(chart)
                    .push(labels)
                    .push(
                        widget::container(style::dim_caption(fill(
                            tr("Average: {} a day"),
                            &duration(average),
                        )))
                        .center_x(Length::Fill),
                    )
                    .padding([theme::spacing().space_xxs, space_s]),
            ),
        );
        // The applications in front the longest over the seven days
        let mut apps: std::collections::BTreeMap<&str, u64> = std::collections::BTreeMap::new();
        for (_, day) in &days {
            for (id, secs) in day.iter().flat_map(|d| d.apps.iter()) {
                *apps.entry(id.as_str()).or_default() += secs;
            }
        }
        let mut apps: Vec<(&str, u64)> = apps.into_iter().filter(|(_, s)| *s >= 60).collect();
        apps.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        apps.truncate(5);
        if let Some(&(_, most)) = apps.first() {
            let mut section = settings::section().title(tr("Most used applications"));
            for (id, secs) in apps {
                let app = self.apps.iter().flatten().find(|app| app.matches(id));
                let icon = match app.map(|app| app.icon.as_str()) {
                    Some(icon) if icon.starts_with('/') => widget::icon::from_path(icon.into()),
                    Some(icon) => widget::icon::from_name(icon).size(24).handle(),
                    None => widget::icon::from_name("application-x-executable")
                        .size(24)
                        .handle(),
                };
                // Not among the applications: the last part of its id
                // (org.kde.krita: Krita)
                let name = app.map_or_else(|| pretty_id(id), |app| app.name.clone());
                section = section.add(
                    settings::item_row(vec![
                        widget::icon(icon).size(24).into(),
                        widget::column::with_capacity(2)
                            .spacing(6)
                            .push(style::row_title(name))
                            .push(
                                widget::progress_bar::determinate_linear(secs as f32 / most as f32)
                                    .girth(6)
                                    .width(Length::Fill),
                            )
                            .width(Length::Fill)
                            .into(),
                        // The same width for every time, so that the bars
                        // line up
                        text::body(duration(secs))
                            .align_x(Alignment::End)
                            .width(Length::Fixed(72.0))
                            .into(),
                    ])
                    .spacing(space_s),
                );
            }
            column = column.push(section);
        }
        column.spacing(space_s).into()
    }

    fn add(&self) -> Element<'_, Message> {
        let space_s = theme::spacing().space_s;
        let space_m = theme::spacing().space_m;
        let ready = !self.name.trim().is_empty() && !self.password.is_empty() && !self.busy;
        let create: Element<'_, Message> = if self.busy {
            widget::progress_bar::indeterminate_circular().into()
        } else {
            widget::button::suggested(tr("Create"))
                .on_press_maybe(ready.then_some(Message::Create))
                .into()
        };
        // The pictures, in two rows, the one picked outlined
        let mut pictures = widget::column::with_capacity(2).spacing(8);
        for chunk in self.faces.chunks(self.faces.len().div_ceil(2).max(1)) {
            let mut row = widget::row::with_capacity(chunk.len()).spacing(8);
            for (face, handle) in chunk {
                row = row.push(style::picture_button(
                    handle.clone(),
                    36,
                    self.face.as_ref() == Some(face),
                    Message::Face(face.clone()),
                ));
            }
            pictures = pictures.push(row);
        }
        let content = widget::column::with_capacity(4)
            .spacing(space_m)
            .push(
                settings::section()
                    .add(settings::item(
                        tr("Name"),
                        widget::text_input(tr("Name"), &self.name)
                            .on_input(Message::Name)
                            .width(Length::Fixed(280.0)),
                    ))
                    .add(settings::item(
                        tr("Password"),
                        widget::secure_input(
                            tr("Password"),
                            &self.password,
                            Some(Message::TogglePassword),
                            self.password_hidden,
                        )
                        .on_input(Message::Password)
                        .on_submit(|_| Message::Create)
                        .width(Length::Fixed(280.0)),
                    ))
                    .add_maybe((!self.faces.is_empty()).then(|| {
                        widget::column::with_capacity(2)
                            .spacing(space_s)
                            .push(text::body(tr("Picture")))
                            .push(pictures)
                            .padding([space_s, 0])
                    })),
            )
            .push(
                text::caption(tr("The child logs in with this name and password."))
                    .align_x(Alignment::Center)
                    .width(Length::Fill),
            )
            .push(
                widget::row::with_capacity(2)
                    .spacing(space_s)
                    .push(widget::space::horizontal().width(Length::Fill))
                    .push(widget::button::standard(tr("Cancel")).on_press(Message::Back))
                    .push(create),
            );
        let picked = self
            .faces
            .iter()
            .find(|(face, _)| self.face.as_ref() == Some(face));
        let picture: Element<'_, Message> = match picked {
            Some((_, handle)) => widget::image(handle.clone()).width(44).height(44).into(),
            None => app_icon(),
        };
        frame(
            picture,
            tr("Add a child account").to_owned(),
            None,
            content.into(),
        )
    }

    /// The child's window: a parent gives more time with their password
    fn more_time(&self) -> Element<'_, Message> {
        let cosmic_theme::Spacing {
            space_xxs,
            space_s,
            space_m,
            ..
        } = theme::spacing();
        let user = user_name();
        let account = self.account(&user);
        let name = account.map_or(user.clone(), |account| account.name.clone());
        let picture: Element<'_, Message> = match account {
            Some(account) => avatar(account, 64),
            None => widget::icon::from_name(ICON).size(64).into(),
        };
        let on = !self.no_limit && !self.busy;
        // The amount: - and +, a field to type it in, then ready-made ones
        let shown = self
            .extra_typed
            .clone()
            .unwrap_or_else(|| duration(u64::from(self.extra) * 60));
        let step = |icon: &'static str, up: bool| {
            widget::button::icon(widget::icon::from_name(icon))
                .on_press_maybe(on.then_some(Message::ExtraStep(up)))
        };
        let mut field = widget::text_input("", shown)
            .select_on_focus(true)
            .width(Length::Fixed(96.0));
        if on {
            field = field
                .on_input(Message::ExtraEdit)
                .on_submit(|_| Message::ExtraCommit)
                .on_unfocus(Message::ExtraCommit);
        }
        let amount = settings::item_row(vec![
            style::row_title(tr("Extra time"))
                .width(Length::Fill)
                .into(),
            widget::row::with_capacity(3)
                .push(step("list-remove-symbolic", false))
                .push(field)
                .push(step("list-add-symbolic", true))
                .align_y(Alignment::Center)
                .into(),
        ])
        .align_y(Alignment::Center);
        let choices = EXTRA_CHOICES
            .into_iter()
            .fold(
                widget::row::with_capacity(EXTRA_CHOICES.len()).spacing(space_xxs),
                |row, minutes| {
                    let label = duration(u64::from(minutes) * 60);
                    let picked = self.extra_typed.is_none() && self.extra == minutes;
                    // Same width each, the text centered
                    let class = if picked {
                        theme::Button::Suggested
                    } else {
                        theme::Button::Standard
                    };
                    row.push(
                        widget::button::custom(text::body(label).center().width(Length::Fill))
                            .padding([5, 8])
                            .width(Length::Fill)
                            .class(class)
                            .on_press_maybe(on.then_some(Message::Extra(minutes))),
                    )
                },
            )
            .width(Length::Fill);
        let no_limit = settings::item_row(vec![
            two_lines(
                tr("No limit today"),
                tr("Bedtime included, until tomorrow morning").to_owned(),
            ),
            widget::toggler(self.no_limit)
                .on_toggle_maybe((!self.busy).then_some(Message::NoLimit))
                .into(),
        ])
        .spacing(space_s);
        // Only with bedtime hours: whether the more time goes past them
        let bedtime = self
            .child
            .as_ref()
            .map(|child| &child.rules)
            .filter(|rules| rules.bedtime_enabled && rules.bedtime_start != rules.bedtime_end)
            .map(|rules| {
                let start = parental_rules::clock(rules.bedtime_start);
                let caption = if self.keep_bedtime {
                    fill(tr("The session closes at {} all the same."), &start)
                } else {
                    fill(tr("Bedtime ({}) does not apply to this time."), &start)
                };
                settings::item_row(vec![
                    two_lines(tr("Keep to bedtime"), caption),
                    widget::toggler(self.keep_bedtime)
                        .on_toggle_maybe(on.then_some(Message::KeepBedtime))
                        .into(),
                ])
                .spacing(space_s)
            });
        let section = settings::section()
            .add(amount)
            .add(choices)
            .add_maybe(bedtime)
            .add(no_limit);

        // Now, and once given
        let (status, after) = self
            .child
            .as_ref()
            .map(|child| child.texts(self.no_limit, self.extra, self.keep_bedtime))
            .unwrap_or_default();

        let give: Element<'_, Message> = if self.busy {
            widget::progress_bar::indeterminate_circular().into()
        } else {
            let label = if self.no_limit {
                tr("Remove the limit").to_owned()
            } else {
                fill(tr("Give {}"), &duration(u64::from(self.extra) * 60))
            };
            widget::button::suggested(label)
                .on_press(Message::Give)
                .into()
        };
        let centered = |line: String| style::dim_caption(line).center().width(Length::Fill);
        let mut column = widget::column::with_capacity(8)
            .spacing(space_s)
            .padding([0, space_m, space_m, space_m])
            .align_x(Alignment::Center)
            .push(picture)
            .push(
                widget::column::with_capacity(2)
                    .spacing(space_xxs)
                    .push(
                        text::title3(fill(tr("More time for {}"), &name))
                            .center()
                            .width(Length::Fill),
                    )
                    .push_maybe(status.map(centered)),
            )
            .push(section);
        if let Some(after) = after {
            column = column.push(text::body(after).center().width(Length::Fill));
        }
        column
            .push(centered(
                tr("A parent confirms with their password.").to_owned(),
            ))
            .push(
                widget::row::with_capacity(3)
                    .spacing(space_s)
                    .push(widget::space::horizontal().width(Length::Fill))
                    .push(widget::button::standard(tr("Cancel")).on_press(Message::Close))
                    .push(give),
            )
            .max_width(420.0)
            .apply(widget::container)
            .center_x(Length::Fill)
            .into()
    }
}

/// A child's rules and time used, read in its session for the "more time"
/// window
#[derive(Clone, Debug)]
struct ChildTime {
    rules: Rules,
    used: u64,
    extra: Vec<parental_rules::Grant>,
    now: Now,
}

impl ChildTime {
    fn read() -> Option<ChildTime> {
        let user = user_name();
        let rules = Rules::load(&user)?;
        let now = Now::read()?;
        let used = Usage::load(&user).used_today(&now);
        Some(ChildTime {
            rules,
            used,
            extra: parental_rules::load_extra(&user),
            now,
        })
    }

    /// When the session closes, in minutes after midnight (None: not today)
    fn end(&self, extra: &[parental_rules::Grant]) -> Option<(u64, u32)> {
        let remaining = parental_rules::verdict(&self.rules, self.used, extra, &self.now)
            .remaining
            .filter(|remaining| *remaining > 0)?;
        let end = (self.now.minute + remaining.div_ceil(60) as u32) % (24 * 60);
        Some((remaining, end))
    }

    /// What the window says now, and once the time is given
    fn texts(
        &self,
        no_limit: bool,
        minutes: u32,
        keep_bedtime: bool,
    ) -> (Option<String>, Option<String>) {
        let before = self.end(&self.extra);
        let status = before
            .map(|(remaining, _)| fill(tr("The session closes in {}."), &duration(remaining)));
        let after = if no_limit {
            if self.rules.bedtime_enabled && self.rules.bedtime_start != self.rules.bedtime_end {
                fill(
                    tr("No limit until tomorrow at {}."),
                    &parental_rules::clock(self.rules.bedtime_end),
                )
            } else {
                tr("No limit for the rest of the day.").to_owned()
            }
        } else {
            let mut extra = self.extra.clone();
            extra.push(parental_rules::Grant::new(
                self.now.unix,
                minutes,
                !keep_bedtime,
            ));
            match (self.end(&extra), before) {
                (Some((_, new)), Some((_, old))) if new != old => {
                    tr("The session will close at {new} instead of {old}.")
                        .replace("{new}", &parental_rules::clock(new))
                        .replace("{old}", &parental_rules::clock(old))
                }
                (Some((_, new)), _) => fill(
                    tr("The session will close at {}."),
                    &parental_rules::clock(new),
                ),
                (None, _) => return (status, None),
            }
        };
        (status, Some(after))
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
