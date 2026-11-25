// "Is this computer ready?": the live session's look at the hardware before
// installing (second page of `cosmic-blue-tour --live`). It only reads:
// /sys, /proc, and lspci and lsblk, without root, without the network. Each
// check says what it found in plain words, and for a problem what to do.
// A check that cannot run says so: never a false alarm.

use crate::hardware::{gpu_name, memory_gb, pci_names, vendor_by_id};
use crate::i18n::{fill, tr};
use std::path::Path;
use std::process::{Command, Stdio};

/// Every text of the checks, for the translation check (main.rs all_texts)
pub const TEXTS: &[&str] = &[
    "Wi-Fi",
    "{}: works.",
    "Wi-Fi works.",
    "{} has no free driver. Plug in a network cable, or a small USB Wi-Fi adapter: everything else works.",
    "{} was found but does not start. Its driver may need firmware that is missing.",
    "No Wi-Fi card found. A network cable works.",
    "Sound",
    "Sound card found.",
    "No sound card was recognized. USB or Bluetooth headphones may still work.",
    "Graphics card",
    "{}: the driver is already in the system.",
    "{}: the NVIDIA driver is ready.",
    "{}. For games, switch to the NVIDIA edition after installing: one click in cosmic-blue Tools → NVIDIA driver.",
    "{}: no driver, the picture may be slow.",
    "Memory",
    "{} GB: too little, 4 GB is the minimum.",
    "{} GB: it works, but 8 GB are recommended for a browser with many tabs.",
    "{} GB of memory.",
    "Disk",
    "No internal disk found. If this computer has one, its firmware may hide it: in its settings, set the storage mode to AHCI instead of RAID or Intel RST.",
    "{} GB disk: 64 GB is the minimum.",
    "{} GB disk, empty.",
    "{} GB disk.",
    "{} GB disk. Windows is on it: you can keep both.",
    "{} GB disk. Windows is on it, encrypted with BitLocker: before installing, turn BitLocker off in Windows, or at least write down its recovery key.",
    "Could not be checked.",
    // The pages (tour_view.rs)
    "Checking this computer…",
    "Test the sound",
    "How to do it",
    "Copy the report",
    "Copied",
    "A printer or a game controller: plug it in to check it.",
    "One thing to look at before installing",
    "{} things to look at before installing",
    "Review",
    "Installation guide",
    "Keep Windows next to it, choose where to install, the NVIDIA key.",
];

/// How a check went
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    Good,
    /// Worth knowing, nothing wrong
    Info,
    /// Works, with something to look at
    Warn,
    /// Will not work as it is
    Bad,
}

