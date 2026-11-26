// The text styles of cosmic-blue's apps (the tours, Tools, Parental
// controls), so that their rows look alike: a row's title in semibold and
// its caption dimmed, as in COSMIC Settings.

// (Each program uses its part)
#![allow(dead_code)]

use cosmic::cosmic_theme::palette::{Clamp, FromColor, IntoColor, Oklch, Srgb, Srgba};
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
    // Top-aligned: the picture and the title stay at the same place on
    // every page, however many lines the caption has
    widget::row::with_capacity(2)
        .spacing(theme::spacing().space_s)
        .align_y(Alignment::Start)
        .push(picture)
        .push(lines)
        .width(Length::Fill)
        .into()
}

/// A tile (the tour's origin cards, the kinds of apps): a settings-row
/// background, an accent border when chosen, a little lighter under the
/// mouse (a custom style is told whether
/// the button has the keyboard focus, not whether it is selected)
pub fn tile_style(selected: bool) -> theme::Button {
    fn style(selected: bool, theme: &cosmic::Theme, hover: bool) -> cosmic::widget::button::Style {
        let cosmic = theme.cosmic();
        let component = &theme.current_container().component;
        let mut style = cosmic::widget::button::Style::new();
        style.background = Some(cosmic::iced::Background::Color(
            if hover {
                component.hover
            } else {
                component.base
            }
            .into(),
        ));
        style.border_radius = cosmic.corner_radii.radius_s.into();
        style.text_color = Some(component.on.into());
        style.icon_color = Some(if selected {
            cosmic.accent.base.into()
        } else {
            component.on.into()
        });
        if selected {
            style.border_width = 2.0;
            style.border_color = cosmic.accent.base.into();
        }
        style
    }
    theme::Button::Custom {
        active: Box::new(move |_, theme| style(selected, theme, false)),
        disabled: Box::new(move |theme| style(selected, theme, false)),
        hovered: Box::new(move |_, theme| style(selected, theme, true)),
        pressed: Box::new(move |_, theme| style(selected, theme, true)),
    }
}

/// How round the corners of a person's picture are, as a share of its side:
/// the same shape at every size (9 px at 43 px, like the wallpapers'
/// thumbnails in the tour)
const CORNER: f32 = 0.21;

/// The corner radius of a person's picture `size` pixels wide
fn corner(size: f32) -> f32 {
    (size * CORNER).round()
}

/// A person's picture (a PNG or JPEG file) as a square with rounded corners
/// and a smooth edge, at `size` pixels a side: the image widget's own
/// rounding has a jagged edge. COSMIC shows these pictures round; the login
/// screen keeps them round too. None when the file cannot be read.
pub fn rounded_picture(path: &std::path::Path, size: u32) -> Option<widget::image::Handle> {
    let picture = image::open(path).ok()?;
    let mut picture = picture
        .resize_to_fill(size, size, image::imageops::FilterType::CatmullRom)
        .into_rgba8();
    for (x, y, pixel) in picture.enumerate_pixels_mut() {
        let coverage = rounded_coverage(size as f32, x as f32 + 0.5, y as f32 + 0.5);
        pixel[3] = (f32::from(pixel[3]) * coverage).round() as u8;
    }
    Some(widget::image::Handle::from_rgba(
        size,
        size,
        picture.into_raw(),
    ))
}

/// How much of the pixel centred on (x, y) lies inside a rounded square
/// `size` pixels wide (0 to 1): its signed distance to the edge, so that
/// the edge is smooth
fn rounded_coverage(size: f32, x: f32, y: f32) -> f32 {
    let half = size / 2.0;
    let radius = corner(size);
    let qx = (x - half).abs() - (half - radius);
    let qy = (y - half).abs() - (half - radius);
    let outside = qx.max(0.0).hypot(qy.max(0.0));
    let distance = outside + qx.max(qy).min(0.0) - radius;
    (0.5 - distance).clamp(0.0, 1.0)
}

