// What the Computer page of Celenn OS Tools (more.rs) says about the
// computer itself: its model, processor, memory, graphics cards and how it
// starts. Read once, without root, from /sys and /proc (and lspci for the
// names of the graphics cards, when it is there). Nothing personal: no
// serial numbers, no user or host name.

use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::OnceLock;

/// The computer, as the page shows it. Empty texts: not found
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Hardware {
    /// Maker and model, e.g. "Lenovo ThinkPad T14 Gen 3"
    pub model: String,
    /// The hypervisor's name when this is a virtual machine, e.g. "QEMU"
    pub virtual_machine: Option<String>,
    pub cpu: String,
    pub cores: usize,
    /// Installed memory, in whole GB
    pub memory_gb: u64,
    /// The graphics cards, e.g. ["Intel Iris Xe Graphics"]
    pub gpus: Vec<String>,
    pub boot: Boot,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Boot {
    #[default]
    Bios,
    /// UEFI: Some(whether Secure Boot is on), None when it cannot be read
    Uefi(Option<bool>),
}

pub fn hardware() -> &'static Hardware {
    static HARDWARE: OnceLock<Hardware> = OnceLock::new();
    HARDWARE.get_or_init(read)
}

fn read_file(path: impl AsRef<Path>) -> String {
    std::fs::read_to_string(path)
        .map(|text| text.trim().to_owned())
        .unwrap_or_default()
}

fn read() -> Hardware {
    let dmi = |name: &str| read_file(Path::new("/sys/class/dmi/id").join(name));
    let virt = Command::new("systemd-detect-virt")
        .arg("--vm")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .unwrap_or_default();
    let cpuinfo = read_file("/proc/cpuinfo");
    let (cpu, cores) = cpu(&cpuinfo);
    Hardware {
        model: model(
            &dmi("sys_vendor"),
            &dmi("product_name"),
            &dmi("product_version"),
            &dmi("board_vendor"),
            &dmi("board_name"),
        ),
        virtual_machine: hypervisor(&virt),
        cpu,
        cores: if cores > 0 {
            cores
        } else {
            std::thread::available_parallelism().map_or(0, usize::from)
        },
        memory_gb: memory_gb(&read_file("/proc/meminfo")),
        gpus: gpus(),
        boot: boot(),
    }
}

/// Texts that firmware makers leave in place of a real name
fn placeholder(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.is_empty()
        || [
            "to be filled",
            "o.e.m",
            "default string",
            "system product name",
            "system manufacturer",
            "not applicable",
            "not specified",
            "none",
            "unknown",
        ]
        .iter()
        .any(|junk| lower.contains(junk))
}

/// The maker's usual short name
fn short_vendor(vendor: &str) -> String {
    let lower = vendor.to_lowercase();
    for (key, short) in [
        ("lenovo", "Lenovo"),
        ("hewlett", "HP"),
        ("dell", "Dell"),
        ("asustek", "ASUS"),
        ("micro-star", "MSI"),
        ("gigabyte", "Gigabyte"),
        ("acer", "Acer"),
        ("framework", "Framework"),
        ("microsoft", "Microsoft"),
        ("apple", "Apple"),
        ("samsung", "Samsung"),
        ("system76", "System76"),
        ("tuxedo", "TUXEDO"),
    ] {
        if lower.contains(key) {
            return short.to_owned();
        }
    }
    vendor.trim_end_matches([',', '.']).to_owned()
}

/// "Maker Model", from the firmware's tables (/sys/class/dmi/id). Lenovo
/// puts the model's name in product_version, the product_name being a
/// part number. A computer built from parts: its motherboard
pub fn model(
    vendor: &str,
    product: &str,
    version: &str,
    board_vendor: &str,
    board: &str,
) -> String {
    let (vendor, name) = if vendor.to_lowercase().contains("lenovo") && !placeholder(version) {
        (vendor, version)
    } else if !placeholder(product) {
        (vendor, product)
    } else if !placeholder(board) {
        (board_vendor, board)
    } else {
        return String::new();
    };
    let vendor = if placeholder(vendor) {
        String::new()
    } else {
        short_vendor(vendor)
    };
    if vendor.is_empty() || name.to_lowercase().starts_with(&vendor.to_lowercase()) {
        name.to_owned()
    } else {
        format!("{vendor} {name}")
    }
}

/// systemd-detect-virt's answer, as people know the hypervisor
pub fn hypervisor(virt: &str) -> Option<String> {
    let name = match virt {
        "" | "none" => return None,
        "kvm" | "qemu" => "QEMU/KVM",
        "oracle" => "VirtualBox",
        "vmware" => "VMware",
        "microsoft" => "Hyper-V",
        "parallels" => "Parallels",
        "xen" => "Xen",
        "apple" => "Apple",
        other => other,
    };
    Some(name.to_owned())
}

