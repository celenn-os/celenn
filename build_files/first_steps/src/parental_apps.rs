// Parental controls: the applications a parent can allow or block for a
// child account ("Applications validated by a parent"), shared by the app
// (the list to tick), the agent (hides blocked apps from the child's
// launcher) and the service (stops blocked apps and downloaded programs).
//
// An application is a launcher (.desktop file) of the system or of a system
// Flatpak, named by its id (the file name without ".desktop").

// (Each program uses its part)
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Where the launchers of installed applications are, in order of precedence
pub const APP_DIRS: &[&str] = &[
    "/var/lib/flatpak/exports/share/applications",
    "/usr/local/share/applications",
    "/usr/share/applications",
];

/// Launchers never listed: the parental controls themselves (the child asks
/// for more time with them)
const NEVER_LISTED: &[&str] = &[
    "io.github.cosmic_blue_os.ParentalControls",
    "io.github.cosmic_blue_os.ParentalControls.MoreTime",
    "io.github.cosmic_blue_os.ParentalControlsAgent",
];

/// Categories left unticked when a parent turns the option on (the rest of
/// the applications installed then are ticked)
const NOT_TICKED_FIRST: &[&str] = &["TerminalEmulator", "System", "Development", "Network"];
/// ... except these (a browser is in "Network")
const TICKED_FIRST: &[&str] = &["WebBrowser", "Office", "Education", "Graphics", "Game"];

/// Where a child must not run programs from: whatever it can write to
/// (downloads, AppImages, programs it copied or built)
pub const WRITABLE_DIRS: &[&str] = &[
    "/home/",
    "/var/home/",
    "/tmp/",
    "/var/tmp/",
    "/dev/shm/",
    "/run/user/",
    "/run/media/",
    "/media/",
    "/mnt/",
    "/var/mnt/",
];

#[derive(Clone, Debug, PartialEq)]
pub struct App {
    /// Launcher id: org.mozilla.firefox, com.system76.CosmicTerm...
    pub id: String,
    /// Name in the system's language
    pub name: String,
    /// Description, in the system's language (may be empty)
    pub comment: String,
    pub icon: String,
    /// The Flatpak app id, for a Flatpak
    pub flatpak: Option<String>,
    /// The program it starts (full path), for an app of the system
    pub program: Option<PathBuf>,
    pub categories: Vec<String>,
}

impl App {
    /// Ticked by default when a parent turns the option on
    pub fn ticked_first(&self) -> bool {
        let has = |list: &[&str]| self.categories.iter().any(|c| list.contains(&c.as_str()));
        has(TICKED_FIRST) || !has(NOT_TICKED_FIRST)
    }
}

/// The value of a key of the [Desktop Entry] group, in the language asked
/// (fr_CH, then fr, then no language)
fn entry_value(entries: &BTreeMap<String, String>, key: &str, lang: &str) -> Option<String> {
    let base = lang.split(['.', '@']).next().unwrap_or("");
    let short = base.split('_').next().unwrap_or("");
    [
        format!("{key}[{base}]"),
        format!("{key}[{short}]"),
        key.to_owned(),
    ]
    .iter()
    .filter(|k| !k.contains("[]"))
    .find_map(|k| entries.get(k).cloned())
}

/// The [Desktop Entry] group of a launcher
fn read_entry(path: &Path) -> Option<BTreeMap<String, String>> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut entries = BTreeMap::new();
    let mut in_entry = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_entry || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            entries
                .entry(key.trim().to_owned())
                .or_insert_with(|| value.trim().to_owned());
        }
    }
    Some(entries)
}

/// The program a launcher's Exec line starts, as a full path
fn program(exec: &str) -> Option<PathBuf> {
    let mut words = exec.split_whitespace().filter(|w| !w.contains('='));
    let mut first = words.next()?.trim_matches('"');
    if first.ends_with("/env") || first == "env" {
        first = words.next()?.trim_matches('"');
    }
    let path = if first.starts_with('/') {
        PathBuf::from(first)
    } else {
        ["/usr/bin", "/usr/sbin", "/usr/local/bin"]
            .iter()
            .map(|dir| Path::new(dir).join(first))
            .find(|p| p.exists())?
    };
    Some(std::fs::canonicalize(&path).unwrap_or(path))
}

/// The applications a parent can allow or block, by name
pub fn installed(lang: &str) -> Vec<App> {
    let dirs: Vec<PathBuf> = APP_DIRS.iter().map(PathBuf::from).collect();
    installed_in(&dirs, lang)
}

