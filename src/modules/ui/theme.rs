//! Silo UI theme: sci-fi post-apocalyptic color palette.
//!
//! Colors come from `theme/color_theme/silo_color_theme`:
//! - background: `#161616`
//! - text: `#6cd5ca`
//! - border/orange: `#FFBF00`
//! - detail: `#3e768b`
//!
//! The Iced [`Palette`] derives the remaining background variants
//! (weak, weaker, base) from the base background color.

use iced::theme::Palette;
use iced::{Color, Theme};

/// Builds a [`Color`] from 8-bit RGB channels.
const fn rgb8(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

/// DARK background: #161616.
pub const BACK: Color = rgb8(0x16, 0x16, 0x16);

/// TEAL text and primary accent: #6cd5ca.
pub const TEAL: Color = rgb8(0x6c, 0xd5, 0xca);

/// ORANGE border accent: #FFBF00.
pub const ORANGE: Color = rgb8(0xFF, 0xBF, 0x00);

/// DETAIL teal-blue: #3e768b.
pub const DETAIL: Color = rgb8(0x3e, 0x76, 0x8b);

/// GREY secondary text: #b8b8b8.
pub const GREY: Color = rgb8(0xb8, 0xb8, 0xb8);

/// The Silo color palette mapped onto the Iced [`Palette`] slots.
pub fn silo_palette() -> Palette {
    Palette {
        background: BACK,
        text: TEAL,
        primary: TEAL,
        success: DETAIL,
        warning: ORANGE,
        danger: rgb8(0xc3, 0x42, 0x3f),
    }
}

/// The custom Silo Iced theme.
///
/// The window background is taken from the `background` palette color,
/// so the whole window surface is painted `#161616`.
pub fn silo_theme() -> Theme {
    Theme::custom("Silo", silo_palette())
}
