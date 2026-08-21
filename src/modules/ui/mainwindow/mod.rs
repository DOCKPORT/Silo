//! MainWindow: the Iced main window for Silo.
//!
//! A blank window whose whole surface is painted with the Silo theme
//! background `#161616`. The window shows no content yet; the view
//! returns a full-window [`Container`] that is ready to receive children
//! as the UI grows.

mod about_dialog;
mod action_area;
mod config_silo_dialog;
mod config_silo_dialog_elements;
mod config_silo_dialog_folders;
mod sync_progress_bar;
mod sync_silo_dialog;

use std::path::PathBuf;

use iced::widget::{Stack, container, text};
use iced::window::Position;
use iced::{Length, Size, Subscription, Task, application};

use crate::modules::config;

use super::font;
use super::scaling::Scaling;
use super::scanlines;
use super::theme::silo_theme;

/// The Silo application state.
#[derive(Debug, Default)]
pub struct SiloApp {
    /// Whether the pointer is currently over the logo.
    logo_hovered: bool,
    /// Whether the pointer is currently over the CONFIG button.
    config_hovered: bool,
    /// Whether the pointer is currently over the SYNC button.
    sync_hovered: bool,
    /// Whether the pointer is currently over the + button in the folder box.
    plus_hovered: bool,
    /// Whether the About dialog is currently open.
    about_open: bool,
    /// Whether the Config Silo dialog is currently open.
    config_dialog_open: bool,
    /// Whether the Sync Silo dialog is currently open.
    sync_dialog_open: bool,
    /// The selected source folders, loaded once when the Config dialog opens.
    folder_paths: Vec<PathBuf>,
    /// The index of the folder chip the pointer is currently over, if any.
    hovered_chip: Option<usize>,
}

/// Messages that drive the Silo application.
#[derive(Debug, Clone)]
enum Message {
    /// The window was resized to a new size.
    WindowResized(Size),
    /// The pointer entered or left the logo.
    LogoHovered(bool),
    /// The pointer entered or left the CONFIG button.
    ConfigHovered(bool),
    /// The pointer entered or left the SYNC button.
    SyncHovered(bool),
    /// The pointer entered or left the + button in the folder box.
    PlusHovered(bool),
    /// The + button in the folder box was pressed; opens the OS native folder
    /// picker.
    PlusPressed,
    /// The folder picker returned a chosen folder, or `None` if the user
    /// cancelled. A chosen folder is appended to `silo_data_paths` in the
    /// settings database.
    FolderPicked(Option<PathBuf>),
    /// The pointer entered or left a folder chip; carries the chip index.
    ChipHovered(usize, bool),
    /// A folder chip was pressed; opens the folder in the OS file explorer.
    ChipPressed(PathBuf),
    /// The logo was pressed; opens the About dialog.
    LogoPressed,
    /// Closes the About dialog.
    CloseAboutDialog,
    /// The CONFIG. SILO button was pressed; opens the Config dialog.
    OpenConfigSiloDialog,
    /// Closes the Config Silo dialog.
    CloseConfigSiloDialog,
    /// The SYNC SILO button was pressed; opens the Sync dialog.
    OpenSyncSiloDialog,
    /// Closes the Sync Silo dialog.
    CloseSyncSiloDialog,
    /// The GitHub logo was pressed; opens the project page.
    OpenGithub,
    /// A no-op message used to absorb clicks.
    NoOp,
}

/// Boots the Silo application.
///
/// Returns the initial state and a no-op [`Task`].
fn new() -> (SiloApp, Task<Message>) {
    (SiloApp::default(), Task::none())
}

