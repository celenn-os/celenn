// "Other disks" on the Computer page of Celenn OS Tools: the internal
// data disks (a second disk, Windows' D:, a games disk) that can open at
// every startup, so Files, Steam and other apps always find them at the
// same place. Reading is done here, without root: lsblk, /etc/fstab. The
// change is made by /usr/libexec/celenn-tools-helper `disks UUID=on`
// through pkexec, which checks everything again and has the last word (an
// fstab line marked x-celenn-disk, at /var/mnt/<name>, nofail).
//
// Never offered: the system's own partitions, removable and USB disks,
// partitions that are not data (EFI, Microsoft reserved, recovery). Shown
// without a switch: encrypted disks (BitLocker, LUKS: they need a key),
// the partition Windows runs from (the helper refuses it; remembered in
// ~/.local/state/celenn/windows-disks), and disks set up by hand in
// /etc/fstab.

use crate::i18n::{fill, tr};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const HELPER: &str = "/usr/libexec/celenn-tools-helper";
/// The fstab option that marks the lines Tools manages
const MARK: &str = "x-celenn-disk";
const LSBLK_COLUMNS: &str = "NAME,TYPE,FSTYPE,LABEL,UUID,SIZE,RM,ROTA,TRAN,PARTTYPE,MOUNTPOINTS";
/// Partition types that are never data: EFI system, Microsoft reserved,
/// Windows recovery, BIOS boot (GPT); EFI and Windows recovery (MBR)
const NOT_DATA: &[&str] = &[
    "c12a7328-f81f-11d2-ba4b-00a0c93ec93b",
    "e3c9e316-0b5c-4db8-817d-f92df00215ae",
    "de94bba4-06d1-4d40-a16a-bfd50179d6ac",
    "21686148-6449-6e6f-744e-656564454649",
    "0xef",
    "0x27",
];