/// The fill of a round icon button with a white symbol (the toolbox's tiles,
/// as the dock's icons): the accent, made darker when needed so that white
/// keeps a contrast of at least 4:1 on it. The bright dark theme accent
/// (#3a9dff) only gives 2.8:1; it keeps its hue and becomes #1180e0. A
/// darker accent (the light theme's) is used as is.
pub fn fill_under_white(accent: Srgba) -> Srgba {
    let mut oklch: Oklch = Srgb::new(accent.red, accent.green, accent.blue).into_color();
    let mut srgb = accent.color;
    while contrast_with_white(srgb) < MIN_WHITE_CONTRAST && oklch.l > 0.3 {
        oklch.l -= 0.005;
        srgb = Srgb::from_color(oklch).clamp();
    }
    Srgba::new(srgb.red, srgb.green, srgb.blue, accent.alpha)
}

/// Contrast of white text or symbols on a filled icon button
const MIN_WHITE_CONTRAST: f32 = 4.0;

/// WCAG contrast ratio between white and a color
fn contrast_with_white(color: Srgb) -> f32 {
    let luminance = |c: f32| {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let l = 0.2126 * luminance(color.red)
        + 0.7152 * luminance(color.green)
        + 0.0722 * luminance(color.blue);
    1.05 / (l + 0.05)
}

/// A button for a person's picture, `size` pixels wide: the selected (or
/// focused) one outlined in the accent color, a fainter outline on hover,
/// drawn over the picture's edge in its rounded shape, as the image buttons
/// of the wallpapers
pub fn picture_button<'a, M: Clone + 'a>(
    handle: widget::image::Handle,
    size: u16,
    selected: bool,
    on_press: M,
) -> widget::Button<'a, M> {
    let ring = move |width: f32, alpha: f32, theme: &cosmic::Theme| {
        let mut style = widget::button::Style::new();
        style.border_radius = corner(f32::from(size)).into();
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
        .padding(0)
        .on_press(on_press)
}

#[cfg(test)]
mod tests {
    use super::{contrast_with_white, fill_under_white, rounded_coverage};
    use cosmic::cosmic_theme::palette::{IntoColor, Oklch, Srgb, Srgba};

    fn hex(color: Srgba) -> String {
        let c: Srgba<u8> = color.into_format();
        format!("#{:02x}{:02x}{:02x}", c.red, c.green, c.blue)
    }

    #[test]
    fn icon_fill_keeps_white_readable() {
        // The dark theme accent gets darker, same hue, white at 4:1
        let accent = Srgba::new(0x3a as f32 / 255.0, 0x9d as f32 / 255.0, 1.0, 1.0);
        let fill = fill_under_white(accent);
        let ratio = contrast_with_white(fill.color);
        assert!((4.0..4.3).contains(&ratio), "{} {ratio}", hex(fill));
        let before: Oklch = Srgb::new(accent.red, accent.green, accent.blue).into_color();
        let after: Oklch = fill.color.into_color();
        let hue = (before.hue.into_degrees() - after.hue.into_degrees()).abs();
        assert!(hue < 3.0, "hue moved by {hue}");
        // The light theme accent is dark enough: unchanged
        let light = Srgba::new(0.0, 0x56 as f32 / 255.0, 0xb2 as f32 / 255.0, 1.0);
        assert_eq!(hex(fill_under_white(light)), "#0056b2");
    }

    #[test]
    fn rounded_square_edges() {
        // The middle and the middle of each side are covered
        assert_eq!(rounded_coverage(43.0, 21.5, 21.5), 1.0);
        assert_eq!(rounded_coverage(43.0, 0.5, 21.5), 1.0);
        assert_eq!(rounded_coverage(43.0, 21.5, 42.5), 1.0);
        // The corner pixel is outside, the rounding is symmetric
        assert_eq!(rounded_coverage(43.0, 0.5, 0.5), 0.0);
        assert_eq!(
            rounded_coverage(43.0, 2.5, 4.5),
            rounded_coverage(43.0, 40.5, 38.5)
        );
        // The edge of the rounding is partly covered (smooth)
        let edge = rounded_coverage(43.0, 1.5, 4.5);
        assert!(edge > 0.0 && edge < 1.0, "{edge}");
    }
}