/// Handles application messages.
///
/// A window resize updates the live scale factor so `sp` values follow the
/// current client area. The scaling module ignores no-op changes. A logo hover
/// toggles which logo variant is shown.
fn update(state: &mut SiloApp, message: Message) -> Task<Message> {
    match message {
        Message::WindowResized(size) => {
            Scaling::global().set_window_size(size.width, size.height);
            Task::none()
        }
        Message::LogoHovered(hovered) => {
            state.logo_hovered = hovered;
            Task::none()
        }
        Message::ConfigHovered(hovered) => {
            state.config_hovered = hovered;
            Task::none()
        }
        Message::SyncHovered(hovered) => {
            state.sync_hovered = hovered;
            Task::none()
        }
        Message::PlusHovered(hovered) => {
            state.plus_hovered = hovered;
            Task::none()
        }
        Message::PlusPressed => {
            // Open the OS native folder picker. The picked folder arrives as
            // `FolderPicked`; the folder list is added in a later step.
            Task::perform(
                rfd::AsyncFileDialog::new()
                    .set_title("Select a source folder")
                    .pick_folder(),
                |file| Message::FolderPicked(file.map(|handle| handle.path().to_path_buf())),
            )
        }
        Message::FolderPicked(selection) => {
            if let Some(path) = selection {
                // Append the picked folder to the settings database: one row
                // per folder, never replacing existing rows. Duplicate paths
                // are ignored by the unique index on the `path` column.
                match config::add_data_path(&path) {
                    Ok(()) => {
                        // Keep the in-memory list in sync with the database
                        // while the dialog stays open. Duplicates are skipped,
                        // mirroring the database rule.
                        if !state.folder_paths.contains(&path) {
                            state.folder_paths.push(path);
                        }
                    }
                    Err(err) => {
                        eprintln!("silo: could not save the selected folder {path:?}: {err}");
                    }
                }
            }
            Task::none()
        }
        Message::ChipHovered(index, hovered) => {
            if hovered {
                state.hovered_chip = Some(index);
            } else if state.hovered_chip == Some(index) {
                state.hovered_chip = None;
            }
            Task::none()
        }
        Message::ChipPressed(path) => {
            // Open the folder in the native OS file explorer.
            if let Err(err) = std::process::Command::new("xdg-open").arg(&path).spawn() {
                eprintln!("silo: could not open the folder {path:?}: {err}");
            }
            Task::none()
        }
        Message::LogoPressed => {
            state.about_open = true;
            Task::none()
        }
        Message::CloseAboutDialog => {
            state.about_open = false;
            Task::none()
        }
        Message::OpenConfigSiloDialog => {
            state.config_dialog_open = true;
            // Load the saved source folders once at open. Reloading on every
            // redraw would read the database on each frame.
            state.folder_paths = match config::load() {
                Ok(settings) => settings.silo_data_paths,
                Err(err) => {
                    eprintln!("silo: could not load the saved folders: {err}");
                    Vec::new()
                }
            };
            Task::none()
        }
        Message::CloseConfigSiloDialog => {
            state.config_dialog_open = false;
            state.plus_hovered = false;
            state.hovered_chip = None;
            Task::none()
        }
        Message::OpenSyncSiloDialog => {
            state.sync_dialog_open = true;
            Task::none()
        }
        Message::CloseSyncSiloDialog => {
            state.sync_dialog_open = false;
            Task::none()
        }
        Message::OpenGithub => {
            // Open the project page in the default browser.
            let _ = std::process::Command::new("xdg-open")
                .arg(about_dialog::GITHUB_URL)
                .spawn();
            Task::none()
        }
        Message::NoOp => Task::none(),
    }
}

/// Builds the application view.
///
/// The base is a full-window [`Container`] with no visible content. The theme's
/// `background` palette color (`#161616`) paints the whole surface. The
/// [`action_area`] overlay is stacked above it, and the [`scanlines`] overlay
/// sits on top of everything for the retro CRT screen look.
fn view(state: &SiloApp) -> iced::Element<'_, Message> {
    let base = container(text("")).width(Length::Fill).height(Length::Fill);

    let mut stack = Stack::new().push(base).push(action_area::view(
        state.logo_hovered,
        state.config_hovered,
        state.sync_hovered,
    ));

    if state.about_open {
        stack = stack.push(about_dialog::view());
    }

    if state.config_dialog_open {
        stack = stack.push(config_silo_dialog::view(
            state.plus_hovered,
            &state.folder_paths,
            state.hovered_chip,
        ));
    }

    if state.sync_dialog_open {
        stack = stack.push(sync_silo_dialog::view());
    }

    // The scanlines stay on top of everything, including the dialog, for the
    // retro CRT screen look.
    stack.push(scanlines::overlay()).into()
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
