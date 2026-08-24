//! AppIcon: the Silo window icon.
//!
//! Rasterizes the embedded `logo/silo_logo_icon.svg` at startup and converts
//! the pixels into the icon format expected by the window system.

/// The app icon SVG, embedded in the binary at compile time so the running
/// program does not depend on any path on disk.
const ICON_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/logo/silo_logo_icon.svg"
));

/// Rasterize the SVG app icon into the window icon.
///
/// Renders the embedded icon at a high resolution and converts the pixels
/// into the icon format expected by the window system.
///
/// Returns `None` when the SVG cannot be parsed or rendered; the application
/// then falls back to the platform default icon.
pub fn load_app_icon() -> Option<iced::window::Icon> {
    const ICON_SIZE: u32 = 256;

    let mut opt = resvg::usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();
    let tree = resvg::usvg::Tree::from_data(ICON_SVG, &opt).ok()?;

    let src_size = tree.size();
    let (src_w, src_h) = (src_size.width(), src_size.height());
    if src_w <= 0.0 || src_h <= 0.0 {
        return None;
    }

    // Scale the source viewbox down to the target icon size.
    let scale = ICON_SIZE as f32 / src_w.max(src_h);
    let mut pixmap = resvg::tiny_skia::Pixmap::new(ICON_SIZE, ICON_SIZE)?;
    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // tiny-skia stores premultiplied alpha; iced expects straight RGBA.
    let premult = pixmap.data();
    let mut rgba = Vec::with_capacity(premult.len());
    for px in premult.chunks_exact(4) {
        let (r, g, b, a) = (px[0], px[1], px[2], px[3]);
        if a == 0 {
            rgba.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            rgba.extend_from_slice(&[
                (u16::from(r) * 255 / u16::from(a)) as u8,
                (u16::from(g) * 255 / u16::from(a)) as u8,
                (u16::from(b) * 255 / u16::from(a)) as u8,
                a,
            ]);
        }
    }

    iced::window::icon::from_rgba(rgba, ICON_SIZE, ICON_SIZE).ok()
}
