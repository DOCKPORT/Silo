//! MainWindow: the Iced main window for Silo.
//!
//! A blank window whose whole surface is painted with the Silo theme
//! background `#161616`. The window shows no content yet; the view
//! returns a full-window [`Container`] that is ready to receive children
//! as the UI grows.

use std::convert::Infallible;

use iced::widget::{container, text};
use iced::window::Position;
use iced::{Length, Size, Task, application};

use super::theme::silo_theme;

/// The Silo application state. Empty for now; it will later hold the
/// silo settings loaded from the config store.
#[derive(Debug, Default)]
pub struct SiloApp;

/// Boots the Silo application.
///
/// Returns the initial state and a no-op [`Task`].
fn new() -> (SiloApp, Task<Infallible>) {
    (SiloApp, Task::none())
}

/// Handles application messages.
///
/// No messages exist yet, so this is a no-op.
fn update(_state: &mut SiloApp, _message: Infallible) -> Task<Infallible> {
    Task::none()
}

/// Builds the application view.
///
/// A full-window [`Container`] with no visible content. The theme's
/// `background` palette color (`#161616`) paints the whole surface.
fn view(_state: &SiloApp) -> iced::Element<'_, Infallible> {
    container(text(""))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Detects the primary monitor's work area or full resolution.
///
/// On Linux this queries `xprop -root _NET_WORKAREA` (the screen area minus
/// taskbar), falling back to `xrandr` for the primary resolution. The returned
/// size is used as the initial window size so the maximized window starts at
/// the real screen size instead of a tiny default.
fn detect_screen_size() -> Size {
    #[cfg(target_os = "linux")]
    {
        // Prefer the work area (screen minus taskbar) so the initial window
        // matches the maximized client area.
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

        // Fall back to the full resolution reported by xrandr.
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

/// Runs the Silo main window.
pub fn run() -> iced::Result {
    let window_settings = iced::window::Settings {
        size: detect_screen_size(),
        position: Position::Centered,
        maximized: true,
        ..iced::window::Settings::default()
    };

    application(new, update, view)
        .theme(silo_theme())
        .title("Silo")
        .window(window_settings)
        .run()
}
