use iced::Size;
use std::sync::atomic::{AtomicU32, Ordering};

/// The reference resolution that all pixel values are designed for.
/// On a 1920×1080 screen, `sp(value)` returns the same value.
/// On larger/smaller screens, values are scaled proportionally.
const REFERENCE_WIDTH: f32 = 1920.0;
const REFERENCE_HEIGHT: f32 = 1080.0;

/// Compute the scale factor for a given window size.
///
/// Uses width-based scaling, but also ensures height does not get squished by
/// taking the smaller of the two factors.
fn compute_factor(width: f32, height: f32) -> f32 {
    let factor_x = width / REFERENCE_WIDTH;
    let factor_y = height / REFERENCE_HEIGHT;
    factor_x.min(factor_y).max(0.0)
}

/// Detects the primary monitor's resolution.
///
/// Platform-specific:
/// - Linux: queries `xrandr` for the actual resolution
/// - Windows/macOS: returns a placeholder (maximized mode is used instead)
fn detect_screen_size() -> Size {
    #[cfg(target_os = "linux")]
    {
        // Prefer the work area (screen minus taskbar) so the initial scale
        // factor matches the maximized window and avoids a startup jump.
        if let Ok(output) = std::process::Command::new("xprop")
            .args(["-root", "_NET_WORKAREA"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(dim) = stdout.split('=').nth(1) {
                let mut parts = dim.trim().split(',');
                let _x = parts.next();
                let _y = parts.next();
                if let (Some(w), Some(h)) = (parts.next(), parts.next())
                    && let (Ok(w), Ok(h)) = (w.trim().parse::<f32>(), h.trim().parse::<f32>())
                {
                    return Size::new(w, h);
                }
            }
        }

        // Fall back to the full resolution reported by `xrandr`.
        if let Ok(output) = std::process::Command::new("xrandr").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if (line.contains(" primary") || line.contains('*'))
                    && let Some(res) = line.split_whitespace().find(|s| {
                        s.contains('x') && s.chars().all(|c| c.is_ascii_digit() || c == 'x')
                    })
                {
                    let parts: Vec<&str> = res.split('x').collect();
                    if parts.len() == 2
                        && let (Ok(w), Ok(h)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>())
                    {
                        return Size::new(w, h);
                    }
                }
            }
        }
        Size::new(1024.0, 768.0)
    }

    #[cfg(not(target_os = "linux"))]
    {
        Size::new(1024.0, 768.0) // maximized mode handles sizing instead
    }
}

/// The live scale factor and the detected screen size.
///
/// The factor is stored atomically so it can be updated while the application
/// runs, for example when the user resizes the window.
pub struct Scaling {
    /// The detected screen size, used once to set the initial window.
    pub screen_size: Size,
    /// The scale factor (window / reference), stored as `f32` bits.
    factor_bits: AtomicU32,
}

impl Scaling {
    /// Returns the global [`Scaling`] instance, computing it on first call.
    pub fn global() -> &'static Self {
        static INSTANCE: std::sync::OnceLock<Scaling> = std::sync::OnceLock::new();
        INSTANCE.get_or_init(|| {
            let screen_size = detect_screen_size();
            let factor = compute_factor(screen_size.width, screen_size.height);
            Scaling {
                screen_size,
                factor_bits: AtomicU32::new(factor.to_bits()),
            }
        })
    }

    /// Ensures the global scale factor is configured.
    ///
    /// Call this as the very first statement in `main()` so the scale is
    /// resolved before any other startup work (API fetches, window creation).
    /// Idempotent: subsequent calls return immediately.
    pub fn init() {
        Self::global();
    }

    /// The current scale factor.
    fn factor(&self) -> f32 {
        f32::from_bits(self.factor_bits.load(Ordering::Relaxed))
    }

    /// Update the scale factor from the current window size.
    ///
    /// Called on every window resize so `sp` values scale live. A resize that
    /// would produce the same factor as the current one is ignored, so a
    /// spurious first event does not invalidate the freshly-computed layout
    /// (which prevents a startup jump).
    pub fn set_window_size(&self, width: f32, height: f32) {
        let factor = compute_factor(width, height);
        let current = self.factor();
        if (factor - current).abs() < 1e-4 {
            return;
        }
        self.factor_bits.store(factor.to_bits(), Ordering::Relaxed);
    }

    /// Scales a pixel value from the reference resolution to the current size.
    ///
    /// Usage: `sp(400)` returns 400 × `scale_factor`.
    pub fn sp(&self, value: f32) -> f32 {
        value * self.factor()
    }
}

/// Convenience shorthand: scales a pixel value to the current screen.
///
/// Equivalent to `Scaling::global().sp(value)`.
pub fn sp(value: f32) -> f32 {
    Scaling::global().sp(value)
}