/// A button next to a check
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Help {
    /// The install guide's "Choose where to install" step (BitLocker)
    Disk,
    /// Play a sound
    TestSound,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Check {
    pub status: Status,
    /// English, translated when shown
    pub title: &'static str,
    /// Already translated
    pub caption: String,
    pub help: Option<Help>,
}

fn check(status: Status, title: &'static str, caption: String) -> Check {
    Check {
        status,
        title,
        caption,
        help: None,
    }
}

/// Runs every check (about a second; run off the window's thread)
pub fn run() -> Vec<Check> {
    let pci = pci_devices();
    vec![
        wifi(&pci, wireless_interfaces()),
        sound(&read("/proc/asound/cards")),
        graphics(&pci),
        memory(memory_gb(&read("/proc/meminfo"))),
        disk(command("lsblk", &DISK_COLUMNS).as_deref()),
    ]
}

/// The report the "Copy the report" button copies, for a forum: one line
/// per check, after the cosmic-blue version. Nothing personal.
pub fn report(checks: &[Check], version: &str) -> String {
    let mut text = format!("cosmic-blue {version}, live session\n");
    for check in checks {
        let mark = match check.status {
            Status::Good => "OK",
            Status::Info => "Info",
            Status::Warn => "Check",
            Status::Bad => "Problem",
        };
        text.push_str(&format!(
            "[{mark}] {}: {}\n",
            tr(check.title),
            check.caption
        ));
    }
    text
}

/// cosmic-blue's version (VERSION of os-release), for the report
pub fn os_version() -> String {
    read("/usr/lib/os-release")
        .lines()
        .find_map(|line| line.strip_prefix("VERSION="))
        .map(|value| value.trim_matches('"').to_owned())
        .unwrap_or_default()
}

fn read(path: impl AsRef<Path>) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

/// A program's output, stopped after 5 seconds (a disk that does not
/// answer must not freeze the page); None when it fails
fn command(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new("timeout")
        .arg("5")
        .arg(program)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

/// A PCI device: its class, vendor id, driver and name
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pci {
    /// "0x028000": network controller, wireless
    pub class: String,
    pub vendor: String,
    /// Kernel driver in use, if any
    pub driver: Option<String>,
    pub name: String,
}

fn pci_devices() -> Vec<Pci> {
    let mut devices: Vec<Pci> = std::fs::read_dir("/sys/bus/pci/devices")
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| {
            let path = entry.path();
            let slot = entry.file_name().to_string_lossy().into_owned();
            let class = read(path.join("class")).trim().to_owned();
            let vendor = read(path.join("vendor")).trim().to_owned();
            let driver = std::fs::read_link(path.join("driver"))
                .ok()
                .and_then(|link| Some(link.file_name()?.to_string_lossy().into_owned()));
            // Names only for what the page shows (lspci is slow-ish)
            let name = if class.starts_with("0x0280") || class.starts_with("0x03") {
                match pci_names(&slot) {
                    Some((vendor_name, device)) => gpu_name(&vendor_name, &device),
                    None => vendor_by_id(&vendor).to_owned(),
                }
            } else {
                String::new()
            };
            Pci {
                class,
                vendor,
                driver,
                name,
            }
        })
        .collect();
    devices.sort_by(|a, b| a.name.cmp(&b.name));
    devices
}

/// Network interfaces that are Wi-Fi (PCI, USB or other)
fn wireless_interfaces() -> usize {
    std::fs::read_dir("/sys/class/net")
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().join("wireless").exists())
        .count()
}

fn or_card(name: &str) -> String {
    if name.is_empty() {
        "Wi-Fi".to_owned()
    } else {
        name.to_owned()
    }
}

pub fn wifi(pci: &[Pci], interfaces: usize) -> Check {
    let cards: Vec<&Pci> = pci
        .iter()
        .filter(|device| device.class.starts_with("0x0280"))
        .collect();
    let title = "Wi-Fi";
    // A card without a driver, and no other card that works
    if let (0, Some(card)) = (interfaces, cards.iter().find(|card| card.driver.is_none())) {
        return check(
            Status::Bad,
            title,
            fill(
                tr(
                    "{} has no free driver. Plug in a network cable, or a small USB Wi-Fi adapter: everything else works.",
                ),
                &or_card(&card.name),
            ),
        );
    }
    if interfaces > 0 {
        return match cards.iter().find(|card| card.driver.is_some()) {
            Some(card) if !card.name.is_empty() => {
                check(Status::Good, title, fill(tr("{}: works."), &card.name))
            }
            _ => check(Status::Good, title, tr("Wi-Fi works.").to_owned()),
        };
    }
    match cards.first() {
        // A driver, but no Wi-Fi interface: usually missing firmware
        Some(card) => check(
            Status::Warn,
            title,
            fill(
                tr(
                    "{} was found but does not start. Its driver may need firmware that is missing.",
                ),
                &or_card(&card.name),
            ),
        ),
        None => check(
            Status::Info,
            title,
            tr("No Wi-Fi card found. A network cable works.").to_owned(),
        ),
    }
}

/// From /proc/asound/cards: " 0 [PCH ]: HDA-Intel - HDA Intel PCH"
pub fn sound(cards: &str) -> Check {
    let found = cards.lines().any(|line| {
        let line = line.trim_start();
        line.chars().next().is_some_and(|c| c.is_ascii_digit()) && line.contains('[')
    });
    if found {
        Check {
            help: Some(Help::TestSound),
            ..check(Status::Good, "Sound", tr("Sound card found.").to_owned())
        }
    } else {
        check(
            Status::Warn,
            "Sound",
            tr("No sound card was recognized. USB or Bluetooth headphones may still work.")
                .to_owned(),
        )
    }
}

