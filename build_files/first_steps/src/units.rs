// Sizes in bytes as people read them, shared by the apps (the tour's error
// cards, Tools' maintenance and Updates pages, Parental controls' error
// cards), so that a size looks the same everywhere.

use crate::i18n::{language, tr};

/// The units' English texts, for the programs' lists of texts (TEXTS)
pub const TEXTS: &[&str] = &["B", "kB", "MB", "GB", "TB"];

/// Bytes as the file manager shows them: "12.3 GB" (powers of 1000), with
/// the units and the decimal comma of the session language ("12,3 Go")
pub fn size(bytes: u64) -> String {
    let units = [tr("B"), tr("kB"), tr("MB"), tr("GB"), tr("TB")];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1000.0 && unit + 1 < units.len() {
        value /= 1000.0;
        unit += 1;
    }
    let number = if unit == 0 || value >= 100.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    };
    // Every translated language writes 1,5 (English 1.5)
    let number = if language().is_some() {
        number.replace('.', ",")
    } else {
        number
    };
    format!("{number} {}", units[unit])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes() {
        assert_eq!(size(0), "0 B");
        assert_eq!(size(350_000_000), "350 MB");
        assert_eq!(size(9_400_000_000), "9.4 GB");
        assert_eq!(size(238_000_000_000), "238 GB");
        assert_eq!(size(1_500_000_000_000), "1.5 TB");
    }
}
