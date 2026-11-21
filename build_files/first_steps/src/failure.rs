// When a change made by a button does not work: a card that stays at the
// top of the page until it is closed (a toast goes away before it is read).
// It says what did not work and why, in plain words, and what to do:
// retry, open the network settings, or report the problem. Shared by
// cosmic-blue Tools and Parental controls.
//
// The cause is guessed from what the command wrote on its error output
// (the technical details, shown folded in the card) and from the state of
// the computer: no network, a full disk, a server that does not answer.
//
// "Report a problem" first says what the report holds, in the card, then
// runs /usr/libexec/cosmic-blue-report-bug (the script of `ujust
// report-bug`) with the error added to the system details: the form opens in
// the browser, filled in, and nothing is sent until it is submitted there.

use crate::i18n::tr;
use cosmic::iced::core::text::Wrapping;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, text};
use cosmic::{Apply, Element, theme};
use std::process::Stdio;

const REPORT: &str = "/usr/libexec/cosmic-blue-report-bug";
/// Lines of the error output kept for the details and the report
const DETAIL_LINES: usize = 12;

pub const TEXTS: &[&str] = &[
    "“{}” did not work",
    "No internet connection. Connect to a network, then try again. Nothing was changed.",
    "The disk is full ({} free). Free up space with Clean up, in cosmic-blue Tools, Maintenance, then try again.",
    "The server does not answer. Try again in a few minutes.",
    "Something unexpected went wrong. Nothing was changed.",
    "Try again",
    "Network settings",
    "Report the problem",
    "Technical details",
    "The report opens in your browser, filled in: the cosmic-blue version, the hardware and this error. No files or personal data. You read it before sending it; write in your own language. Sending needs a GitHub account.",
    "Open the report",
    "Cancel",
    "Close",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    Offline,
    DiskFull,
    Server,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    pub cause: Cause,
    /// The last lines of the error output, and the command's exit status
    pub details: String,
}

/// What a command gave: done, cancelled (the password prompt dismissed, or
/// the password wrong three times: pkexec's 126 and 127), or a failure
pub type Outcome = Option<Result<(), Failure>>;

/// Runs a command in the background and keeps what it said on failure.
/// `input` goes to its standard input when given.
pub async fn run(program: &str, args: Vec<String>, input: Option<String>) -> Outcome {
    use tokio::io::AsyncWriteExt;
    let child = tokio::process::Command::new(program)
        .args(&args)
        .stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn();
    let mut child = match child {
        Ok(child) => child,
        Err(error) => {
            eprintln!("Could not start {program}: {error}");
            return Some(Err(failure(&format!("{program}: {error}"))));
        }
    };
    if let (Some(input), Some(mut stdin)) = (input, child.stdin.take()) {
        let _ = stdin.write_all(input.as_bytes()).await;
    }
    match child.wait_with_output().await {
        Ok(output) if output.status.success() => Some(Ok(())),
        // pkexec, or a script that ran it and passed its exit code on
        Ok(output) if matches!(output.status.code(), Some(126 | 127)) => None,
        Ok(output) => {
            let error = String::from_utf8_lossy(&output.stderr);
            eprintln!("{program} {args:?}: {}\n{error}", output.status);
            Some(Err(failure(&format!("{error}\n{program}: {}", output.status))))
        }
        Err(error) => {
            eprintln!("{program}: {error}");
            Some(Err(failure(&format!("{program}: {error}"))))
        }
    }
}

/// The last lines of an error output, and why it most likely failed
pub fn failure(error: &str) -> Failure {
    let lines: Vec<&str> = error.lines().filter(|line| !line.trim().is_empty()).collect();
    let details = lines[lines.len().saturating_sub(DETAIL_LINES)..].join("\n");
    let cause = cause(error, online(), free_space());
    Failure { cause, details }
}

fn cause(error: &str, online: Option<bool>, free: Option<u64>) -> Cause {
    let error = error.to_lowercase();
    let has = |words: &[&str]| words.iter().any(|word| error.contains(word));
    if has(&["no space left", "not enough disk space", "disk quota"])
        || free.is_some_and(|free| free < 300_000_000)
    {
        Cause::DiskFull
    } else if online == Some(false)
        || has(&[
            "could not resolve",
            "temporary failure in name resolution",
            "network is unreachable",
            "name or service not known",
            "no route to host",
        ])
    {
        Cause::Offline
    } else if has(&[
        "timed out",
        "timeout",
        "503",
        "502",
        "504",
        "500 internal",
        "service unavailable",
        "connection refused",
        "connection reset",
    ]) {
        Cause::Server
    } else {
        Cause::Other
    }
}

