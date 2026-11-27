// Parental controls, in a child's session: the agent that warns before the
// session closes and tells the service which app is in front

use super::*;

/// The app id in front, for the service (parental_rules::FOCUS); none: the
/// file goes (a new file, renamed over the old one, so that the service
/// never reads half of it)
pub(super) fn write_focus(path: &Path, app: Option<&str>) {
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
pub(super) fn agent() {
    let user = user_name();
    // Administrators cannot have rules (see celenn-parental-helper)
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
pub(super) fn more_time_launcher(show: bool) {
    let Some(home) = std::env::var_os("HOME") else {
        return;
    };
    let path = Path::new(&home).join(format!(".local/share/applications/{MORE_TIME_ID}.desktop"));
    if !show {
        let _ = std::fs::remove_file(&path);
        return;
    }
    let entry = format!(
        "[Desktop Entry]\nType=Application\nName={}\nComment={}\nIcon={ICON}\nExec=/usr/libexec/celenn-parental-controls --more-time\nCategories=Utility;\nTerminal=false\n",
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
pub(super) fn notify(remaining: u64, end: &str) {
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
                    std::process::Command::new("/usr/libexec/celenn-parental-controls")
                        .arg("--more-time")
                        .spawn()
                })
        {
            eprintln!("Could not open the more time window: {error}");
        }
    });
}

/// The warning notification shown: its id, and the notify-send waiting on it
pub(super) static SHOWN: std::sync::Mutex<(Option<u32>, Option<u32>)> =
    std::sync::Mutex::new((None, None));
