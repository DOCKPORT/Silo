//! Font: the single source of truth for the app-wide typeface.
//!
//! The font files live under `theme/font/IoskeleyMono`. This module declares
//! the family name, the default [`Font`], and the embedded bytes of the
//! Regular weight so the rest of the app never hardcodes a font.

use iced::Font;

/// The family name as embedded in the TTF files.
pub const FAMILY: &str = "Ioskeley Mono";

/// The app-wide default font.
pub const FONT: Font = Font::with_name(FAMILY);

/// The bytes of the Regular weight, embedded at build time for registration.
pub const FONT_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/theme/font/IoskeleyMono/Normal/Hinted/IoskeleyMono-Regular.ttf"
));
