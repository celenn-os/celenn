// The text styles of cosmic-blue's apps (the tours, Tools, Parental
// controls), so that their rows look alike: a row's title in semibold and
// its caption dimmed, as in COSMIC Settings.

// (Each program uses its part)
#![allow(dead_code)]

use cosmic::theme;
use cosmic::widget::{self, text};
use std::borrow::Cow;

/// A row's title
pub fn row_title<'a>(title: impl Into<Cow<'a, str>> + 'a) -> widget::Text<'a, cosmic::Theme> {
    text::heading(title)
}

/// A row's caption, dimmed under its title
pub fn dim_caption<'a>(caption: impl Into<Cow<'a, str>> + 'a) -> widget::Text<'a, cosmic::Theme> {
    text::caption(caption).class(theme::Text::Custom(|theme| {
        let mut color = theme.cosmic().on_bg_color();
        color.alpha = 0.72;
        cosmic::iced::widget::text::Style {
            color: Some(color.into()),
            ..Default::default()
        }
    }))
}

/// The top of a page, as in COSMIC Settings: its picture (an icon in the
/// style of the dock's buttons, or a person's picture), its title and a
/// dimmed caption, on the left, so that the page's rows come right after
pub fn page_header<'a, M: 'static>(
    picture: cosmic::Element<'a, M>,
    title: impl Into<Cow<'a, str>> + 'a,
    caption: Option<String>,
) -> cosmic::Element<'a, M> {
    use cosmic::iced::core::text::Wrapping;
    use cosmic::iced::{Alignment, Length};
    let lines = widget::column::with_capacity(2)
        .spacing(2)
        .push(text::title3(title))
        .push_maybe(caption.map(|caption| dim_caption(caption).wrapping(Wrapping::Word)))
        .width(Length::Fill);
    widget::row::with_capacity(2)
        .spacing(theme::spacing().space_s)
        .align_y(Alignment::Center)
        .push(picture)
        .push(lines)
        .width(Length::Fill)
        .into()
}