pub fn graphics(pci: &[Pci]) -> Check {
    let title = "Graphics card";
    let mut status = Status::Good;
    let mut sentences = Vec::new();
    for card in pci.iter().filter(|device| device.class.starts_with("0x03")) {
        let name = if card.name.is_empty() {
            tr(title).to_owned()
        } else {
            card.name.clone()
        };
        let (card_status, text) = match (card.vendor.as_str(), card.driver.as_deref()) {
            (_, None) => (Status::Warn, tr("{}: no driver, the picture may be slow.")),
            ("0x10de", Some("nvidia")) => (Status::Good, tr("{}: the NVIDIA driver is ready.")),
            ("0x10de", Some(_)) => (
                Status::Info,
                tr(
                    "{}. For games, switch to the NVIDIA edition after installing: one click in cosmic-blue Tools → NVIDIA driver.",
                ),
            ),
            _ => (Status::Good, tr("{}: the driver is already in the system.")),
        };
        status = status.max(card_status);
        sentences.push((card_status, fill(text, &name)));
    }
    if sentences.is_empty() {
        return check(Status::Info, title, tr("Could not be checked.").to_owned());
    }
    // A card to look at: only what is said about it (a laptop's Intel
    // graphics next to its NVIDIA card are not news)
    let caption: Vec<String> = sentences
        .into_iter()
        .filter(|(card_status, _)| *card_status == status)
        .map(|(_, sentence)| sentence)
        .collect();
    check(status, title, caption.join(" "))
}

pub fn memory(gb: u64) -> Check {
    let value = gb.to_string();
    match gb {
        0 => check(
            Status::Info,
            "Memory",
            tr("Could not be checked.").to_owned(),
        ),
        1..=3 => check(
            Status::Bad,
            "Memory",
            fill(tr("{} GB: too little, 4 GB is the minimum."), &value),
        ),
        4..=7 => check(
            Status::Warn,
            "Memory",
            fill(
                tr("{} GB: it works, but 8 GB are recommended for a browser with many tabs."),
                &value,
            ),
        ),
        _ => check(Status::Good, "Memory", fill(tr("{} GB of memory."), &value)),
    }
}

const DISK_COLUMNS: [&str; 4] = [
    "--json",
    "--bytes",
    "--output",
    "NAME,TYPE,SIZE,RM,TRAN,FSTYPE",
];
/// 64 GB, as disk makers count (a "64 GB" disk is a little less)
const MINIMUM_DISK: u64 = 60_000_000_000;

