// What the pages of cosmic-blue Tools share (tools.rs): the page frame (a
// compact header with the page's icon, its title and purpose), the rows'
// text and symbols, and running a command in the background.

use crate::i18n::tr;
use cosmic::iced::Length;
use cosmic::iced::core::text::Wrapping;
use cosmic::widget;
use cosmic::{Apply, Element, cosmic_theme, theme};
use std::sync::OnceLock;

/// A page: its header as in COSMIC Settings (the page's icon, its title and
/// what it is for, on the left), then its content; it scrolls only when the
/// window is made smaller than its default size
pub fn header<'a, M: 'static>(
    icon: &'static str,
    title: &'static str,
    lead: &'static str,
    content: Element<'a, M>,
) -> Element<'a, M> {
    let cosmic_theme::Spacing {
        space_xxs, space_m, ..
    } = theme::spacing();
    // A child or a standard account: the changes that need root ask for
    // someone else's password, said once under the page's purpose
    let lead = if is_admin() {
        lead.to_owned()
    } else {
        format!(
            "{lead}\n{}",
            tr("Some changes ask for the password of a parent or an administrator.")
        )
    };
    widget::column::with_capacity(5)
        .push(widget::space::vertical().height(space_xxs))
        .push(crate::style::page_header(
            widget::icon::from_name(icon).size(44).into(),
            title,
            Some(lead),
        ))
        .push(widget::space::vertical().height(space_m))
        .push(content)
        .push(widget::space::vertical().height(space_m))
        .max_width(640.0)
        .width(640.0)
        .apply(widget::container)
        .center_x(Length::Fill)
        .apply(widget::scrollable)
        .into()
}

/// Whether the account is an administrator (in the wheel group): its
/// password is the one pkexec asks for
pub fn is_admin() -> bool {
    static ADMIN: OnceLock<bool> = OnceLock::new();
    *ADMIN.get_or_init(|| {
        std::process::Command::new("id")
            .arg("-nG")
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .split_whitespace()
                    .any(|group| group == "wheel")
            })
            // Unknown: no notice
            .unwrap_or(true)
    })
}

/// A translated text with its {} replaced
pub fn fill(text: &str, value: &str) -> String {
    text.replacen("{}", value, 1)
}

/// The symbol in front of a row: in the accent color when it stands out
pub fn state_icon<'a, M: 'static>(name: &'static str, accent: bool) -> Element<'a, M> {
    let icon = widget::icon::from_name(name)
        .size(24)
        .apply(widget::container);
    if accent {
        icon.class(theme::Container::custom(|theme| {
            cosmic::iced::widget::container::Style {
                icon_color: Some(theme.cosmic().accent_color().into()),
                ..Default::default()
            }
        }))
        .into()
    } else {
        icon.into()
    }
}

pub use crate::style::{dim_caption, row_title};

/// A row's text: a first line (usually its title, `row_title`), and a
/// dimmed caption under it
pub fn two_lines<'a, M: 'static>(first: Element<'a, M>, caption: String) -> Element<'a, M> {
    widget::column::with_capacity(2)
        .spacing(2)
        .push(first)
        .push(dim_caption(caption).wrapping(Wrapping::Word))
        .width(Length::Fill)
        .into()
}

/// A row with its title and caption only (no first-line widget)
pub fn label<'a, M: 'static>(title: &'static str, caption: String) -> Element<'a, M> {
    two_lines(row_title(title).into(), caption)
}

/// What the image's screenshot test asks a page to show at once, from
/// COSMIC_BLUE_TOOLS_PREVIEW (tests/vm-lib.sh): "failure" (a failure card on
/// Maintenance) or "developer-tools" (the list open on Extra software)
pub fn preview(what: &str) -> bool {
    std::env::var("COSMIC_BLUE_TOOLS_PREVIEW").is_ok_and(|value| value == what)
}

/// Starts a program without waiting for it (a browser, another app)
pub fn spawn(program: &str, args: &[&str]) {
    if let Err(error) = std::process::Command::new(program).args(args).spawn() {
        eprintln!("Could not start {program}: {error}");
    }
}
