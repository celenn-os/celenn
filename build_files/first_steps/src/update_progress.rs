// Progress of a system download, for the progress bar of the Updates page.
//
// celenn-bootc (the bootc that uupd and celenn-updates-helper run)
// copies bootc's progress, JSON lines written to its --progress-fd, to
// PROGRESS. Each line is an event: "ProgressBytes" while the image is
// downloaded (task "pulling": bytes and bytesTotal of what is left to
// download), then "ProgressSteps" while it is imported and staged. Only the
// last complete line matters. The file is removed when bootc ends.

use crate::i18n::tr;
use crate::units::size;
use std::time::{Duration, SystemTime};

pub const PROGRESS: &str = "/run/celenn/update-progress.jsonl";
/// bootc writes an event at least every few seconds while it downloads;
/// an older file was left by a download that was stopped
const FRESH: Duration = Duration::from_secs(60);

pub const TEXTS: &[&str] = &[
    "Downloading the new version: {done} of {total} ({percent}%)",
    "Installing the new version…",
];

#[derive(Clone, Debug, PartialEq)]
pub enum Progress {
    /// Bytes downloaded and to download
    Download { done: u64, total: u64 },
    /// Import and staging of the downloaded image: steps done and total
    Install { done: u64, total: u64 },
}

impl Progress {
    /// Share done, from 0 to 1 (None when unknown)
    pub fn fraction(&self) -> Option<f32> {
        let (Self::Download { done, total } | Self::Install { done, total }) = *self;
        (total > 0).then(|| (done.min(total) as f64 / total as f64) as f32)
    }

    /// What the bar shows under it, in the session language
    pub fn text(&self) -> String {
        match *self {
            Self::Download { done, total } => {
                let percent = self.fraction().map_or(0, |f| (f * 100.0).floor() as u32);
                tr("Downloading the new version: {done} of {total} ({percent}%)")
                    .replace("{done}", &size(done))
                    .replace("{total}", &size(total))
                    .replace("{percent}", &percent.to_string())
            }
            Self::Install { .. } => tr("Installing the new version…").to_owned(),
        }
    }
}

/// The last event of a download still running (the file changed less than a
/// minute ago), or None
pub fn read() -> Option<Progress> {
    let file = std::fs::File::open(PROGRESS).ok()?;
    let modified = file.metadata().ok()?.modified().ok()?;
    if SystemTime::now()
        .duration_since(modified)
        .unwrap_or_default()
        > FRESH
    {
        return None;
    }
    parse_last(&std::fs::read_to_string(PROGRESS).ok()?)
}

/// The last event of the file that says where the download is
pub fn parse_last(lines: &str) -> Option<Progress> {
    lines.lines().rev().find_map(parse)
}

fn parse(line: &str) -> Option<Progress> {
    let event: serde_json::Value = serde_json::from_str(line).ok()?;
    let number = |key: &str| event[key].as_u64().unwrap_or(0);
    match event["type"].as_str()? {
        "ProgressBytes" if number("bytesTotal") > 0 => Some(Progress::Download {
            done: number("bytes"),
            total: number("bytesTotal"),
        }),
        "ProgressSteps" => Some(Progress::Install {
            done: number("steps"),
            total: number("stepsTotal"),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_event() {
        let lines = concat!(
            r#"{"type":"Start","version":"0.1"}"#,
            "\n",
            r#"{"type":"ProgressBytes","task":"pulling","bytes":500000000,"bytesTotal":2000000000,"steps":1,"stepsTotal":5}"#,
            "\n",
            r#"{"type":"ProgressBytes","task":"pulling","bytes":1500000000,"bytesTotal":2000000000,"steps":3,"stepsTotal":5}"#,
            "\n",
            r#"{"type":"ProgressByt"#,
        );
        let progress = parse_last(lines).unwrap();
        assert_eq!(
            progress,
            Progress::Download {
                done: 1_500_000_000,
                total: 2_000_000_000
            }
        );
        assert_eq!(progress.fraction(), Some(0.75));
        let steps = r#"{"type":"ProgressSteps","task":"staging","steps":1,"stepsTotal":2}"#;
        assert_eq!(
            parse_last(steps),
            Some(Progress::Install { done: 1, total: 2 })
        );
        assert_eq!(parse_last(""), None);
    }
}