/// The largest internal disk, from lsblk's JSON (DISK_COLUMNS). The USB
/// stick the live session runs from, removable disks, CD drives, zram and
/// loop devices are not internal disks. Windows on it: you can keep it;
/// BitLocker on it: something to do before installing.
pub fn disk(lsblk: Option<&str>) -> Check {
    let title = "Disk";
    let Some(devices) = lsblk
        .and_then(|json| serde_json::from_str::<serde_json::Value>(json).ok())
        .and_then(|value| value.get("blockdevices")?.as_array().cloned())
    else {
        return check(Status::Info, title, tr("Could not be checked.").to_owned());
    };
    let text = |value: &serde_json::Value, key: &str| {
        value
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_owned()
    };
    let removable = |value: &serde_json::Value| match value.get("rm") {
        Some(serde_json::Value::Bool(rm)) => *rm,
        Some(serde_json::Value::String(rm)) => rm == "1",
        Some(serde_json::Value::Number(rm)) => rm.as_u64() == Some(1),
        _ => false,
    };
    let size = |value: &serde_json::Value| match value.get("size") {
        Some(serde_json::Value::Number(size)) => size.as_u64().unwrap_or(0),
        Some(serde_json::Value::String(size)) => size.parse().unwrap_or(0),
        _ => 0,
    };
    let internal = devices.iter().filter(|device| {
        let name = text(device, "name");
        text(device, "type") == "disk"
            && !removable(device)
            && text(device, "tran") != "usb"
            && !name.starts_with("zram")
            && !name.starts_with("loop")
    });
    let Some(largest) = internal.max_by_key(|device| size(device)) else {
        return check(
            Status::Bad,
            title,
            tr("No internal disk found. If this computer has one, its firmware may hide it: in its settings, set the storage mode to AHCI instead of RAID or Intel RST.").to_owned(),
        );
    };
    let gb = (size(largest) as f64 / 1e9).round() as u64;
    let value = gb.to_string();
    let parts: Vec<serde_json::Value> = largest
        .get("children")
        .and_then(|children| children.as_array().cloned())
        .unwrap_or_default();
    let fstypes: Vec<String> = parts
        .iter()
        .map(|part| text(part, "fstype").to_lowercase())
        .collect();
    let bitlocker = fstypes.iter().any(|fs| fs == "bitlocker");
    let windows = bitlocker || fstypes.iter().any(|fs| fs == "ntfs");
    if size(largest) < MINIMUM_DISK {
        check(
            Status::Bad,
            title,
            fill(tr("{} GB disk: 64 GB is the minimum."), &value),
        )
    } else if bitlocker {
        Check {
            help: Some(Help::Disk),
            ..check(
                Status::Warn,
                title,
                fill(
                    tr(
                        "{} GB disk. Windows is on it, encrypted with BitLocker: before installing, turn BitLocker off in Windows, or at least write down its recovery key.",
                    ),
                    &value,
                ),
            )
        }
    } else if windows {
        check(
            Status::Good,
            title,
            fill(
                tr("{} GB disk. Windows is on it: you can keep both."),
                &value,
            ),
        )
    } else if parts.is_empty() {
        check(Status::Good, title, fill(tr("{} GB disk, empty."), &value))
    } else {
        check(Status::Good, title, fill(tr("{} GB disk."), &value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(class: &str, vendor: &str, driver: Option<&str>, name: &str) -> Pci {
        Pci {
            class: class.to_owned(),
            vendor: vendor.to_owned(),
            driver: driver.map(str::to_owned),
            name: name.to_owned(),
        }
    }

    #[test]
    fn wifi_cards() {
        let intel = card("0x028000", "0x8086", Some("iwlwifi"), "Intel Wi-Fi 6 AX201");
        assert_eq!(wifi(std::slice::from_ref(&intel), 1).status, Status::Good);
        assert_eq!(
            wifi(std::slice::from_ref(&intel), 1).caption,
            "Intel Wi-Fi 6 AX201: works."
        );
        // Driver loaded, no interface: firmware
        assert_eq!(wifi(&[intel], 0).status, Status::Warn);
        let broadcom = card("0x028000", "0x14e4", None, "Broadcom BCM4360");
        assert_eq!(wifi(std::slice::from_ref(&broadcom), 0).status, Status::Bad);
        // A USB adapter works around it
        assert_eq!(wifi(&[broadcom], 1).status, Status::Good);
        // A desktop without Wi-Fi
        assert_eq!(wifi(&[], 0).status, Status::Info);
        assert_eq!(wifi(&[], 1).caption, "Wi-Fi works.");
    }

    #[test]
    fn sound_cards() {
        let cards = " 0 [PCH            ]: HDA-Intel - HDA Intel PCH\n                      HDA Intel PCH at 0xf1230000 irq 147\n";
        assert_eq!(sound(cards).status, Status::Good);
        assert_eq!(sound(cards).help, Some(Help::TestSound));
        assert_eq!(sound("--- no soundcards ---\n").status, Status::Warn);
        assert_eq!(sound("").status, Status::Warn);
    }

    #[test]
    fn graphics_cards() {
        let intel = card("0x030000", "0x8086", Some("i915"), "Intel Iris Xe Graphics");
        assert_eq!(graphics(std::slice::from_ref(&intel)).status, Status::Good);
        let nouveau = card(
            "0x030000",
            "0x10de",
            Some("nouveau"),
            "NVIDIA GeForce GTX 1060",
        );
        let both = graphics(&[intel, nouveau.clone()]);
        assert_eq!(both.status, Status::Info);
        // Only the card to look at
        assert!(
            both.caption
                .starts_with("NVIDIA GeForce GTX 1060. For games")
        );
        let nvidia = Pci {
            driver: Some("nvidia".to_owned()),
            ..nouveau
        };
        assert_eq!(graphics(&[nvidia]).status, Status::Good);
        let none = card("0x030000", "0x1002", None, "AMD Radeon RX 9070");
        assert_eq!(graphics(&[none]).status, Status::Warn);
        // Network cards are not graphics cards
        let wifi = card("0x028000", "0x8086", Some("iwlwifi"), "Intel");
        assert_eq!(graphics(&[wifi]).status, Status::Info);
    }

    #[test]
    fn memory_sizes() {
        assert_eq!(memory(2).status, Status::Bad);
        assert_eq!(memory(4).status, Status::Warn);
        assert_eq!(memory(16).status, Status::Good);
        assert_eq!(memory(16).caption, "16 GB of memory.");
        assert_eq!(memory(0).status, Status::Info);
    }

    #[test]
    fn disks() {
        // The live USB stick, an NVMe disk with Windows and BitLocker
        let json = r#"{"blockdevices": [
            {"name":"sda","type":"disk","size":31914983424,"rm":true,"tran":"usb","fstype":null,
             "children":[{"name":"sda1","type":"part","size":4000000000,"rm":true,"tran":null,"fstype":"iso9660"}]},
            {"name":"zram0","type":"disk","size":8589934592,"rm":false,"tran":null,"fstype":null},
            {"name":"nvme0n1","type":"disk","size":512110190592,"rm":false,"tran":"nvme","fstype":null,
             "children":[
                {"name":"nvme0n1p1","type":"part","size":272629760,"rm":false,"tran":"nvme","fstype":"vfat"},
                {"name":"nvme0n1p3","type":"part","size":510000000000,"rm":false,"tran":"nvme","fstype":"BitLocker"}]}
        ]}"#;
        let bitlocker = disk(Some(json));
        assert_eq!(bitlocker.status, Status::Warn);
        assert_eq!(bitlocker.help, Some(Help::Disk));
        assert!(
            bitlocker
                .caption
                .starts_with("512 GB disk. Windows is on it, encrypted")
        );
        let ntfs = json.replace("BitLocker", "ntfs");
        assert_eq!(
            disk(Some(&ntfs)).caption,
            "512 GB disk. Windows is on it: you can keep both."
        );
        assert_eq!(disk(Some(&ntfs)).status, Status::Good);
        // Older lsblk: "0"/"1" strings, sizes as strings
        let old = r#"{"blockdevices":[{"name":"vda","type":"disk","size":"42949672960","rm":"0","tran":null,"fstype":null}]}"#;
        let small = disk(Some(old));
        assert_eq!(small.status, Status::Bad);
        assert_eq!(small.caption, "43 GB disk: 64 GB is the minimum.");
        let empty = r#"{"blockdevices":[{"name":"sda","type":"disk","size":256060514304,"rm":false,"tran":"sata","fstype":null}]}"#;
        assert_eq!(disk(Some(empty)).caption, "256 GB disk, empty.");
        // Only the USB stick: the internal disk is hidden (RAID mode)
        let hidden = r#"{"blockdevices":[{"name":"sda","type":"disk","size":31914983424,"rm":false,"tran":"usb","fstype":null}]}"#;
        assert_eq!(disk(Some(hidden)).status, Status::Bad);
        assert_eq!(disk(None).status, Status::Info);
        assert_eq!(disk(Some("not json")).status, Status::Info);
    }

    #[test]
    fn forum_report() {
        let report = report(&[memory(4), memory(16)], "45.20261125");
        assert!(report.starts_with("cosmic-blue 45.20261125, live session\n"));
        assert!(report.contains("[Check] Memory: 4 GB"));
        assert!(report.contains("[OK] Memory: 16 GB of memory."));
    }
}
