// Which cosmic-blue image runs, and whether it suits the computer's graphics
// card: shared by the tour (system.rs) and Tools (more.rs)

use std::path::Path;

const IMAGE_INFO: &str = "/usr/share/ublue-os/image-info.json";

/// "cosmic-blue" or "cosmic-blue-nvidia" (empty when unknown)
pub fn image_name() -> String {
    std::fs::read_to_string(IMAGE_INFO)
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .and_then(|info| info.get("image-name")?.as_str().map(str::to_owned))
        .unwrap_or_default()
}

/// An NVIDIA graphics card: Some(whether the NVIDIA edition runs); None
/// without one
pub fn nvidia() -> Option<bool> {
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
    found.then(|| image_name().ends_with("-nvidia"))
}
