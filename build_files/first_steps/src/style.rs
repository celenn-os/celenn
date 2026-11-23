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

/// A person's picture (a PNG or JPEG file) as a disk with a smooth edge, at
/// `size` pixels a side: the image widget's own rounding has a jagged edge.
/// None when the file cannot be read.
pub fn round_picture(path: &std::path::Path, size: u32) -> Option<widget::image::Handle> {
    let picture = image::open(path).ok()?;
    let mut picture = picture
        .resize_to_fill(size, size, image::imageops::FilterType::CatmullRom)
        .into_rgba8();
    let center = size as f32 / 2.0;
    for (x, y, pixel) in picture.enumerate_pixels_mut() {
        let distance =
            ((x as f32 + 0.5 - center).powi(2) + (y as f32 + 0.5 - center).powi(2)).sqrt();
        let coverage = (center - distance + 0.5).clamp(0.0, 1.0);
        pixel[3] = (f32::from(pixel[3]) * coverage).round() as u8;
    }
    Some(widget::image::Handle::from_rgba(
        size,
        size,
        picture.into_raw(),
    ))
}

/// A round button for a person's picture, `size` pixels wide: a ring in the
/// accent color around the selected (or focused) one, a fainter one on hover
pub fn picture_button<'a, M: Clone + 'a>(
    handle: widget::image::Handle,
    size: u16,
    selected: bool,
    on_press: M,
) -> widget::Button<'a, M> {
    let ring = move |width: f32, alpha: f32, theme: &cosmic::Theme| {
        let mut style = widget::button::Style::new();
        style.border_radius = (f32::from(size) / 2.0 + 4.0).into();
        style.border_width = width;
        let mut color: cosmic::iced::Color = theme.cosmic().accent.base.into();
        color.a = alpha;
        style.border_color = color;
        style
    };
    widget::button::custom(widget::image(handle).width(size).height(size))
        .class(theme::Button::Custom {
            active: Box::new(move |focused, theme| {
                ring(if selected || focused { 2.0 } else { 0.0 }, 1.0, theme)
            }),
            disabled: Box::new(move |theme| ring(0.0, 1.0, theme)),
            hovered: Box::new(move |_, theme| ring(2.0, if selected { 1.0 } else { 0.5 }, theme)),
            pressed: Box::new(move |_, theme| ring(2.0, 1.0, theme)),
        })
        .padding(3)
        .on_press(on_press)
}
