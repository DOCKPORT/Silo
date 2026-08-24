//! SyncSiloActions: the SYNC SILO dialog logic.
//!
//! Owns the dialog's state and messages: the destination picker and chip, the
//! dry-run flow, and the sync run flow. The dialog box drawing lives in
//! [`super::sync_silo_dialog`].

use std::path::PathBuf;

use iced::Task;

use crate::modules::{config, sync_engine};

use super::app::{open_in_file_explorer, pick_folder, set_hovered};
use super::status_format::{StatusKind, StatusLine, dry_run_result_lines, sync_result_lines};
use super::{Message, SiloApp};

/// The Sync Silo dialog state.
///
/// The destination row mirrors the settings database while the dialog stays
/// open; every change is persisted immediately.
#[derive(Debug, Default)]
pub(super) struct SyncState {
    /// The rsync destination path, loaded once when the Sync dialog opens.
    pub(super) rsync_dest_path: Option<PathBuf>,
    /// Whether the pointer is currently over the + button in the destination
    /// box.
    pub(super) dest_plus_hovered: bool,
    /// Whether the pointer is currently over the destination chip.
    pub(super) dest_chip_hovered: bool,
    /// Whether the destination remove menu is open.
    pub(super) dest_menu_open: bool,
    /// Whether the pointer is over the open destination remove menu.
    pub(super) dest_menu_hovered: bool,
    /// Whether the pointer is currently over the DRY-RUN button.
    pub(super) dry_run_hovered: bool,
    /// Whether the pointer is currently over the SYNC button in the dialog.
    pub(super) sync_run_hovered: bool,
    /// The lines shown in the Sync dialog STATUS box, newest last.
    pub(super) sync_status: Vec<StatusLine>,
}

impl SyncState {
    /// Resets the dialog interaction flags.
    pub(super) fn reset(&mut self) {
        self.dest_plus_hovered = false;
        self.dest_chip_hovered = false;
        self.dest_menu_open = false;
        self.dest_menu_hovered = false;
        self.dry_run_hovered = false;
        self.sync_run_hovered = false;
    }
}

/// Messages that drive the Sync Silo dialog.
#[derive(Debug, Clone)]
pub(super) enum SyncMsg {
    /// The pointer entered or left the + button in the destination box.
    DestPlusHovered(bool),
    /// The + button in the destination box was pressed; opens the OS native
    /// folder picker.
    DestPlusPressed,
    /// The folder picker returned a chosen destination, or `None` if the user
    /// cancelled. A chosen folder replaces the row in `rsync_dest_path`.
    DestFolderPicked(Option<PathBuf>),
    /// The pointer entered or left the destination chip.
    DestChipHovered(bool),
    /// The destination chip was pressed; opens the folder in the OS file
    /// explorer.
    DestChipPressed,
    /// The destination chip was right-pressed; opens its remove menu.
    DestChipMenuRequested,
    /// The remove menu item was pressed; removes the destination.
    RemoveDestPath,
    /// Dismisses the open destination remove menu.
    CloseDestMenu,
    /// The pointer entered or left the open destination remove menu.
    DestMenuHovered(bool),
    /// The pointer entered or left the DRY-RUN button.
    DryRunHovered(bool),
    /// The DRY-RUN button was pressed; starts the dry-run status flow.
    DryRunPressed,
    /// The dry run finished; carries the result lines for the STATUS box.
    DryRunFinished(Vec<StatusLine>),
    /// The pointer entered or left the SYNC button in the dialog.
    SyncRunHovered(bool),
    /// The SYNC button was pressed; starts the sync status flow.
    SyncRunPressed,
    /// The sync is ready to run; carries the plan the engine task will use.
    SyncStarted(sync_engine::SyncPlan),
    /// The sync finished; carries the outcome lines for the STATUS box.
    SyncFinished(Vec<StatusLine>),
}

/// Opens the dialog: resets the interaction flags and loads the saved
/// destination.
pub(super) fn open(state: &mut SiloApp) -> Task<Message> {
    state.sync.reset();
    // Load the saved destination once at open. Reloading on every redraw
    // would read the database on each frame.
    match config::load() {
        Ok(settings) => state.sync.rsync_dest_path = settings.rsync_dest_path,
        Err(err) => {
            eprintln!("silo: could not load the saved settings: {err}");
            state.sync.rsync_dest_path = None;
        }
    };
    // Start with a fresh STATUS box on every open.
    state.sync.sync_status.clear();
    Task::none()
}