pub const TEXTS: &[&str] = &[
    "Other disks",
    "No other disk in this computer.",
    "{} opens at every startup.",
    "{} open at every startup.",
    "None opens at startup yet: choose the ones Files and Steam should always find.",
    "Choose…",
    "Turn on the ones you want open at every startup, without clicking them in Files.",
    "They open at every startup, even before you log in, and always at the same place: Files, Steam and your apps find them. USB drives are not concerned.",
    "Back",
    "{} disk",
    "{} GB",
    "{} TB",
    "SSD",
    "hard disk",
    "Linux format ({})",
    "Windows format (NTFS)",
    "Windows and Mac format (exFAT)",
    "FAT format",
    "Good for a Steam library.",
    "For Steam games, prefer a disk in Linux format.",
    "In Files and in {}.",
    "Not opened at this startup. If Windows is also on this computer, turn off its Fast Startup.",
    "Not opened at this startup: the disk was not found, or it has an error.",
    "Encrypted with BitLocker: open it in Files, with its recovery key.",
    "Encrypted: open it in Files, with its password.",
    "Windows' own disk stays closed, so that nothing can harm it.",
    "Already opened at startup by a setting of your own.",
    "Not offered",
    "Done.",
    "{}: a file on it is open. Close it, then try again.",
    "{}: a file on it is open. It closes at the next restart.",
    "{}: Windows did not fully shut down. In Windows, turn off Fast Startup and shut down, then try again.",
    "{}: Windows' own disk stays closed, so that nothing can harm it.",
    "{}: could not be opened.",
    "{}: could not be closed.",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    /// Not opened at startup: the switch is off
    Off,
    /// Opened at startup by Tools, at this folder; whether it is open now
    On { path: String, open: bool },
    /// In /etc/fstab by the person or the installer: theirs
    ByHand,
    /// BitLocker (true) or LUKS (false): needs a key, not offered
    Encrypted { bitlocker: bool },
    /// The partition Windows runs from: not offered
    Windows,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Disk {
    pub uuid: String,
    /// Its label, else what it is ("1 TB disk")
    pub name: String,
    pub size: u64,
    pub ssd: bool,
    pub fstype: String,
    pub status: Status,
}

impl Disk {
    /// Whether the switch can change it
    pub fn switchable(&self) -> bool {
        matches!(self.status, Status::Off | Status::On { .. })
    }

    pub fn at_startup(&self) -> bool {
        matches!(self.status, Status::On { .. } | Status::ByHand)
    }

    /// The row's caption: size and kind, format, then its state or a tip
    pub fn caption(&self) -> String {
        let mut parts = vec![format!(
            "{} {}",
            size_text(self.size),
            tr(if self.ssd { "SSD" } else { "hard disk" })
        )];
        let linux = matches!(self.fstype.as_str(), "ext4" | "btrfs" | "xfs");
        match self.fstype.as_str() {
            "ntfs" => parts.push(tr("Windows format (NTFS)").to_owned()),
            "exfat" => parts.push(tr("Windows and Mac format (exFAT)").to_owned()),
            "vfat" => parts.push(tr("FAT format").to_owned()),
            fs if linux => parts.push(fill(tr("Linux format ({})"), fs)),
            _ => {}
        }
        let first = format!("{}.", parts.join(", "));
        let second = match &self.status {
            Status::On { open: false, .. } if self.fstype == "ntfs" => tr(
                "Not opened at this startup. If Windows is also on this computer, turn off its Fast Startup.",
            )
            .to_owned(),
            Status::On { open: false, .. } => {
                tr("Not opened at this startup: the disk was not found, or it has an error.")
                    .to_owned()
            }
            Status::On { path, .. } => fill(tr("In Files and in {}."), &shown_path(path)),
            Status::ByHand => tr("Already opened at startup by a setting of your own.").to_owned(),
            Status::Encrypted { bitlocker: true } => {
                tr("Encrypted with BitLocker: open it in Files, with its recovery key.").to_owned()
            }
            Status::Encrypted { bitlocker: false } => {
                tr("Encrypted: open it in Files, with its password.").to_owned()
            }
            Status::Windows => {
                tr("Windows' own disk stays closed, so that nothing can harm it.").to_owned()
            }
            Status::Off if linux => tr("Good for a Steam library.").to_owned(),
            Status::Off if self.fstype == "ntfs" => {
                tr("For Steam games, prefer a disk in Linux format.").to_owned()
            }
            Status::Off => String::new(),
        };
        if second.is_empty() {
            first
        } else {
            format!("{first} {second}")
        }
    }

    /// A problem to show in the row: opened at startup, but not open now
    pub fn problem(&self) -> bool {
        matches!(self.status, Status::On { open: false, .. })
    }
}

/// /var/mnt/x as people see it: /mnt/x (/mnt links to var/mnt)
fn shown_path(path: &str) -> String {
    path.strip_prefix("/var")
        .filter(|rest| rest.starts_with("/mnt/"))
        .unwrap_or(path)
        .to_owned()
}

/// A size as disk makers count it: "500 GB", "1 TB", "1.5 TB"
pub fn size_text(bytes: u64) -> String {
    const GB: u64 = 1_000_000_000;
    if bytes >= 1000 * GB {
        let tenths = (bytes + 50 * GB) / (100 * GB);
        let number = if tenths.is_multiple_of(10) {
            (tenths / 10).to_string()
        } else {
            // "1.5 TB" in English, "1,5 To" in Celenn OS's other languages
            let point = if crate::i18n::language().is_some() {
                ","
            } else {
                "."
            };
            format!("{}{point}{}", tenths / 10, tenths % 10)
        };
        fill(tr("{} TB"), &number)
    } else {
        fill(tr("{} GB"), &((bytes + GB / 2) / GB).max(1).to_string())
    }
}

/// Our lines of /etc/fstab (UUID -> folder), and the UUIDs, labels and
/// device paths that other lines mount
pub fn parse_fstab(fstab: &str) -> (BTreeMap<String, String>, BTreeSet<String>) {
    let mut ours = BTreeMap::new();
    let mut others = BTreeSet::new();
    for line in fstab.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 4 || fields[0].starts_with('#') {
            continue;
        }
        if let Some(uuid) = fields[0].strip_prefix("UUID=")
            && fields[3].split(',').any(|option| option == MARK)
        {
            ours.insert(uuid.to_owned(), fields[1].to_owned());
        } else {
            others.insert(fields[0].to_owned());
        }
    }
    (ours, others)
}