/// The applications of these launcher folders, by name (the first folder
/// with an id wins)
pub fn installed_in(dirs: &[PathBuf], lang: &str) -> Vec<App> {
    let mut apps: BTreeMap<String, App> = BTreeMap::new();
    for dir in dirs {
        let Ok(files) = std::fs::read_dir(dir) else {
            continue;
        };
        for file in files.flatten() {
            let path = file.path();
            let Some(id) = path
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|n| n.strip_suffix(".desktop"))
                .map(str::to_owned)
            else {
                continue;
            };
            if apps.contains_key(&id) || NEVER_LISTED.contains(&id.as_str()) {
                continue;
            }
            let Some(entries) = read_entry(&path) else {
                continue;
            };
            let yes = |key: &str| entries.get(key).is_some_and(|v| v == "true");
            let shown_in_cosmic = entries
                .get("OnlyShowIn")
                .is_none_or(|v| v.split(';').any(|d| d == "COSMIC"))
                && !entries
                    .get("NotShowIn")
                    .is_some_and(|v| v.split(';').any(|d| d == "COSMIC"));
            if entries.get("Type").map(String::as_str) != Some("Application")
                || yes("NoDisplay")
                || yes("Hidden")
                || !shown_in_cosmic
            {
                continue;
            }
            let Some(name) = entry_value(&entries, "Name", lang) else {
                continue;
            };
            let comment = entry_value(&entries, "Comment", lang)
                .or_else(|| entry_value(&entries, "GenericName", lang))
                .unwrap_or_default();
            let flatpak = entries.get("X-Flatpak").cloned();
            let program = match &flatpak {
                Some(_) => None,
                None => entries.get("Exec").and_then(|exec| program(exec)),
            };
            apps.insert(
                id.clone(),
                App {
                    id,
                    name,
                    comment,
                    icon: entries.get("Icon").cloned().unwrap_or_default(),
                    flatpak,
                    program,
                    categories: entries
                        .get("Categories")
                        .map(|c| {
                            c.split(';')
                                .filter(|c| !c.is_empty())
                                .map(str::to_owned)
                                .collect()
                        })
                        .unwrap_or_default(),
                },
            );
        }
    }
    let mut apps: Vec<App> = apps.into_values().collect();
    apps.sort_by_key(|app| app.name.to_lowercase());
    apps
}

/// What a child whose applications are validated may run
#[derive(Debug, Default)]
pub struct Policy {
    /// Flatpak app ids of the allowed applications (any other Flatpak is
    /// stopped, even one installed later or for the child alone)
    flatpaks: BTreeSet<String>,
    /// Programs of the system of the applications not allowed (a program
    /// also started by an allowed application stays allowed)
    programs: BTreeSet<PathBuf>,
    /// Application names, by Flatpak app id and by program (for the child's
    /// message)
    names: BTreeMap<String, String>,
}

/// Why a process of a child is stopped
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stop {
    /// An application not allowed: its name
    App(String),
    /// A downloaded program: its file name
    Downloaded(String),
}

impl Policy {
    pub fn new(apps: &[App], allowed: &BTreeSet<String>) -> Policy {
        let mut policy = Policy::default();
        let mut kept = BTreeSet::new();
        for app in apps {
            let ok = allowed.contains(&app.id);
            if let Some(id) = &app.flatpak {
                policy.names.insert(id.clone(), app.name.clone());
                if ok {
                    policy.flatpaks.insert(id.clone());
                }
            } else if let Some(program) = &app.program {
                policy
                    .names
                    .insert(program.to_string_lossy().into_owned(), app.name.clone());
                if ok {
                    kept.insert(program.clone());
                } else {
                    policy.programs.insert(program.clone());
                }
            }
        }
        // Never the programs every session needs, even if a launcher starts them
        for essential in ESSENTIAL {
            kept.insert(PathBuf::from(essential));
        }
        policy
            .programs
            .retain(|p| !kept.contains(p) && !ESSENTIAL_DIRS.iter().any(|d| p.starts_with(d)));
        policy
    }

    /// Whether a process (its program, its control group) must stop
    pub fn check(&self, exe: Option<&Path>, cgroup: &str) -> Option<Stop> {
        // A Flatpak's processes (games of Steam included, whatever their
        // files) go with the Flatpak
        if let Some(id) = flatpak_of(cgroup) {
            if self.flatpaks.contains(&id) {
                return None;
            }
            return Some(Stop::App(self.names.get(&id).cloned().unwrap_or(id)));
        }
        let exe = exe?;
        if is_downloaded(exe) {
            let name = exe
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            return Some(Stop::Downloaded(name));
        }
        if self.programs.contains(exe) {
            let key = exe.to_string_lossy().into_owned();
            return Some(Stop::App(self.names.get(&key).cloned().unwrap_or(key)));
        }
        None
    }
}