/// Handles a Sync Silo dialog message.
pub(super) fn update(state: &mut SiloApp, message: SyncMsg) -> Task<Message> {
    match message {
        SyncMsg::DestPlusHovered(hovered) => {
            set_hovered(&mut state.sync.dest_plus_hovered, hovered)
        }
        SyncMsg::DestPlusPressed => {
            // Open the OS native folder picker. The picked folder arrives as
            // `SyncMsg::DestFolderPicked`.
            pick_folder("Select a sync destination", |selection| {
                Message::Sync(SyncMsg::DestFolderPicked(selection))
            })
        }
        SyncMsg::DestFolderPicked(selection) => {
            if let Some(path) = selection {
                // Replace the destination row, keeping exactly one path in the
                // table, and mirror the new path in the dialog state.
                match config::set_rsync_dest_path(Some(&path)) {
                    Ok(()) => state.sync.rsync_dest_path = Some(path),
                    Err(err) => {
                        eprintln!("silo: could not save the destination folder {path:?}: {err}");
                    }
                }
            }
            Task::none()
        }
        SyncMsg::DestChipHovered(hovered) => {
            set_hovered(&mut state.sync.dest_chip_hovered, hovered)
        }
        SyncMsg::DestChipPressed => {
            // Open the destination folder in the native OS file explorer.
            if let Some(path) = &state.sync.rsync_dest_path {
                open_in_file_explorer(path);
            }
            state.sync.dest_menu_open = false;
            state.sync.dest_menu_hovered = false;
            Task::none()
        }
        SyncMsg::DestChipMenuRequested => {
            // Right-pressing the chip again collapses the menu.
            state.sync.dest_menu_open = !state.sync.dest_menu_open;
            state.sync.dest_menu_hovered = false;
            Task::none()
        }
        SyncMsg::DestMenuHovered(hovered) => {
            set_hovered(&mut state.sync.dest_menu_hovered, hovered)
        }
        SyncMsg::RemoveDestPath => {
            // Clear the destination row, keeping exactly one row in the table,
            // and mirror the change in the dialog state.
            match config::set_rsync_dest_path(None) {
                Ok(()) => state.sync.rsync_dest_path = None,
                Err(err) => {
                    eprintln!("silo: could not remove the destination folder: {err}");
                }
            }
            state.sync.dest_chip_hovered = false;
            state.sync.dest_menu_open = false;
            state.sync.dest_menu_hovered = false;
            Task::none()
        }
        SyncMsg::CloseDestMenu => {
            state.sync.dest_menu_open = false;
            state.sync.dest_menu_hovered = false;
            Task::none()
        }
        SyncMsg::DryRunHovered(hovered) => set_hovered(&mut state.sync.dry_run_hovered, hovered),
        SyncMsg::DryRunPressed => {
            state.sync.sync_status.push(StatusLine {
                kind: StatusKind::Info,
                text: "Dry run in progress...".to_string(),
            });

            // Build the plan from the current settings. The engine performs
            // the pre-flight checks (rsync present, sources exist) in the
            // background task.
            let Some(plan) = build_sync_plan(state) else {
                state.sync.sync_status.push(StatusLine {
                    kind: StatusKind::Error,
                    text: "Dry run failed: no sync destination selected".to_string(),
                });
                return Task::none();
            };

            Task::perform(async move { sync_engine::dry_run(&plan) }, |result| {
                let lines = match result {
                    Ok(outcome) => dry_run_result_lines(outcome),
                    Err(err) => vec![StatusLine {
                        kind: StatusKind::Error,
                        text: format!("Dry run failed: {err}"),
                    }],
                };
                Message::Sync(SyncMsg::DryRunFinished(lines))
            })
        }
        SyncMsg::DryRunFinished(lines) => {
            state.sync.sync_status.extend(lines);
            Task::none()
        }
        SyncMsg::SyncRunHovered(hovered) => set_hovered(&mut state.sync.sync_run_hovered, hovered),
        SyncMsg::SyncRunPressed => {
            state.sync.sync_status.push(StatusLine {
                kind: StatusKind::Info,
                text: "Preparing sync...".to_string(),
            });

            // Build the plan from the same settings as the dry run. The
            // engine performs the pre-flight checks in the background task.
            let Some(plan) = build_sync_plan(state) else {
                state.sync.sync_status.push(StatusLine {
                    kind: StatusKind::Error,
                    text: "Sync failed: no sync destination selected".to_string(),
                });
                return Task::none();
            };

            // Stage two: mark the run as in progress, then spawn the engine
            // task carrying the plan.
            Task::perform(async move { plan }, |plan| {
                Message::Sync(SyncMsg::SyncStarted(plan))
            })
        }
        SyncMsg::SyncStarted(plan) => {
            state.sync.sync_status.push(StatusLine {
                kind: StatusKind::Info,
                text: "Sync in progress...".to_string(),
            });
            Task::perform(async move { sync_engine::sync(&plan) }, |result| {
                Message::Sync(SyncMsg::SyncFinished(sync_result_lines(result)))
            })
        }
        SyncMsg::SyncFinished(lines) => {
            state.sync.sync_status.extend(lines);
            Task::none()
        }
    }
}

/// Builds the sync plan from the current settings.
///
/// The source folders, exclude patterns, and destination come from the
/// in-memory state, which mirrors the settings database tables. Returns `None`
/// when no sync destination is selected.
fn build_sync_plan(state: &SiloApp) -> Option<sync_engine::SyncPlan> {
    let destination = state.sync.rsync_dest_path.clone()?;
    Some(sync_engine::SyncPlan::new(
        state.config.folder_paths.clone(),
        state.config.exclude_patterns.clone(),
        destination,
    ))
}