fn text(value: &serde_json::Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_owned()
}

fn flag(value: &serde_json::Value, key: &str) -> bool {
    match value.get(key) {
        Some(serde_json::Value::Bool(b)) => *b,
        // Older lsblk: "1" / "0"
        Some(serde_json::Value::String(s)) => s == "1",
        _ => false,
    }
}

fn mountpoints(value: &serde_json::Value) -> Vec<String> {
    let mut points: Vec<String> = value
        .get("mountpoints")
        .and_then(|v| v.as_array())
        .map(|array| {
            array
                .iter()
                .filter_map(|p| p.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    // An opened encrypted disk: what is inside it
    if let Some(children) = value.get("children").and_then(|v| v.as_array()) {
        for child in children {
            points.extend(mountpoints(child));
        }
    }
    points
}

/// The disks to show, from lsblk's JSON (LSBLK_COLUMNS, sizes in bytes),
/// /etc/fstab, the UUIDs the helper refused as Windows' own, and whether a
/// folder is a Windows system (opened in Files, with Windows\System32)
pub fn parse(
    lsblk: &str,
    fstab: &str,
    windows: &BTreeSet<String>,
    is_windows: &dyn Fn(&str) -> bool,
) -> Vec<Disk> {
    let Some(devices) = serde_json::from_str::<serde_json::Value>(lsblk)
        .ok()
        .and_then(|value| value.get("blockdevices")?.as_array().cloned())
    else {
        return Vec::new();
    };
    let (ours, others) = parse_fstab(fstab);
    let mut disks = Vec::new();
    for device in &devices {
        if text(device, "type") != "disk" || flag(device, "rm") || text(device, "tran") == "usb" {
            continue;
        }
        let ssd = !flag(device, "rota");
        let Some(parts) = device.get("children").and_then(|v| v.as_array()) else {
            continue;
        };
        for part in parts {
            let uuid = text(part, "uuid");
            let fstype = text(part, "fstype");
            let parttype = text(part, "parttype").to_lowercase();
            if text(part, "type") != "part" || uuid.is_empty() || NOT_DATA.contains(&&*parttype) {
                continue;
            }
            let ours_path = ours.get(&uuid);
            let points = mountpoints(part);
            // Mounted by the system (/, /boot, /var, swap...): not ours to
            // offer. Opened in Files (/run/media) or by us: fine.
            if points
                .iter()
                .any(|point| !point.starts_with("/run/media/") && Some(point) != ours_path)
            {
                continue;
            }
            let label = text(part, "label");
            let size = part.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
            let status = match fstype.as_str() {
                "BitLocker" => Status::Encrypted { bitlocker: true },
                "crypto_LUKS" => Status::Encrypted { bitlocker: false },
                "ext4" | "btrfs" | "xfs" | "ntfs" | "exfat" | "vfat" => {
                    let path = format!("/dev/{}", text(part, "name"));
                    if let Some(folder) = ours_path {
                        Status::On {
                            path: folder.clone(),
                            open: points.contains(folder),
                        }
                    } else if others.contains(&format!("UUID={uuid}"))
                        || others.contains(&path)
                        || (!label.is_empty() && others.contains(&format!("LABEL={label}")))
                    {
                        Status::ByHand
                    } else if fstype == "ntfs"
                        && (windows.contains(&uuid) || points.iter().any(|p| is_windows(p)))
                    {
                        Status::Windows
                    } else {
                        Status::Off
                    }
                }
                _ => continue,
            };
            // A small FAT partition is a firmware or recovery one, not data
            if fstype == "vfat" && size < 2_000_000_000 {
                continue;
            }
            disks.push(Disk {
                name: if label.is_empty() {
                    fill(tr("{} disk"), &size_text(size))
                } else {
                    label
                },
                uuid,
                size,
                ssd,
                fstype,
                status,
            });
        }
    }
    disks
}

fn state_file() -> PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| Path::new(&home).join(".local/state")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("celenn/windows-disks")
}

fn windows_disks() -> BTreeSet<String> {
    std::fs::read_to_string(state_file())
        .unwrap_or_default()
        .lines()
        .map(str::to_owned)
        .collect()
}

/// Remembers a partition the helper refused as Windows' own
pub fn remember_windows(uuid: &str) {
    let mut known = windows_disks();
    if known.insert(uuid.to_owned()) {
        let file = state_file();
        if let Some(parent) = file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let text: String = known.into_iter().map(|uuid| uuid + "\n").collect();
        let _ = std::fs::write(file, text);
    }
}

/// The disks of this computer (blocking: lsblk)
pub fn read() -> Vec<Disk> {
    let lsblk = Command::new("lsblk")
        .args(["-J", "-b", "-o", LSBLK_COLUMNS])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
        .unwrap_or_default();
    let fstab = std::fs::read_to_string("/etc/fstab").unwrap_or_default();
    parse(&lsblk, &fstab, &windows_disks(), &|folder| {
        Path::new(folder).join("Windows/System32").is_dir()
    })
}

/// The row's caption on the Computer page
pub fn summary(disks: &[Disk]) -> String {
    let on: Vec<&str> = disks
        .iter()
        .filter(|disk| disk.at_startup())
        .map(|disk| disk.name.as_str())
        .collect();
    match on.len() {
        _ if disks.is_empty() => tr("No other disk in this computer.").to_owned(),
        0 => tr("None opens at startup yet: choose the ones Files and Steam should always find.")
            .to_owned(),
        1 => fill(tr("{} opens at every startup."), on[0]),
        _ => fill(tr("{} open at every startup."), &on.join(", ")),
    }
}

/// What the helper said about each disk it did not change (or changed
/// with a note), as lines for the person; and the UUIDs it refused as
/// Windows' own. Its output: one "<UUID> <reason>" line per disk.
pub fn results(output: &str, disks: &[Disk]) -> (Vec<String>, Vec<String>) {
    let mut lines = Vec::new();
    let mut windows = Vec::new();
    for line in output.lines() {
        let Some((uuid, reason)) = line.trim().split_once(' ') else {
            continue;
        };
        let name = disks
            .iter()
            .find(|disk| disk.uuid == uuid)
            .map_or(uuid, |disk| disk.name.as_str());
        let template = match reason {
            "busy" => "{}: a file on it is open. Close it, then try again.",
            "busy-until-restart" => "{}: a file on it is open. It closes at the next restart.",
            "windows-not-shut-down" => {
                "{}: Windows did not fully shut down. In Windows, turn off Fast Startup and shut down, then try again."
            }
            "windows" => {
                windows.push(uuid.to_owned());
                "{}: Windows' own disk stays closed, so that nothing can harm it."
            }
            "not-closed" => "{}: could not be closed.",
            _ => "{}: could not be opened.",
        };
        lines.push(fill(tr(template), name));
    }
    (lines, windows)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A desktop: the system SSD (EFI, /boot, btrfs root), a Windows SSD
    /// (EFI, reserved, C:, recovery), a data hard disk with an NTFS D:
    /// and an ext4 games partition, a BitLocker partition, a USB stick
    const LSBLK: &str = r#"{"blockdevices": [
      {"name":"nvme0n1","type":"disk","fstype":null,"label":null,"uuid":null,"size":512110190592,"rm":false,"rota":false,"tran":"nvme","parttype":null,"mountpoints":[null],
       "children":[
        {"name":"nvme0n1p1","type":"part","fstype":"vfat","label":null,"uuid":"AAAA-0001","size":629145600,"rm":false,"rota":false,"tran":null,"parttype":"c12a7328-f81f-11d2-ba4b-00a0c93ec93b","mountpoints":["/boot/efi"]},
        {"name":"nvme0n1p2","type":"part","fstype":"ext4","label":"boot","uuid":"boot-uuid","size":2147483648,"rm":false,"rota":false,"tran":null,"parttype":"0fc63daf-8483-4772-8e79-3d69d8477de4","mountpoints":["/boot"]},
        {"name":"nvme0n1p3","type":"part","fstype":"btrfs","label":"fedora","uuid":"root-uuid","size":509000000000,"rm":false,"rota":false,"tran":null,"parttype":"0fc63daf-8483-4772-8e79-3d69d8477de4","mountpoints":["/var","/sysroot","/etc"]}
       ]},
      {"name":"nvme1n1","type":"disk","fstype":null,"label":null,"uuid":null,"size":1000204886016,"rm":false,"rota":false,"tran":"nvme","parttype":null,"mountpoints":[null],
       "children":[
        {"name":"nvme1n1p1","type":"part","fstype":"vfat","label":null,"uuid":"BBBB-0001","size":104857600,"rm":false,"rota":false,"tran":null,"parttype":"c12a7328-f81f-11d2-ba4b-00a0c93ec93b","mountpoints":[null]},
        {"name":"nvme1n1p2","type":"part","fstype":null,"label":null,"uuid":null,"size":16777216,"rm":false,"rota":false,"tran":null,"parttype":"e3c9e316-0b5c-4db8-817d-f92df00215ae","mountpoints":[null]},
        {"name":"nvme1n1p3","type":"part","fstype":"ntfs","label":"Windows","uuid":"C0C0C0C0C0C0C0C0","size":998000000000,"rm":false,"rota":false,"tran":null,"parttype":"ebd0a0a2-b9e5-4433-87c0-68b6b72699c7","mountpoints":[null]},
        {"name":"nvme1n1p4","type":"part","fstype":"ntfs","label":null,"uuid":"RECOVERY0000","size":700000000,"rm":false,"rota":false,"tran":null,"parttype":"de94bba4-06d1-4d40-a16a-bfd50179d6ac","mountpoints":[null]}
       ]},
      {"name":"sda","type":"disk","fstype":null,"label":null,"uuid":null,"size":2000398934016,"rm":false,"rota":true,"tran":"sata","parttype":null,"mountpoints":[null],
       "children":[
        {"name":"sda1","type":"part","fstype":"ntfs","label":"Données","uuid":"D0D0D0D0D0D0D0D0","size":1000000000000,"rm":false,"rota":true,"tran":null,"parttype":"ebd0a0a2-b9e5-4433-87c0-68b6b72699c7","mountpoints":["/run/media/lea/Données"]},
        {"name":"sda2","type":"part","fstype":"ext4","label":"Jeux","uuid":"games-uuid","size":900000000000,"rm":false,"rota":true,"tran":null,"parttype":"0fc63daf-8483-4772-8e79-3d69d8477de4","mountpoints":["/var/mnt/Jeux"]},
        {"name":"sda3","type":"part","fstype":"BitLocker","label":null,"uuid":"","size":100000000000,"rm":false,"rota":true,"tran":null,"parttype":"ebd0a0a2-b9e5-4433-87c0-68b6b72699c7","mountpoints":[null]}
       ]},
      {"name":"sdb","type":"disk","fstype":null,"label":null,"uuid":null,"size":32000000000,"rm":true,"rota":false,"tran":"usb","parttype":null,"mountpoints":[null],
       "children":[
        {"name":"sdb1","type":"part","fstype":"exfat","label":"USB","uuid":"USB0-0001","size":32000000000,"rm":true,"rota":false,"tran":null,"parttype":"0x7","mountpoints":["/run/media/lea/USB"]}
       ]},
      {"name":"zram0","type":"disk","fstype":null,"label":null,"uuid":null,"size":8000000000,"rm":false,"rota":false,"tran":null,"parttype":null,"mountpoints":["[SWAP]"]}
    ]}"#;

    const FSTAB: &str = "# /etc/fstab\n\
        UUID=root-uuid / btrfs subvol=root,compress=zstd:1,ro 0 0\n\
        UUID=boot-uuid /boot ext4 defaults 1 2\n\
        UUID=games-uuid /var/mnt/Jeux ext4 defaults,nofail,x-systemd.device-timeout=10s,x-gvfs-show,x-celenn-disk 0 2\n";

    fn disks(windows: &[&str]) -> Vec<Disk> {
        let windows = windows.iter().map(|uuid| (*uuid).to_owned()).collect();
        parse(LSBLK, FSTAB, &windows, &|_| false)
    }

    #[test]
    fn only_internal_data_partitions() {
        let all = disks(&[]);
        let found: Vec<&str> = all.iter().map(|d| d.name.as_str()).collect();
        // BitLocker without a UUID in lsblk is skipped too: nothing to show
        assert_eq!(found, ["Windows", "Données", "Jeux"]);
    }

    #[test]
    fn states() {
        let all = disks(&["C0C0C0C0C0C0C0C0"]);
        assert_eq!(all[0].status, Status::Windows);
        assert!(!all[0].switchable());
        assert_eq!(all[1].status, Status::Off);
        assert!(all[1].switchable());
        assert_eq!(
            all[2].status,
            Status::On {
                path: "/var/mnt/Jeux".into(),
                open: true
            }
        );
        assert!(all[2].caption().contains("/mnt/Jeux."));
        assert!(!all[2].caption().contains("/var/"));
        assert_eq!(summary(&all), "Jeux opens at every startup.");
    }

    #[test]
    fn not_open_is_a_problem() {
        let lsblk = LSBLK.replace(
            r#""mountpoints":["/var/mnt/Jeux"]"#,
            r#""mountpoints":[null]"#,
        );
        let all = parse(&lsblk, FSTAB, &BTreeSet::new(), &|_| false);
        let games = all.iter().find(|d| d.name == "Jeux").unwrap();
        assert!(games.problem());
        assert!(games.caption().contains("Not opened at this startup"));
    }

    #[test]
    fn by_hand_and_encrypted() {
        let fstab = format!("{FSTAB}UUID=D0D0D0D0D0D0D0D0 /data ntfs3 defaults 0 0\n");
        let lsblk = LSBLK.replace(
            r#""uuid":"","size":100000000000"#,
            r#""uuid":"bitlocker-uuid","size":100000000000"#,
        );
        let all = parse(&lsblk, &fstab, &BTreeSet::new(), &|_| false);
        // /run/media mount of a disk that fstab mounts elsewhere: still by hand
        assert_eq!(all[1].status, Status::ByHand);
        assert_eq!(all[3].status, Status::Encrypted { bitlocker: true });
        assert_eq!(all[3].name, "100 GB disk");
        assert_eq!(summary(&all), "Données, Jeux open at every startup.");
    }

    #[test]
    fn windows_seen_in_files() {
        let all = parse(LSBLK, FSTAB, &BTreeSet::new(), &|folder| {
            folder == "/run/media/lea/Données"
        });
        assert_eq!(all[1].status, Status::Windows);
    }

    #[test]
    fn no_disk() {
        assert_eq!(summary(&[]), "No other disk in this computer.");
        assert!(parse("", "", &BTreeSet::new(), &|_| false).is_empty());
    }

    #[test]
    fn sizes() {
        assert_eq!(size_text(512_110_190_592), "512 GB");
        assert_eq!(size_text(1_000_204_886_016), "1 TB");
        assert_eq!(size_text(2_000_398_934_016), "2 TB");
        assert_eq!(size_text(1_500_000_000_000), "1.5 TB");
        assert_eq!(size_text(100), "1 GB");
    }

    #[test]
    fn helper_results() {
        let all = disks(&[]);
        let (lines, windows) = results(
            "C0C0C0C0C0C0C0C0 windows\nD0D0D0D0D0D0D0D0 windows-not-shut-down\ngames-uuid busy-until-restart\n",
            &all,
        );
        assert_eq!(windows, ["C0C0C0C0C0C0C0C0"]);
        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with("Windows: "));
        assert!(lines[1].starts_with("Données: Windows did not fully shut down"));
        assert!(lines[2].starts_with("Jeux: "));
    }

    #[test]
    fn fstab_lines() {
        let (ours, others) = parse_fstab(FSTAB);
        assert_eq!(
            ours.get("games-uuid").map(String::as_str),
            Some("/var/mnt/Jeux")
        );
        assert!(others.contains("UUID=root-uuid"));
        assert!(!others.contains("UUID=games-uuid"));
    }
}
