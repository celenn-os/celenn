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
