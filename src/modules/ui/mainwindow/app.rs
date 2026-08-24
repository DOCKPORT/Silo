//! App: the Silo application shell.
//!
//! Boots the application, listens for window resize events, and runs the
//! native OS helpers (folder picker, file explorer). The message dispatch and
//! window drawing stay in [`super`] (the `mod.rs` coordinator).

use std::path::{Path, PathBuf};

use iced::window::Position;
use iced::{Subscription, Task, application};

use crate::modules::{config, silo_size};

use super::app_icon;
use super::{Message, SiloApp, update, view};
use crate::modules::ui::font;
use crate::modules::ui::scaling::Scaling;
use crate::modules::ui::theme::silo_theme;

/// Boots the Silo application.
///
/// Returns the initial state and a no-op [`Task`]. The saved settings are
/// read once from the database, so the folder list, excludes, destination,
/// and populated flag are live from the first frame. Reading again on every
/// frame would hit the database each redraw.
fn new() -> (SiloApp, Task<Message>) {
    // Load the saved settings once at startup, so the Sync dialog builds its
    // plan from the real source folders. Without this, the folder list stays
    // empty until the Config dialog loads the rows.
    let mut app = SiloApp {
        silo_size: "--".to_string(),
        ..SiloApp::default()
    };
    match config::load() {
        Ok(settings) => {
            app.config.folder_paths = settings.silo_data_paths;
            app.config.exclude_patterns = settings.excludes;
            app.sync.rsync_dest_path = settings.rsync_dest_path;
        }
        Err(err) => {
            eprintln!("silo: could not load the saved settings: {err}");
        }
    };
    (
        app,
        // Compute the total silo size in the background, so the SILO SIZE
        // label is live from the first frame.
        silo_size_task(),
    )
}

/// Spawns a background task that computes the total silo size.
///
/// The task reads the current settings from the database, so it reflects the
/// latest folders and exclude patterns. The result maps to
/// `Message::SiloSizeComputed`; an unreadable source folder maps to `N/A`.
pub(super) fn silo_size_task() -> Task<Message> {
    Task::perform(
        async { silo_size::silo_size_label() },
        Message::SiloSizeComputed,
    )
}

/// Sets a boolean hover flag and returns a no-op task.
pub(super) fn set_hovered(slot: &mut bool, hovered: bool) -> Task<Message> {
    *slot = hovered;
    Task::none()
}

/// Opens the OS native folder picker with the given title.
///
/// The picked folder maps through `map`, which the caller uses to produce
/// `Message::Config(ConfigMsg::FolderPicked)` or
/// `Message::Sync(SyncMsg::DestFolderPicked)`.
pub(super) fn pick_folder(
    title: &'static str,
    map: impl Fn(Option<PathBuf>) -> Message + Send + 'static,
) -> Task<Message> {
    Task::perform(
        rfd::AsyncFileDialog::new().set_title(title).pick_folder(),
        move |file| map(file.map(|handle| handle.path().to_path_buf())),
    )
}

/// Opens a folder in the native OS file explorer.
pub(super) fn open_in_file_explorer(path: &Path) {
    if let Err(err) = std::process::Command::new("xdg-open").arg(path).spawn() {
        eprintln!("silo: could not open the folder {path:?}: {err}");
    }
}

/// The application's subscriptions.
///
/// Listens for window resize events so the scale factor stays in sync with the
/// live client area.
fn subscription(_state: &SiloApp) -> Subscription<Message> {
    iced::window::resize_events().map(|(_id, size)| Message::WindowResized(size))
}

/// Runs the Silo main window.
pub fn run() -> iced::Result {
    let screen_size = Scaling::global().screen_size;

    let window_settings = iced::window::Settings {
        size: screen_size,
        position: Position::Centered,
        maximized: true,
        icon: app_icon::load_app_icon(),
        ..iced::window::Settings::default()
    };

    application(new, update, view)
        .font(font::FONT_BYTES)
        .default_font(font::FONT)
        .theme(silo_theme())
        .title("Silo")
        .window(window_settings)
        .subscription(subscription)
        .run()
}
