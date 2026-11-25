// Whether the computer reaches the internet, as NetworkManager sees it:
// shared by the tour (system.rs) and the error cards (failure.rs)

use std::process::{Command, Stdio};

/// Some(true) with full internet access; Some(false) without, and also on a
/// network that does not reach the internet ("limited") or that asks to log
/// in first (a captive portal: "portal", hotel or station Wi-Fi); None when
/// unknown
pub fn online() -> Option<bool> {
    let output = Command::new("nmcli")
        .args(["networking", "connectivity"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    connectivity(&String::from_utf8_lossy(&output.stdout))
}

/// `nmcli networking connectivity`'s answer, read as online()
fn connectivity(answer: &str) -> Option<bool> {
    match answer.trim() {
        "full" => Some(true),
        "none" | "limited" | "portal" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::connectivity;

    #[test]
    fn connectivity_answers() {
        assert_eq!(connectivity("full\n"), Some(true));
        for answer in ["none", "limited", "portal\n"] {
            assert_eq!(connectivity(answer), Some(false), "{answer}");
        }
        assert_eq!(connectivity("unknown"), None);
        assert_eq!(connectivity(""), None);
    }
}
