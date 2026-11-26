//! Writes the cosmic-blue default COSMIC themes (dark and light) with the
//! frosted glass effect turned on, to $XDG_CONFIG_HOME/cosmic.
//!
//! COSMIC apps read the computed theme (com.system76.CosmicTheme.*), not the
//! builder that COSMIC Settings edits, and the frosted glass switches change
//! the computed colors (transparency). So both are generated with the same
//! code as COSMIC Settings, and shipped as system defaults in /usr/share/cosmic.
//!
//! The colors revolve around the "cosmic" blue of the logo and wallpaper:
//! - the accent is a blue from the logo instead of COSMIC's default teal;
//! - the window background is a deep night blue instead of neutral gray, and
//!   the controls get a slight tint of the same hue (COSMIC derives every
//!   surface from these, keeping hue and chroma, so they stay discreet).
//!
//! The window hint (the outline of the active window) is left unset, so it
//! follows the accent, also when the user picks another accent in COSMIC
//! Settings. Every color can still be changed there (Appearance).

use cosmic_config::{ConfigSet, CosmicConfigEntry};
use cosmic_theme::palette::{Srgb, Srgba};
use cosmic_theme::{AlphaMap, BlurStrength, Theme, ThemeBuilder};

/// Dark theme accent: a sky blue between the cyan and the blue of the logo
/// orbit gradient (#3a9dff).
const ACCENT_DARK: Srgb<u8> = Srgb::new(0x3a, 0x9d, 0xff);
/// Dark theme accent of earlier images (until 2026-11-26): the middle of the
/// logo orbit gradient.
const ACCENT_DARK_4F8DFF: Srgb<u8> = Srgb::new(0x4f, 0x8d, 0xff);
/// Light theme accent: the dark theme accent's hue (oklch 252) made dark
/// enough to stay readable on light surfaces (4.9:1 on the window
/// background), so that both modes share one blue.
const ACCENT_LIGHT: Srgb<u8> = Srgb::new(0x00, 0x56, 0xb2);
/// Light theme accent of earlier images (until 2026-11-26): the deep blue
/// of the logo planet, a more violet hue (265) than the dark accent.
const ACCENT_LIGHT_1D47C4: Srgb<u8> = Srgb::new(0x1d, 0x47, 0xc4);

/// Width of the outline of the active window, in pixels. COSMIC's 3 px
/// outline in the bright accent frames every window heavily over the blue
/// wallpaper; 1 px still shows which window is active. COSMIC Settings
/// (Desktop > Window management) can change it.
const ACTIVE_HINT: u32 = 1;
/// Outline width of earlier images (COSMIC's default, until 2026-11-26)
const ACTIVE_HINT_3: u32 = 3;

/// Window background, dark theme: night blue at the lightness of COSMIC's
/// default dark gray (oklch 0.225 0.03, hue of the accent).
const BG_DARK: Srgb<u8> = Srgb::new(0x14, 0x1c, 0x2a);
/// Window background, light theme: a cool, barely blue gray at the lightness
/// of COSMIC's default light gray (oklch 0.88 0.012).
const BG_LIGHT: Srgb<u8> = Srgb::new(0xd3, 0xd8, 0xe0);
/// Tint of buttons and other controls (oklch chroma 0.03, hue of the accent;
/// COSMIC only keeps its hue and chroma).
const NEUTRAL_TINT: Srgb<u8> = Srgb::new(0x5a, 0x64, 0x75);

fn colors(
    mut builder: ThemeBuilder,
    accent: Srgb<u8>,
    bg: Srgb<u8>,
    active_hint: u32,
) -> ThemeBuilder {
    let bg: Srgb = bg.into_format();
    builder.active_hint = active_hint;
    builder
        .accent(accent.into_format())
        .bg_color(Srgba::new(bg.red, bg.green, bg.blue, 1.0))
        .neutral_tint(NEUTRAL_TINT.into_format())
}

/// The same switches as "Frosted glass" in COSMIC's first-run assistant.
fn frosted(mut builder: ThemeBuilder) -> ThemeBuilder {
    builder.frosted_windows = true;
    builder.frosted_system_interface = true;
    builder.frosted_panel = true;
    builder.frosted_applets = true;
    builder
}

/// Frosted glass settings of COSMIC Settings (Appearance > Frosted glass).
///
/// The default wallpaper is a fairly even blue with thin bright arcs, close
/// to the night blue of windows. With COSMIC's defaults (Thickness Medium,
/// Glass opacity 0.5, about 75 % opaque) the blur erases the arcs and the
/// surface looks plain, not like glass. So:
/// - "Thickness" Low: the arcs stay visible as soft lines behind windows,
///   the panel and the dock, which is what makes the glass readable;
/// - "Glass opacity" 0.2 in the dark theme (surfaces about 45 % opaque,
///   headers and cards about 75 %), 0.35 in the light theme (about 63 %),
///   which keeps dark text readable over the dark wallpaper.
///
/// The panel and the dock use the theme's background, so the same values
/// give windows, panel and dock the same glass.
const THICKNESS: BlurStrength = BlurStrength::Low;
const GLASS_OPACITY_DARK: f32 = 0.2;
const GLASS_OPACITY_LIGHT: f32 = 0.35;

/// Frosted glass of earlier images: (thickness, glass opacity)
const GLASS_0_8: (BlurStrength, f32) = (BlurStrength::High, 0.8); // until 2026-11-19
const GLASS_0_65: (BlurStrength, f32) = (BlurStrength::High, 0.65); // until 2026-11-21