/// NetworkManager's connectivity: None when unknown
fn online() -> Option<bool> {
    let output = std::process::Command::new("nmcli")
        .args(["networking", "connectivity"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    match String::from_utf8_lossy(&output.stdout).trim() {
        "full" | "limited" | "portal" => Some(true),
        "none" => Some(false),
        _ => None,
    }
}

/// Free space where apps and the system's data are (/var)
fn free_space() -> Option<u64> {
    let output = std::process::Command::new("df")
        .args(["--output=avail", "-B1", "/var"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .nth(1)?
        .trim()
        .parse()
        .ok()
}

fn size(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.1} GB", bytes as f64 / 1e9)
    } else {
        format!("{} MB", bytes / 1_000_000)
    }
}

#[derive(Clone, Debug)]
pub enum CardMessage {
    Retry,
    Network,
    /// Report the problem: first what the report holds, then open it
    Report,
    ReportOpen,
    ReportCancel,
    Details,
    Close,
}

/// A failure shown on a page, and what the person opened in its card
#[derive(Clone, Debug)]
pub struct Card<J> {
    /// What did not work: the button's row title, translated
    title: String,
    failure: Failure,
    /// What "Try again" runs again
    pub job: J,
    reporting: bool,
    details: bool,
}

/// What the page does after a message of the card
pub enum CardAction<J> {
    None,
    Retry(J),
    Close,
}

impl<J: Clone> Card<J> {
    pub fn new(title: &str, failure: Failure, job: J) -> Self {
        Card {
            title: title.to_owned(),
            failure,
            job,
            reporting: false,
            details: false,
        }
    }

    pub fn update(&mut self, message: CardMessage) -> CardAction<J> {
        match message {
            CardMessage::Retry => return CardAction::Retry(self.job.clone()),
            CardMessage::Network => spawn("cosmic-settings", &["network"], None),
            CardMessage::Report => self.reporting = true,
            CardMessage::ReportCancel => self.reporting = false,
            CardMessage::ReportOpen => {
                self.reporting = false;
                let error = format!("{}\n{}", self.title, self.failure.details);
                spawn(REPORT, &["open"], Some(("COSMIC_BLUE_REPORT_ERROR", &error)));
            }
            CardMessage::Details => self.details = !self.details,
            CardMessage::Close => return CardAction::Close,
        }
        CardAction::None
    }

    pub fn view<'a, M: Clone + 'static>(
        &'a self,
        map: impl Fn(CardMessage) -> M + 'a,
    ) -> Element<'a, M> {
        let space = theme::spacing();
        let reason = match self.failure.cause {
            Cause::Offline => tr(
                "No internet connection. Connect to a network, then try again. Nothing was changed.",
            )
            .to_owned(),
            Cause::DiskFull => tr(
                "The disk is full ({} free). Free up space with Clean up, in cosmic-blue Tools, Maintenance, then try again.",
            )
            .replacen("{}", &free_space().map(size).unwrap_or_default(), 1),
            Cause::Server => tr("The server does not answer. Try again in a few minutes.").to_owned(),
            Cause::Other => tr("Something unexpected went wrong. Nothing was changed.").to_owned(),
        };
        let title = widget::row::with_capacity(3)
            .spacing(space.space_s)
            .align_y(Alignment::Center)
            .push(
                widget::icon::from_name("dialog-warning-symbolic")
                    .size(24)
                    .apply(widget::container)
                    .class(theme::Container::custom(|theme| {
                        cosmic::iced::widget::container::Style {
                            icon_color: Some(theme.cosmic().warning_color().into()),
                            ..Default::default()
                        }
                    })),
            )
            .push(
                text::heading(tr("“{}” did not work").replacen("{}", &self.title, 1))
                    .wrapping(Wrapping::Word)
                    .width(Length::Fill),
            )
            .push(
                widget::button::icon(widget::icon::from_name("window-close-symbolic"))
                    .tooltip(tr("Close"))
                    .on_press(map(CardMessage::Close)),
            );
        let mut buttons = widget::row::with_capacity(3)
            .spacing(space.space_xs)
            .push(widget::button::suggested(tr("Try again")).on_press(map(CardMessage::Retry)));
        if self.failure.cause == Cause::Offline {
            buttons = buttons.push(
                widget::button::standard(tr("Network settings")).on_press(map(CardMessage::Network)),
            );
        }
        // A full disk or no network is not a bug
        if matches!(self.failure.cause, Cause::Other | Cause::Server) {
            buttons = buttons.push(
                widget::button::standard(tr("Report the problem"))
                    .on_press(map(CardMessage::Report)),
            );
        }
        let mut column = widget::column::with_capacity(5)
            .spacing(space.space_xs)
            .push(title)
            .push(text::body(reason).wrapping(Wrapping::Word));
        if self.reporting {
            column = column.push(
                widget::column::with_capacity(2)
                    .spacing(space.space_xs)
                    .push(
                        text::caption(tr("The report opens in your browser, filled in: the cosmic-blue version, the hardware and this error. No files or personal data. You read it before sending it; write in your own language. Sending needs a GitHub account."))
                            .wrapping(Wrapping::Word),
                    )
                    .push(
                        widget::row::with_capacity(3)
                            .spacing(space.space_xs)
                            .push(widget::space::horizontal())
                            .push(
                                widget::button::standard(tr("Cancel"))
                                    .on_press(map(CardMessage::ReportCancel)),
                            )
                            .push(
                                widget::button::suggested(tr("Open the report"))
                                    .on_press(map(CardMessage::ReportOpen)),
                            ),
                    )
                    .apply(widget::container)
                    .padding(space.space_s)
                    .class(theme::Container::Card),
            );
        } else {
            column = column.push(buttons);
        }
        if !self.failure.details.is_empty() {
            column = column.push(
                widget::button::text(tr("Technical details"))
                    .trailing_icon(widget::icon::from_name(if self.details {
                        "go-up-symbolic"
                    } else {
                        "go-down-symbolic"
                    }))
                    .on_press(map(CardMessage::Details)),
            );
            if self.details {
                column = column.push(
                    text::monotext(self.failure.details.as_str())
                        .size(12)
                        .wrapping(Wrapping::WordOrGlyph),
                );
            }
        }
        column
            .padding(space.space_s)
            .width(Length::Fill)
            .apply(widget::container)
            .class(theme::Container::custom(|theme| {
                let cosmic = theme.cosmic();
                let warning: cosmic::iced::Color = cosmic.warning_color().into();
                cosmic::iced::widget::container::Style {
                    background: Some(
                        cosmic::iced::Color { a: 0.12, ..warning }.into(),
                    ),
                    border: cosmic::iced::Border {
                        color: cosmic::iced::Color { a: 0.5, ..warning },
                        width: 1.0,
                        radius: cosmic.corner_radii.radius_s.into(),
                    },
                    ..Default::default()
                }
            }))
            .into()
    }
}

fn spawn(program: &str, args: &[&str], env: Option<(&str, &str)>) {
    let mut command = std::process::Command::new(program);
    command.args(args);
    if let Some((key, value)) = env {
        command.env(key, value);
    }
    if let Err(error) = command.spawn() {
        eprintln!("Could not start {program}: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn causes() {
        let flathub = "error: Unable to load summary from remote flathub: Could not resolve hostname";
        assert_eq!(cause(flathub, None, None), Cause::Offline);
        assert_eq!(cause("anything", Some(false), None), Cause::Offline);
        assert_eq!(cause("write: No space left on device", Some(true), None), Cause::DiskFull);
        assert_eq!(cause("anything", Some(true), Some(100_000_000)), Cause::DiskFull);
        assert_eq!(cause("HTTP 503 Service Unavailable", Some(true), None), Cause::Server);
        assert_eq!(cause("exit status: 1", Some(true), Some(50_000_000_000)), Cause::Other);
    }

    #[test]
    fn details_keep_the_last_lines() {
        let error: String = (1..=20).map(|n| format!("line {n}\n")).collect();
        let details = failure(&error).details;
        assert!(details.starts_with("line 9\n"));
        assert!(details.ends_with("line 20"));
    }
}
