//! MainWindow: the Iced main window for Silo.
//!
//! A blank window whose whole surface is painted with the Silo theme
//! background `#161616`. The window shows no content yet; the view
//! returns a full-window [`Container`] that is ready to receive children
//! as the UI grows.

mod about_dialog;
mod action_area;
mod app;
mod app_icon;
mod config_silo_actions;
mod config_silo_dialog;
mod config_silo_dialog_elements;
mod config_silo_dialog_exclude;
mod config_silo_dialog_folders;
mod status_format;
mod sync_progress;
mod sync_progress_bar;
mod sync_silo_actions;
mod sync_silo_dialog;
mod sync_silo_dialog_elements;

use iced::widget::{Stack, container, text};
use iced::{Length, Size, Task};

use super::scaling::Scaling;
use super::scanlines;

use app::{set_hovered, silo_size_task};
use config_silo_actions::{ConfigMsg, ConfigState};
use sync_silo_actions::{SyncMsg, SyncState};

/// The Silo application state.
///
/// The application-level fields (hover flags, open dialogs, the total silo
/// size label) live here; each dialog owns its own state group in
/// [`ConfigState`] and [`SyncState`].
#[derive(Debug, Default)]
pub struct SiloApp {
    /// Whether the pointer is currently over the logo.
    logo_hovered: bool,
    /// Whether the pointer is currently over the CONFIG button.
    config_hovered: bool,
    /// Whether the pointer is currently over the SYNC button.
    sync_hovered: bool,
    /// Whether the About dialog is currently open.
    about_open: bool,
    /// Whether the pointer is currently over the About dialog CLOSE button.
    about_close_hovered: bool,
    /// Whether the Config Silo dialog is currently open.
    config_dialog_open: bool,
    /// Whether the Sync Silo dialog is currently open.
    sync_dialog_open: bool,
    /// The human-readable total silo size, for example "5.5 GiB". Holds "--"
    /// while the first computation runs, and "N/A" when a source folder
    /// cannot be read.
    silo_size: String,
    /// The Config Silo dialog state: the folder and exclude rows plus their
    /// interaction flags.
    config: ConfigState,
    /// The Sync Silo dialog state: the destination, the STATUS box, and the
    /// interaction flags.
    sync: SyncState,
}

/// Messages that drive the Silo application.
///
/// The application-level variants are handled here in [`update`]. Each dialog
/// wraps its own message enum so the dialog logic stays in its own module.
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
    /// The logo was pressed; opens the About dialog.
    LogoPressed,
    /// Closes the About dialog.
    CloseAboutDialog,
    /// The pointer entered or left the About dialog CLOSE button.
    AboutCloseHovered(bool),
    /// The CONFIG. SILO button was pressed; opens the Config dialog.
    OpenConfigSiloDialog,
    /// Closes the Config Silo dialog.
    CloseConfigSiloDialog,
    /// The SYNC SILO button was pressed; opens the Sync dialog.
    OpenSyncSiloDialog,
    /// Closes the Sync Silo dialog.
    CloseSyncSiloDialog,
    /// A background total-size computation finished; carries the size label.
    SiloSizeComputed(String),
    /// The GitHub logo was pressed; opens the project page.
    OpenGithub,
    /// A no-op message used to absorb clicks.
    NoOp,
    /// A message for the Config Silo dialog; handled by
    /// [`config_silo_actions`].
    Config(ConfigMsg),
    /// A message for the Sync Silo dialog; handled by [`sync_silo_actions`].
    Sync(SyncMsg),
}

/// Handles application messages.
///
/// A window resize updates the live scale factor so `sp` values follow the
/// current client area. The scaling module ignores no-op changes. A logo hover
/// toggles which logo variant is shown. The dialog messages are handed off to
/// the dialog modules, which own their state groups.
fn update(state: &mut SiloApp, message: Message) -> Task<Message> {
    match message {
        Message::WindowResized(size) => {
            Scaling::global().set_window_size(size.width, size.height);
            Task::none()
        }
        Message::LogoHovered(hovered) => set_hovered(&mut state.logo_hovered, hovered),
        Message::ConfigHovered(hovered) => set_hovered(&mut state.config_hovered, hovered),
        Message::SyncHovered(hovered) => set_hovered(&mut state.sync_hovered, hovered),
        Message::LogoPressed => {
            state.about_open = true;
            Task::none()
        }
        Message::CloseAboutDialog => {
            state.about_open = false;
            Task::none()
        }
        Message::AboutCloseHovered(hovered) => set_hovered(&mut state.about_close_hovered, hovered),
        Message::OpenConfigSiloDialog => {
            state.config_dialog_open = true;
            config_silo_actions::open(state)
        }
        Message::CloseConfigSiloDialog => {
            state.config_dialog_open = false;
            state.config.reset();
            // The folders or exclude patterns may have changed while the
            // dialog was open, so recompute the total size in the background.
            silo_size_task()
        }
        Message::OpenSyncSiloDialog => {
            state.sync_dialog_open = true;
            sync_silo_actions::open(state)
        }
        Message::CloseSyncSiloDialog => {
            state.sync_dialog_open = false;
            state.sync.reset();
            Task::none()
        }
        Message::SiloSizeComputed(label) => {
            state.silo_size = label;
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
        Message::Config(message) => config_silo_actions::update(state, message),
        Message::Sync(message) => sync_silo_actions::update(state, message),
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

    // The live STATUS label derives from the loaded source folders, so it is
    // true exactly when at least one folder exists.
    let is_populated = !state.config.folder_paths.is_empty();

    let mut stack = Stack::new().push(base).push(action_area::view(
        state.logo_hovered,
        state.config_hovered,
        state.sync_hovered,
        is_populated,
        &state.silo_size,
    ));

    if state.about_open {
        stack = stack.push(about_dialog::view(state.about_close_hovered));
    }

    if state.config_dialog_open {
        stack = stack.push(config_silo_dialog::view(&state.config));
    }

    if state.sync_dialog_open {
        stack = stack.push(sync_silo_dialog::view(&state.sync));
    }

    // The scanlines stay on top of everything, including the dialog, for the
    // retro CRT screen look.
    stack.push(scanlines::overlay()).into()
}

/// Runs the Silo main window.
pub use app::run;