/// The alpha map COSMIC Settings writes for a "Glass opacity" value (same
/// formula as cosmic-settings 1.11.0, appearance/mod.rs, Message::Glass).
fn glass_alpha_map(glass: f32) -> AlphaMap {
    let alpha_map = AlphaMap::default();
    let offset = (glass - 0.5) / 0.5
        * if glass < 0.5 {
            alpha_map.extremely_high_2
        } else {
            1.0 - alpha_map.extremely_low
        };
    alpha_map.offset(offset)
}

fn frosted_glass(mut builder: ThemeBuilder, thickness: BlurStrength, glass: f32) -> ThemeBuilder {
    builder.frosted = thickness;
    builder.alpha_map = glass_alpha_map(glass);
    builder
}

/// What changed between the cosmic-blue themes of successive images
struct Look {
    accent_dark: Srgb<u8>,
    accent_light: Srgb<u8>,
    active_hint: u32,
    thickness: BlurStrength,
    glass_dark: f32,
    glass_light: f32,
}

/// The current cosmic-blue themes
const CURRENT: Look = Look {
    accent_dark: ACCENT_DARK,
    accent_light: ACCENT_LIGHT,
    active_hint: ACTIVE_HINT,
    thickness: THICKNESS,
    glass_dark: GLASS_OPACITY_DARK,
    glass_light: GLASS_OPACITY_LIGHT,
};

/// The themes of earlier images, by the name of their variant
/// (until 2026-11-26: light accent #1d47c4 and a 3 px active window outline)
const UNTIL_20261126: Look = Look {
    accent_light: ACCENT_LIGHT_1D47C4,
    active_hint: ACTIVE_HINT_3,
    ..CURRENT
};

/// The cosmic-blue dark and light themes
fn cosmic_blue(look: &Look) {
    write(
        frosted_glass(
            colors(
                frosted(ThemeBuilder::dark()),
                look.accent_dark,
                BG_DARK,
                look.active_hint,
            ),
            look.thickness,
            look.glass_dark,
        ),
        frosted_glass(
            colors(
                frosted(ThemeBuilder::light()),
                look.accent_light,
                BG_LIGHT,
                look.active_hint,
            ),
            look.thickness,
            look.glass_light,
        ),
    );
    icon_theme();
}

/// The icon theme, which COSMIC 1.12 moved into the theme: up to COSMIC 1.11
/// apps read it from com.system76.CosmicTk (shipped in system_files), since
/// 1.12 from the theme (libcosmic 4c853eb), which would otherwise say
/// "Cosmic". It is set as a config key rather than a ThemeBuilder field, so
/// that this builds with both libcosmic versions (1.11 ignores the key).
const ICON_THEME: &str = "cosmic-blue";

fn icon_theme() {
    for config in [
        ThemeBuilder::dark_config(),
        Theme::dark_config(),
        ThemeBuilder::light_config(),
        Theme::light_config(),
    ] {
        config
            .expect("theme config")
            .set("icon_theme", ICON_THEME)
            .expect("write icon theme");
    }
}

/// Writes a dark and a light theme (builder and computed theme).
fn write(dark: ThemeBuilder, light: ThemeBuilder) {
    dark.write_entry(&ThemeBuilder::dark_config().expect("dark builder config"))
        .expect("write dark builder");
    dark.build()
        .write_entry(&Theme::dark_config().expect("dark theme config"))
        .expect("write dark theme");

    light
        .write_entry(&ThemeBuilder::light_config().expect("light builder config"))
        .expect("write light builder");
    light
        .build()
        .write_entry(&Theme::light_config().expect("light theme config"))
        .expect("write light theme");
}

/// Without an argument: the cosmic-blue themes. With "stock", "frosted",
/// "glass-0.8", "glass-0.65", "accent-4f8dff" or "accent-3a9dff": themes
/// from earlier images,
/// which accounts may still hold as a copy (COSMIC's own default, written by
/// the first-run assistant, the same with frosted glass, the default before
/// the blue colors, the blue themes with earlier frosted glass settings, and
/// the earlier dark accent, the earlier light accent and outline). They go
/// into
/// /usr/share/cosmic-blue/previous-defaults.list, so that
/// cosmic-blue-migrate-defaults can tell them from a user's own choice.
fn main() {
    match std::env::args().nth(1).as_deref() {
        None => cosmic_blue(&CURRENT),
        Some("accent-3a9dff") => cosmic_blue(&UNTIL_20261126),
        Some("accent-4f8dff") => cosmic_blue(&Look {
            accent_dark: ACCENT_DARK_4F8DFF,
            ..UNTIL_20261126
        }),
        Some("glass-0.8") => cosmic_blue(&Look {
            accent_dark: ACCENT_DARK_4F8DFF,
            thickness: GLASS_0_8.0,
            glass_dark: GLASS_0_8.1,
            glass_light: GLASS_0_8.1,
            ..UNTIL_20261126
        }),
        Some("glass-0.65") => cosmic_blue(&Look {
            accent_dark: ACCENT_DARK_4F8DFF,
            thickness: GLASS_0_65.0,
            glass_dark: GLASS_0_65.1,
            glass_light: GLASS_0_65.1,
            ..UNTIL_20261126
        }),
        Some("stock") => write(ThemeBuilder::dark(), ThemeBuilder::light()),
        Some("frosted") => write(
            frosted(ThemeBuilder::dark()),
            frosted(ThemeBuilder::light()),
        ),
        Some(other) => panic!("unknown theme variant {other}"),
    }
}
