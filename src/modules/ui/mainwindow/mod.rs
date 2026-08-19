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
mod sync_progress_bar;
mod sync_silo_dialog;

use iced::widget::{Stack, container, text};
use iced::window::Position;
use iced::{Length, Size, Subscription, Task, application};

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
            Task::none()
        }
        Message::CloseConfigSiloDialog => {
            state.config_dialog_open = false;
            state.plus_hovered = false;
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
        stack = stack.push(config_silo_dialog::view(state.plus_hovered));
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