/// Programs never stopped: what every session needs, even if a launcher
/// starts them
const ESSENTIAL: &[&str] = &[
    "/usr/bin/bash",
    "/usr/bin/sh",
    "/usr/bin/env",
    "/usr/bin/flatpak",
    "/usr/bin/xdg-open",
    "/usr/bin/bwrap",
];
/// ... nor the session's own (COSMIC's settings, panel, files' helpers)
const ESSENTIAL_DIRS: &[&str] = &["/usr/libexec/"];

/// The processes of an account: id, program, control group
pub fn processes(uid: u32) -> Vec<(u32, Option<PathBuf>, String)> {
    use std::os::unix::fs::MetadataExt;
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let pid: u32 = entry.file_name().to_str()?.parse().ok()?;
            if entry.metadata().ok()?.uid() != uid {
                return None;
            }
            let exe = std::fs::read_link(entry.path().join("exe"))
                .ok()
                .map(|exe| {
                    // A program removed or replaced while running
                    let text = exe.to_string_lossy();
                    match text.strip_suffix(" (deleted)") {
                        Some(path) => PathBuf::from(path),
                        None => exe,
                    }
                });
            let cgroup = std::fs::read_to_string(entry.path().join("cgroup")).unwrap_or_default();
            Some((pid, exe, cgroup))
        })
        .collect()
}

/// The line that marks the launchers hiding an application from a child
const MARK: &str = "X-CosmicBlue-Parental=true";

