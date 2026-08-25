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
mod silo_allocation_chart;
mod silo_analysis_layout;
mod silo_stats_table;
mod status_format;
mod sync_progress;
mod sync_progress_bar;
mod sync_silo_actions;
mod sync_silo_dialog;
mod sync_silo_dialog_elements;

use std::collections::BTreeMap;

use iced::widget::{Stack, container, text};
use iced::{Length, Size, Task};

use crate::modules::silo_analysis::{Allocation, AllocationFile};

use super::scaling::Scaling;
use super::scanlines;

use app::{prepare_breakdown_task, set_hovered, silo_allocation_task, silo_size_task};
use config_silo_actions::{ConfigMsg, ConfigState};
use silo_allocation_chart::PreparedBreakdown;
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
    /// The file-type allocation of the silo: the per-extension summary and
    /// the files behind it. Empty while the background computation runs or
    /// when the silo is empty.
    allocation: Allocation,
    /// The extension whose file breakdown is currently expanded in the
    /// ALLOCATION chart, or `None`.
    expanded_extension: Option<String>,
    /// The extension whose breakdown is being prepared in the background, or
    /// `None`.
    pending_extension: Option<String>,
    /// The prepared breakdowns, cached in memory so re-expanding an extension
    /// is instant.
    prepared: BTreeMap<String, PreparedBreakdown>,
    /// Bumps whenever the chart data changes, so the lazy ALLOCATION chart
    /// knows when to rebuild its cached subtree.
    allocation_generation: u64,
    /// The current vertical scroll offset of the ALLOCATION chart, in pixels.
    breakdown_scroll_offset: f32,
    /// The current viewport height of the ALLOCATION chart, in pixels.
    breakdown_viewport_height: f32,
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
    /// A background file-type allocation computation finished; carries the
    /// per-extension statistics and the files behind them.
    AllocationComputed(Allocation),
    /// An extension row in the ALLOCATION chart was pressed; toggles its file
    /// breakdown.
    AllocationRowPressed(String),
    /// A file breakdown finished preparing in the background; carries the
    /// prepared data.
    BreakdownPrepared(PreparedBreakdown),
    /// The ALLOCATION chart scrolled; carries the new absolute offset and
    /// viewport height so the virtualized list can pick its visible window.
    BreakdownScrolled {
        /// The vertical scroll offset, in pixels.
        offset: f32,
        /// The visible viewport height, in pixels.
        viewport_height: f32,
    },
    /// The window regained focus; refresh the total silo size in the
    /// background so the label reflects files changed on disk.
    RefreshSiloSize,
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
            // dialog was open, so recompute the total size and the file-type
            // allocation in the background.
            Task::batch([silo_size_task(), silo_allocation_task()])
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
        Message::AllocationComputed(allocation) => {
            state.allocation = allocation;
            state.allocation_generation += 1;
            // The new data invalidates the prepared breakdowns. The chart
            // keeps its scroll position and clamps to the new content.
            state.prepared.clear();
            state.expanded_extension = None;
            state.pending_extension = None;
            Task::none()
        }
        Message::AllocationRowPressed(extension) => {
            // Pressing the open row collapses it. An already-prepared row
            // expands instantly from the cache; any other row is prepared in
            // the background and expands when the data is ready. The chart
            // keeps its scroll position, so the clicked row stays on screen.
            let is_open = state.expanded_extension.as_deref() == Some(extension.as_str())
                || state.pending_extension.as_deref() == Some(extension.as_str());
            if is_open {
                state.expanded_extension = None;
                state.pending_extension = None;
                Task::none()
            } else if state.prepared.contains_key(&extension) {
                state.expanded_extension = Some(extension);
                state.pending_extension = None;
                Task::none()
            } else {
                state.pending_extension = Some(extension.clone());
                let files: Vec<AllocationFile> = state
                    .allocation
                    .files
                    .iter()
                    .filter(|file| file.extension == extension)
                    .cloned()
                    .collect();
                prepare_breakdown_task(files, extension)
            }
        }
        Message::BreakdownPrepared(breakdown) => {
            // Apply only when the user still waits on this extension.
            if state.pending_extension.as_deref() == Some(breakdown.extension.as_str()) {
                let extension = breakdown.extension.clone();
                state.prepared.insert(extension.clone(), breakdown);
                state.expanded_extension = Some(extension);
                state.pending_extension = None;
                state.allocation_generation += 1;
            }
            Task::none()
        }
        Message::BreakdownScrolled {
            offset,
            viewport_height,
        } => {
            state.breakdown_scroll_offset = offset;
            state.breakdown_viewport_height = viewport_height;
            Task::none()
        }
        Message::RefreshSiloSize => silo_size_task(),
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
/// [`action_area`] overlay is stacked above it, the [`silo_analysis_layout`]
/// panel fills the space below the action area, and the [`scanlines`] overlay
/// sits on top of everything for the retro CRT screen look.
fn view(state: &SiloApp) -> iced::Element<'_, Message> {
    let base = container(text("")).width(Length::Fill).height(Length::Fill);

    let mut stack = Stack::new().push(base).push(action_area::view(
        state.logo_hovered,
        state.config_hovered,
        state.sync_hovered,
        &state.silo_size,
    ));

    // The SILO ANALYSIS panel fills the space below the action area.
    let expanded = state
        .expanded_extension
        .as_ref()
        .and_then(|extension| state.prepared.get(extension));

    stack = stack.push(silo_analysis_layout::view(
        &state.silo_size,
        &state.allocation.stats,
        &state.allocation.summary,
        expanded,
        state.pending_extension.as_deref(),
        state.allocation_generation,
        state.breakdown_scroll_offset,
        state.breakdown_viewport_height,
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