/// The processor's name, without trademarks or its speed, and its number
/// of cores (not threads: two threads of one core count once)
pub fn cpu(cpuinfo: &str) -> (String, usize) {
    let mut name = String::new();
    let mut cores = std::collections::BTreeSet::new();
    let mut threads = 0;
    let mut physical = "";
    for line in cpuinfo.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let (key, value) = (key.trim(), value.trim());
        match key {
            "model name" if name.is_empty() => name = value.to_owned(),
            "processor" => threads += 1,
            "physical id" => physical = value,
            "core id" => {
                cores.insert((physical.to_owned(), value.to_owned()));
            }
            _ => {}
        }
    }
    for junk in ["(R)", "(r)", "(TM)", "(tm)"] {
        name = name.replace(junk, "");
    }
    if let Some(at) = name.find(" @ ") {
        name.truncate(at);
    }
    // "AMD Ryzen 7 5800X 8-Core Processor", "Intel Core i5-1135G7 CPU"
    let mut words: Vec<&str> = name.split_whitespace().collect();
    while let Some(last) = words.last() {
        let lower = last.to_lowercase();
        if lower == "processor" || lower == "cpu" || lower.ends_with("-core") {
            words.pop();
        } else {
            break;
        }
    }
    let count = if cores.is_empty() {
        threads
    } else {
        cores.len()
    };
    (words.join(" "), count)
}

/// Installed memory in whole GB. MemTotal is a little less than what is
/// installed (the kernel and the graphics keep some): rounded up
pub fn memory_gb(meminfo: &str) -> u64 {
    meminfo
        .lines()
        .find_map(|line| line.strip_prefix("MemTotal:"))
        .and_then(|rest| rest.split_whitespace().next()?.parse::<u64>().ok())
        .map_or(0, |kib| kib.div_ceil(1024 * 1024))
}

/// A device's name (graphics card, Wi-Fi card) from lspci's vendor and
/// device names: the marketing name in brackets when there is one ("TU117M
/// [GeForce GTX 1650 Mobile]" -> "NVIDIA GeForce GTX 1650 Mobile")
pub fn gpu_name(vendor: &str, device: &str) -> String {
    let lower = vendor.to_lowercase();
    let vendor = if lower.contains("nvidia") {
        "NVIDIA"
    } else if lower.contains("advanced micro devices") || lower.contains("amd") {
        "AMD"
    } else if lower.contains("intel") {
        "Intel"
    } else if lower.contains("broadcom") {
        "Broadcom"
    } else if lower.contains("realtek") {
        "Realtek"
    } else if lower.contains("qualcomm") || lower.contains("atheros") {
        "Qualcomm"
    } else if lower.contains("mediatek") {
        "MediaTek"
    } else if lower.contains("red hat") {
        "Red Hat"
    } else if lower.contains("vmware") {
        "VMware"
    } else {
        vendor
    };
    let device = match (device.rfind('['), device.rfind(']')) {
        (Some(open), Some(close)) if open < close => &device[open + 1..close],
        _ => device,
    };
    // Several models in one name: the first ("Radeon RX 6600/6600 XT")
    let device = device.split('/').next().unwrap_or(device).trim();
    if device.to_lowercase().starts_with(&vendor.to_lowercase()) {
        device.to_owned()
    } else {
        format!("{vendor} {device}").trim().to_owned()
    }
}

/// The graphics cards with a display device (/sys/class/drm/cardN)
fn gpus() -> Vec<String> {
    let mut slots: Vec<String> = std::fs::read_dir("/sys/class/drm")
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            name.strip_prefix("card")
                .is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
        })
        .filter_map(|entry| std::fs::canonicalize(entry.path().join("device")).ok())
        .filter_map(|device| Some(device.file_name()?.to_string_lossy().into_owned()))
        .collect();
    slots.sort();
    slots.dedup();
    slots
        .iter()
        .map(|slot| match pci_names(slot) {
            Some((vendor, device)) => gpu_name(&vendor, &device),
            None => {
                vendor_by_id(&read_file(format!("/sys/bus/pci/devices/{slot}/vendor"))).to_owned()
            }
        })
        .filter(|name| !name.is_empty())
        .collect()
}