/// Hides from a child's launcher the applications not allowed (None: all
/// shown), with launchers of the same id in its own applications
/// (~/.local/share/applications), which take precedence
pub fn hide(dir: &Path, apps: &[App], allowed: Option<&BTreeSet<String>>) {
    let hidden: BTreeSet<&str> = match allowed {
        Some(allowed) => apps
            .iter()
            .filter(|app| !allowed.contains(&app.id))
            .map(|app| app.id.as_str())
            .collect(),
        None => BTreeSet::new(),
    };
    // Ours no longer needed
    if let Ok(files) = std::fs::read_dir(dir) {
        for file in files.flatten() {
            let path = file.path();
            let Some(id) = path
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|n| n.strip_suffix(".desktop"))
            else {
                continue;
            };
            if !hidden.contains(id) && is_ours(&path) {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
    if hidden.is_empty() {
        return;
    }
    let _ = std::fs::create_dir_all(dir);
    for app in apps.iter().filter(|app| hidden.contains(app.id.as_str())) {
        let path = dir.join(format!("{}.desktop", app.id));
        // The child's own launcher of that id is left as it is (the
        // service stops the application anyway)
        if path.exists() {
            continue;
        }
        let entry = format!(
            "[Desktop Entry]\nType=Application\nName={}\nHidden=true\n{MARK}\n",
            app.name
        );
        if let Err(error) = std::fs::write(&path, entry) {
            eprintln!("{}: {error}", path.display());
        }
    }
}

fn is_ours(path: &Path) -> bool {
    std::fs::read_to_string(path).is_ok_and(|text| text.lines().any(|line| line.trim() == MARK))
}

/// True for a program a child must not run: one in a place it can write to
pub fn is_downloaded(program: &Path) -> bool {
    let path = program.to_string_lossy();
    WRITABLE_DIRS.iter().any(|dir| path.starts_with(dir))
}

/// The Flatpak app id of a process, from its control group
/// (".../app-flatpak-org.example.App-1234.scope")
pub fn flatpak_of(cgroup: &str) -> Option<String> {
    let scope = cgroup.lines().find_map(|line| {
        line.rsplit('/')
            .find(|part| part.starts_with("app-flatpak-") && part.ends_with(".scope"))
    })?;
    let rest = scope.strip_prefix("app-flatpak-")?.strip_suffix(".scope")?;
    let (id, number) = rest.rsplit_once('-')?;
    number
        .chars()
        .all(|c| c.is_ascii_digit())
        .then(|| id.replace("\\x2d", "-"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flatpak_scope() {
        assert_eq!(
            flatpak_of(
                "0::/user.slice/user-1001.slice/user@1001.service/app.slice/app-flatpak-com.discordapp.Discord-12345.scope\n"
            ),
            Some("com.discordapp.Discord".to_owned())
        );
        assert_eq!(
            flatpak_of(
                "0::/user.slice/user-1001.slice/user@1001.service/app.slice/app-flatpak-org.foo\\x2dbar.App-7.scope"
            ),
            Some("org.foo-bar.App".to_owned())
        );
        assert_eq!(
            flatpak_of("0::/user.slice/user-1001.slice/session-3.scope"),
            None
        );
    }

    fn app(id: &str, flatpak: Option<&str>, program: Option<&str>) -> App {
        App {
            id: id.to_owned(),
            name: id.to_uppercase(),
            comment: String::new(),
            icon: String::new(),
            flatpak: flatpak.map(str::to_owned),
            program: program.map(PathBuf::from),
            categories: Vec::new(),
        }
    }

    #[test]
    fn policy() {
        let apps = [
            app("firefox", Some("org.mozilla.firefox"), None),
            app("discord", Some("com.discordapp.Discord"), None),
            app("term", None, Some("/usr/bin/cosmic-term")),
            app("files", None, Some("/usr/bin/cosmic-files")),
            app("shell", None, Some("/usr/bin/bash")),
        ];
        let allowed: BTreeSet<String> =
            ["firefox", "files"].iter().map(|s| s.to_string()).collect();
        let policy = Policy::new(&apps, &allowed);
        let scope = |id: &str| {
            format!(
                "0::/user.slice/user-1001.slice/user@1001.service/app.slice/app-flatpak-{id}-42.scope\n"
            )
        };
        let session = "0::/user.slice/user-1001.slice/session-3.scope\n";
        assert_eq!(
            policy.check(
                Some(Path::new("/app/lib/firefox/firefox")),
                &scope("org.mozilla.firefox")
            ),
            None
        );
        assert_eq!(
            policy.check(
                Some(Path::new("/app/discord/Discord")),
                &scope("com.discordapp.Discord")
            ),
            Some(Stop::App("DISCORD".to_owned()))
        );
        // Not in the list (installed later, or for the child alone)
        assert_eq!(
            policy.check(None, &scope("org.torproject.torbrowser-launcher")),
            Some(Stop::App("org.torproject.torbrowser-launcher".to_owned()))
        );
        // A game of an allowed Flatpak, in the child's files
        assert_eq!(
            policy.check(
                Some(Path::new("/home/lea/.var/app/org.mozilla.firefox/x")),
                &scope("org.mozilla.firefox")
            ),
            None
        );
        assert_eq!(
            policy.check(Some(Path::new("/usr/bin/cosmic-term")), session),
            Some(Stop::App("TERM".to_owned()))
        );
        assert_eq!(
            policy.check(Some(Path::new("/usr/bin/cosmic-files")), session),
            None
        );
        assert_eq!(
            policy.check(Some(Path::new("/usr/bin/bash")), session),
            None
        );
        assert_eq!(
            policy.check(
                Some(Path::new(
                    "/var/home/lea/Downloads/tor-browser/Browser/firefox"
                )),
                session
            ),
            Some(Stop::Downloaded("firefox".to_owned()))
        );
        assert_eq!(policy.check(None, session), None);
    }

    #[test]
    fn hidden_launchers() {
        let dir = std::env::temp_dir().join(format!("parental-apps-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("mine.desktop"), "[Desktop Entry]\nName=Mine\n").unwrap();
        let apps = [
            app("a", None, None),
            app("b", None, None),
            app("mine", None, None),
        ];
        let allowed: BTreeSet<String> = ["a".to_owned()].into();
        hide(&dir, &apps, Some(&allowed));
        assert!(!dir.join("a.desktop").exists());
        assert!(is_ours(&dir.join("b.desktop")));
        assert!(!is_ours(&dir.join("mine.desktop")));
        hide(&dir, &apps, None);
        assert!(!dir.join("b.desktop").exists());
        assert!(dir.join("mine.desktop").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn downloaded() {
        assert!(is_downloaded(Path::new(
            "/var/home/lea/Downloads/tor-browser/firefox"
        )));
        assert!(is_downloaded(Path::new("/tmp/.mount_torbrXYZ/AppRun")));
        assert!(!is_downloaded(Path::new("/usr/bin/cosmic-term")));
        assert!(!is_downloaded(Path::new("/app/bin/discord")));
    }

    #[test]
    fn exec_program() {
        assert_eq!(
            program("sh -c 'echo'"),
            Some(std::fs::canonicalize("/bin/sh").unwrap_or("/usr/bin/sh".into()))
        );
        assert_eq!(
            program("/nonexistent/x %U"),
            Some(PathBuf::from("/nonexistent/x"))
        );
        assert_eq!(program("nonexistent-program-xyz"), None);
    }

    #[test]
    fn localized_value() {
        let mut entries = BTreeMap::new();
        entries.insert("Name".to_owned(), "Files".to_owned());
        entries.insert("Name[fr]".to_owned(), "Fichiers".to_owned());
        assert_eq!(
            entry_value(&entries, "Name", "fr_CH.UTF-8"),
            Some("Fichiers".to_owned())
        );
        assert_eq!(
            entry_value(&entries, "Name", "de_DE.UTF-8"),
            Some("Files".to_owned())
        );
    }
}