/// A PCI vendor's short name from its id ("0x10de"), for when lspci is
/// missing; empty when unknown
pub fn vendor_by_id(id: &str) -> &'static str {
    match id {
        "0x10de" => "NVIDIA",
        "0x1002" => "AMD",
        "0x8086" => "Intel",
        "0x14e4" => "Broadcom",
        "0x10ec" => "Realtek",
        "0x168c" | "0x17cb" => "Qualcomm",
        "0x14c3" => "MediaTek",
        "0x1af4" => "Virtio",
        "0x15ad" => "VMware",
        _ => "",
    }
}

/// The vendor and device names of a PCI device (slot "0000:01:00.0"), from
/// lspci, when it is there and knows them
pub fn pci_names(slot: &str) -> Option<(String, String)> {
    // lspci -mm: Slot "Class" "Vendor" "Device" ...
    let output = Command::new("lspci")
        .args(["-mm", "-s", slot])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let line = String::from_utf8_lossy(&output.stdout);
    let fields: Vec<&str> = line.split('"').collect();
    match (fields.get(3), fields.get(5)) {
        (Some(vendor), Some(device)) if !device.is_empty() => {
            Some(((*vendor).to_owned(), (*device).to_owned()))
        }
        _ => None,
    }
}

fn boot() -> Boot {
    if !Path::new("/sys/firmware/efi").exists() {
        return Boot::Bios;
    }
    // The SecureBoot variable: 4 bytes of attributes, then 1 when it is on
    let secure_boot =
        std::fs::read("/sys/firmware/efi/efivars/SecureBoot-8be4df61-93ca-11d2-aa0d-00e098032b8c")
            .ok()
            .and_then(|bytes| bytes.last().copied())
            .map(|on| on == 1);
    Boot::Uefi(secure_boot)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn models() {
        assert_eq!(
            model(
                "LENOVO",
                "21AH00BDGE",
                "ThinkPad T14 Gen 3",
                "LENOVO",
                "21AH00BDGE"
            ),
            "Lenovo ThinkPad T14 Gen 3"
        );
        assert_eq!(
            model("Dell Inc.", "XPS 13 9310", "", "", ""),
            "Dell XPS 13 9310"
        );
        assert_eq!(
            model(
                "To Be Filled By O.E.M.",
                "To Be Filled By O.E.M.",
                "",
                "ASUSTeK COMPUTER INC.",
                "PRIME B550-PLUS"
            ),
            "ASUS PRIME B550-PLUS"
        );
        assert_eq!(
            model("Framework", "Laptop 13", "", "", ""),
            "Framework Laptop 13"
        );
        assert_eq!(model("", "", "", "", ""), "");
    }

    #[test]
    fn cpus() {
        let info = "processor\t: 0\nmodel name\t: AMD Ryzen 7 5800X 8-Core Processor\nphysical id\t: 0\ncore id\t: 0\n\nprocessor\t: 1\nmodel name\t: AMD Ryzen 7 5800X 8-Core Processor\nphysical id\t: 0\ncore id\t: 0\n\nprocessor\t: 2\nphysical id\t: 0\ncore id\t: 1\n";
        assert_eq!(cpu(info), ("AMD Ryzen 7 5800X".to_owned(), 2));
        let intel = "processor : 0\nmodel name : Intel(R) Core(TM) i5-8250U CPU @ 1.60GHz\n";
        assert_eq!(cpu(intel), ("Intel Core i5-8250U".to_owned(), 1));
    }

    #[test]
    fn memory() {
        assert_eq!(memory_gb("MemTotal:        8131544 kB\n"), 8);
        assert_eq!(memory_gb("MemTotal:       15994000 kB\n"), 16);
        assert_eq!(memory_gb(""), 0);
    }

    #[test]
    fn gpu_names() {
        assert_eq!(
            gpu_name(
                "NVIDIA Corporation",
                "TU117M [GeForce GTX 1650 Mobile / Max-Q]"
            ),
            "NVIDIA GeForce GTX 1650 Mobile"
        );
        assert_eq!(
            gpu_name("Intel Corporation", "Alder Lake-P GT2 [Iris Xe Graphics]"),
            "Intel Iris Xe Graphics"
        );
        assert_eq!(
            gpu_name(
                "Advanced Micro Devices, Inc. [AMD/ATI]",
                "Navi 23 [Radeon RX 6600/6600 XT/6600M]"
            ),
            "AMD Radeon RX 6600"
        );
        assert_eq!(
            gpu_name("Red Hat, Inc.", "Virtio 1.0 GPU"),
            "Red Hat Virtio 1.0 GPU"
        );
    }

    #[test]
    fn hypervisors() {
        assert_eq!(hypervisor("none"), None);
        assert_eq!(hypervisor("kvm").as_deref(), Some("QEMU/KVM"));
    }
}
