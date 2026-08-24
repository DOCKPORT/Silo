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
mod config_silo_dialog_exclude;
mod config_silo_dialog_folders;
mod sync_progress_bar;
mod sync_silo_dialog;
mod sync_silo_dialog_elements;

use std::path::{Path, PathBuf};

use iced::widget::{Stack, container, text};
use iced::window::Position;
use iced::{Length, Size, Subscription, Task, application};

use crate::modules::{config, silo_size, sync_engine};

use super::font;
use super::scaling::Scaling;
use super::scanlines;
use super::theme::silo_theme;

/// The kind of a status line; the view maps it to a theme color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StatusKind {
    /// Neutral progress information, rendered in grey.
    Info,
    /// A successful outcome, rendered in teal.
    Success,
    /// A failed outcome, rendered in orange.
    Error,
}

/// One line of output in the Sync dialog STATUS box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StatusLine {
    /// How the line is categorized, which drives its color.
    pub(super) kind: StatusKind,
    /// The text of the line.
    pub(super) text: String,
}

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
    /// The rsync destination path, loaded once when the Sync dialog opens.
    rsync_dest_path: Option<PathBuf>,
    /// Whether the pointer is currently over the + button in the destination box.
    dest_plus_hovered: bool,
    /// Whether the pointer is currently over the destination chip.
    dest_chip_hovered: bool,
    /// Whether the destination remove menu is open.
    dest_menu_open: bool,
    /// Whether the pointer is over the open destination remove menu.
    dest_menu_hovered: bool,
    /// Whether the pointer is currently over the DRY-RUN button.
    dry_run_hovered: bool,
    /// Whether the pointer is currently over the SYNC button in the dialog.
    sync_run_hovered: bool,
    /// The lines shown in the Sync dialog STATUS box, newest last.
    sync_status: Vec<StatusLine>,
    /// The selected source folders, loaded once when the Config dialog opens.
    folder_paths: Vec<PathBuf>,
    /// The size label of each folder, parallel to `folder_paths`. Filled in
    /// asynchronously when the Config dialog opens.
    folder_sizes: Vec<String>,
    /// The index of the folder chip the pointer is currently over, if any.
    hovered_chip: Option<usize>,
    /// The index of the folder chip whose remove menu is open, if any.
    chip_menu: Option<usize>,
    /// Whether the pointer is over the open remove menu row.
    menu_hovered: bool,
    /// Whether the pointer is currently over the + button in the exclude box.
    exclude_plus_hovered: bool,
    /// The exclude patterns, one string per pattern chip.
    exclude_patterns: Vec<String>,
    /// The index of the exclude chip whose delete menu is open, if any.
    exclude_menu: Option<usize>,
    /// Whether the pointer is over the open exclude delete menu.
    exclude_menu_hovered: bool,
    /// Whether the silo has at least one source folder in `silo_data_paths`.
    /// Drives the live STATUS label in the action area.
    is_populated: bool,
    /// The human-readable total silo size, for example "5.46 GiB". Holds "--"
    /// while the first computation runs, and "N/A" when a source folder
    /// cannot be read.
    silo_size: String,
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
    /// A folder's size walk finished; carries the folder path and its label.
    FolderSizeComputed(PathBuf, String),
    /// A background total-size computation finished; carries the size label.
    SiloSizeComputed(String),
    /// A folder chip was right-pressed; opens its remove menu.
    ChipMenuRequested(usize),
    /// The remove menu item was pressed; removes the folder at the index.
    RemoveFolder(usize),
    /// Dismisses the open chip remove menu.
    CloseChipMenu,
    /// The pointer entered or left the open remove menu row.
    MenuHovered(bool),
    /// The pointer entered or left the + button in the exclude box.
    ExcludePlusHovered(bool),
    /// The + button in the exclude box was pressed; adds a new pattern chip.
    ExcludePlusPressed,
    /// A pattern chip's text changed; carries the chip index and new value.
    ExcludePatternChanged(usize, String),
    /// An exclude chip was right-pressed; opens its delete menu.
    ExcludeMenuRequested(usize),
    /// The delete menu item was pressed; removes the pattern at the index.
    ExcludePatternRemoved(usize),
    /// Dismisses the open exclude delete menu.
    CloseExcludeMenu,
    /// The pointer entered or left the open exclude delete menu.
    ExcludeMenuHovered(bool),
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
    /// The GitHub logo was pressed; opens the project page.
    OpenGithub,
    /// A no-op message used to absorb clicks.
    NoOp,
}

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
            app.folder_paths = settings.silo_data_paths;
            app.exclude_patterns = settings.excludes;
            app.rsync_dest_path = settings.rsync_dest_path;
            app.is_populated = !app.folder_paths.is_empty();
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
fn silo_size_task() -> Task<Message> {
    Task::perform(
        async { silo_size::silo_size_label() },
        Message::SiloSizeComputed,
    )
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
        Message::LogoHovered(hovered) => set_hovered(&mut state.logo_hovered, hovered),
        Message::ConfigHovered(hovered) => set_hovered(&mut state.config_hovered, hovered),
        Message::SyncHovered(hovered) => set_hovered(&mut state.sync_hovered, hovered),
        Message::PlusHovered(hovered) => set_hovered(&mut state.plus_hovered, hovered),
        Message::PlusPressed => {
            // Open the OS native folder picker. The picked folder arrives as
            // `FolderPicked`; the folder list is added in a later step.
            pick_folder("Select a source folder", Message::FolderPicked)
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
                            state.folder_paths.push(path.clone());
                            state.folder_sizes.push("...".to_string());
                            state.is_populated = true;
                            // Compute the size label for the new folder.
                            return config_silo_dialog_folders::size_tasks(&[path])
                                .pop()
                                .unwrap_or_else(|| Task::none());
                        }
                    }
                    Err(err) => {
                        eprintln!("silo: could not save the selected folder {path:?}: {err}");
                    }
                }
            }
            Task::none()
        }
        Message::FolderSizeComputed(path, size) => {
            // Look up the folder by path: the list may have changed while the
            // walk ran, so an index captured at task creation could point to
            // the wrong chip.
            if let Some(index) = state.folder_paths.iter().position(|p| p == &path) {
                if let Some(slot) = state.folder_sizes.get_mut(index) {
                    *slot = size;
                }
            }
            Task::none()
        }
        Message::SiloSizeComputed(label) => {
            state.silo_size = label;
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
            open_in_file_explorer(&path);
            state.chip_menu = None;
            state.menu_hovered = false;
            Task::none()
        }
        Message::ChipMenuRequested(index) => {
            // Right-pressing the same chip again collapses the menu.
            state.chip_menu = if state.chip_menu == Some(index) {
                None
            } else {
                Some(index)
            };
            state.menu_hovered = false;
            Task::none()
        }
        Message::RemoveFolder(index) => {
            if let Some(path) = state.folder_paths.get(index) {
                match config::remove_data_path(path) {
                    Ok(()) => {
                        state.folder_paths.remove(index);
                        state.folder_sizes.remove(index);
                        state.is_populated = !state.folder_paths.is_empty();
                    }
                    Err(err) => {
                        eprintln!("silo: could not remove the folder {path:?}: {err}");
                    }
                }
            }
            state.chip_menu = None;
            state.hovered_chip = None;
            state.menu_hovered = false;
            Task::none()
        }
        Message::CloseChipMenu => {
            state.chip_menu = None;
            state.menu_hovered = false;
            Task::none()
        }
        Message::MenuHovered(hovered) => set_hovered(&mut state.menu_hovered, hovered),
        Message::ExcludePlusHovered(hovered) => {
            set_hovered(&mut state.exclude_plus_hovered, hovered)
        }
        Message::ExcludePlusPressed => {
            // Add a new empty pattern chip and persist the list.
            state.exclude_patterns.push(String::new());
            save_excludes(state);
            Task::none()
        }
        Message::ExcludePatternChanged(index, value) => {
            if let Some(slot) = state.exclude_patterns.get_mut(index) {
                *slot = value;
            }
            save_excludes(state);
            Task::none()
        }
        Message::ExcludeMenuRequested(index) => {
            // Right-pressing the same chip again collapses the menu.
            state.exclude_menu = if state.exclude_menu == Some(index) {
                None
            } else {
                Some(index)
            };
            state.exclude_menu_hovered = false;
            Task::none()
        }
        Message::ExcludePatternRemoved(index) => {
            if index < state.exclude_patterns.len() {
                state.exclude_patterns.remove(index);
            }
            state.exclude_menu = None;
            state.exclude_menu_hovered = false;
            save_excludes(state);
            Task::none()
        }
        Message::CloseExcludeMenu => {
            state.exclude_menu = None;
            state.exclude_menu_hovered = false;
            Task::none()
        }
        Message::ExcludeMenuHovered(hovered) => {
            set_hovered(&mut state.exclude_menu_hovered, hovered)
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
            reset_config_dialog(state);
            // Load the saved settings once at open. Reloading on every
            // redraw would read the database on each frame.
            match config::load() {
                Ok(settings) => {
                    state.folder_paths = settings.silo_data_paths;
                    state.exclude_patterns = settings.excludes;
                }
                Err(err) => {
                    eprintln!("silo: could not load the saved settings: {err}");
                    state.folder_paths = Vec::new();
                    state.exclude_patterns = Vec::new();
                }
            };
            // Reflect the loaded rows in the live STATUS label.
            state.is_populated = !state.folder_paths.is_empty();
            // Show a placeholder size label per folder and compute the real
            // sizes asynchronously, so large folders do not freeze the UI.
            state.folder_sizes = vec!["...".to_string(); state.folder_paths.len()];
            Task::batch(config_silo_dialog_folders::size_tasks(&state.folder_paths))
        }
        Message::CloseConfigSiloDialog => {
            state.config_dialog_open = false;
            reset_config_dialog(state);
            // The folders or exclude patterns may have changed while the
            // dialog was open, so recompute the total size in the background.
            silo_size_task()
        }
        Message::OpenSyncSiloDialog => {
            state.sync_dialog_open = true;
            reset_sync_dialog(state);
            // Load the saved destination once at open. Reloading on every
            // redraw would read the database on each frame.
            match config::load() {
                Ok(settings) => state.rsync_dest_path = settings.rsync_dest_path,
                Err(err) => {
                    eprintln!("silo: could not load the saved settings: {err}");
                    state.rsync_dest_path = None;
                }
            };
            // Start with a fresh STATUS box on every open.
            state.sync_status.clear();
            Task::none()
        }
        Message::CloseSyncSiloDialog => {
            state.sync_dialog_open = false;
            reset_sync_dialog(state);
            Task::none()
        }
        Message::DestPlusHovered(hovered) => set_hovered(&mut state.dest_plus_hovered, hovered),
        Message::DestPlusPressed => {
            // Open the OS native folder picker. The picked folder arrives as
            // `DestFolderPicked`.
            pick_folder("Select a sync destination", Message::DestFolderPicked)
        }
        Message::DestFolderPicked(selection) => {
            if let Some(path) = selection {
                // Replace the destination row, keeping exactly one path in
                // the table, and mirror the new path in the dialog state.
                match config::set_rsync_dest_path(Some(&path)) {
                    Ok(()) => state.rsync_dest_path = Some(path),
                    Err(err) => {
                        eprintln!("silo: could not save the destination folder {path:?}: {err}");
                    }
                }
            }
            Task::none()
        }
        Message::DestChipHovered(hovered) => set_hovered(&mut state.dest_chip_hovered, hovered),
        Message::DestChipPressed => {
            // Open the destination folder in the native OS file explorer.
            if let Some(path) = &state.rsync_dest_path {
                open_in_file_explorer(path);
            }
            state.dest_menu_open = false;
            state.dest_menu_hovered = false;
            Task::none()
        }
        Message::DestChipMenuRequested => {
            // Right-pressing the chip again collapses the menu.
            state.dest_menu_open = !state.dest_menu_open;
            state.dest_menu_hovered = false;
            Task::none()
        }
        Message::DestMenuHovered(hovered) => set_hovered(&mut state.dest_menu_hovered, hovered),
        Message::RemoveDestPath => {
            // Clear the destination row, keeping exactly one row in the
            // table, and mirror the change in the dialog state.
            match config::set_rsync_dest_path(None) {
                Ok(()) => state.rsync_dest_path = None,
                Err(err) => {
                    eprintln!("silo: could not remove the destination folder: {err}");
                }
            }
            state.dest_chip_hovered = false;
            state.dest_menu_open = false;
            state.dest_menu_hovered = false;
            Task::none()
        }
        Message::CloseDestMenu => {
            state.dest_menu_open = false;
            state.dest_menu_hovered = false;
            Task::none()
        }
        Message::DryRunHovered(hovered) => set_hovered(&mut state.dry_run_hovered, hovered),
        Message::DryRunPressed => {
            state.sync_status.push(StatusLine {
                kind: StatusKind::Info,
                text: "Dry run in progress...".to_string(),
            });

            // Build the plan from the current settings. The engine performs
            // the pre-flight checks (rsync present, sources exist) in the
            // background task.
            let Some(plan) = build_sync_plan(state) else {
                state.sync_status.push(StatusLine {
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
                Message::DryRunFinished(lines)
            })
        }
        Message::DryRunFinished(lines) => {
            state.sync_status.extend(lines);
            Task::none()
        }
        Message::SyncRunHovered(hovered) => set_hovered(&mut state.sync_run_hovered, hovered),
        Message::SyncRunPressed => {
            state.sync_status.push(StatusLine {
                kind: StatusKind::Info,
                text: "Preparing sync...".to_string(),
            });

            // Build the plan from the same settings as the dry run. The
            // engine performs the pre-flight checks in the background task.
            let Some(plan) = build_sync_plan(state) else {
                state.sync_status.push(StatusLine {
                    kind: StatusKind::Error,
                    text: "Sync failed: no sync destination selected".to_string(),
                });
                return Task::none();
            };

            // Stage two: mark the run as in progress, then spawn the engine
            // task carrying the plan.
            Task::perform(async move { plan }, Message::SyncStarted)
        }
        Message::SyncStarted(plan) => {
            state.sync_status.push(StatusLine {
                kind: StatusKind::Info,
                text: "Sync in progress...".to_string(),
            });
            Task::perform(async move { sync_engine::sync(&plan) }, |result| {
                Message::SyncFinished(sync_result_lines(result))
            })
        }
        Message::SyncFinished(lines) => {
            state.sync_status.extend(lines);
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

/// Sets a boolean hover flag and returns a no-op task.
fn set_hovered(slot: &mut bool, hovered: bool) -> Task<Message> {
    *slot = hovered;
    Task::none()
}

/// Opens the OS native folder picker with the given title.
///
/// The picked folder maps through `map`, which the caller uses to produce
/// `Message::FolderPicked` or `Message::DestFolderPicked`.
fn pick_folder(
    title: &'static str,
    map: impl Fn(Option<PathBuf>) -> Message + Send + 'static,
) -> Task<Message> {
    Task::perform(
        rfd::AsyncFileDialog::new().set_title(title).pick_folder(),
        move |file| map(file.map(|handle| handle.path().to_path_buf())),
    )
}

/// Opens a folder in the native OS file explorer.
fn open_in_file_explorer(path: &Path) {
    if let Err(err) = std::process::Command::new("xdg-open").arg(path).spawn() {
        eprintln!("silo: could not open the folder {path:?}: {err}");
    }
}

/// Resets the Config dialog interaction flags.
fn reset_config_dialog(state: &mut SiloApp) {
    state.plus_hovered = false;
    state.hovered_chip = None;
    state.chip_menu = None;
    state.menu_hovered = false;
    state.exclude_plus_hovered = false;
    state.exclude_menu = None;
    state.exclude_menu_hovered = false;
}

/// Resets the Sync dialog interaction flags.
fn reset_sync_dialog(state: &mut SiloApp) {
    state.dest_plus_hovered = false;
    state.dest_chip_hovered = false;
    state.dest_menu_open = false;
    state.dest_menu_hovered = false;
    state.dry_run_hovered = false;
    state.sync_run_hovered = false;
}

/// Persist the current exclude patterns to the database.
fn save_excludes(state: &SiloApp) {
    if let Err(err) = config::replace_excludes(&state.exclude_patterns) {
        eprintln!("silo: could not save the exclude patterns: {err}");
    }
}

/// Builds the sync plan from the current settings.
///
/// The source folders, exclude patterns, and destination come from the
/// in-memory state, which mirrors the settings database tables. Returns `None`
/// when no sync destination is selected.
fn build_sync_plan(state: &SiloApp) -> Option<sync_engine::SyncPlan> {
    let destination = state.rsync_dest_path.clone()?;
    Some(sync_engine::SyncPlan::new(
        state.folder_paths.clone(),
        state.exclude_patterns.clone(),
        destination,
    ))
}

/// Turns a finished dry run into the lines shown in the STATUS box.
///
/// A teal "Dry run complete" header, the rsync stats summary in grey, and any
/// rsync warnings in orange. Trailing blank space is trimmed from the output.
/// The stats byte counts are re-formatted to IEC units, so they read in GiB
/// like every other size label in the UI.
fn dry_run_result_lines(outcome: sync_engine::DryRunOutcome) -> Vec<StatusLine> {
    let mut lines = vec![StatusLine {
        kind: StatusKind::Success,
        text: "Dry run complete".to_string(),
    }];

    let summary = reformat_stats_summary(&dry_run_summary(&outcome.stdout));
    if !summary.is_empty() {
        lines.push(StatusLine {
            kind: StatusKind::Info,
            text: summary,
        });
    }

    let stderr = outcome.stderr.trim();
    if !stderr.is_empty() {
        lines.push(StatusLine {
            kind: StatusKind::Error,
            text: stderr.to_string(),
        });
    }

    lines
}

/// Extracts the stats summary from a dry run's stdout.
///
/// The full output lists every file change; only the trailing stats block is
/// wanted for the STATUS box. The block starts at the `Number of files:` line
/// and runs to the end. Returns the whole trimmed output when the block is
/// missing.
fn dry_run_summary(stdout: &str) -> String {
    match stdout.find("Number of files:") {
        Some(index) => stdout[index..].trim().to_string(),
        None => stdout.trim().to_string(),
    }
}

/// Re-formats the byte counts in a dry-run stats summary to IEC units.
///
/// rsync prints raw byte numbers in its stats block. Every value that is a
/// byte count becomes a [`silo_size::human_size`] label, so the summary reads
/// in GiB like the rest of the UI. Lines without a byte count pass through
/// unchanged.
fn reformat_stats_summary(summary: &str) -> String {
    summary
        .lines()
        .map(reformat_stats_line)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Re-formats one stats line's byte counts to IEC units.
///
/// Handles the known byte-count shapes in the rsync stats block: `label: N
/// bytes`, `Total bytes sent/received: N`, the trailing `sent N bytes
/// received M bytes ...` line, and `total size is N speedup is ...`. Any
/// other line is returned unchanged. rsync prints thousands separators by
/// default, so the byte counts are normalized before parsing.
fn reformat_stats_line(line: &str) -> String {
    // "label: N bytes", for example "Total file size: 7,423,077,535 bytes".
    if let Some((label, raw)) = line.rsplit_once(": ") {
        if let Some(rest) = raw.strip_suffix(" bytes") {
            if let Some(bytes) = parse_bytes(rest) {
                return format!("{label}: {}", silo_size::human_size(bytes));
            }
        }
        // "Total bytes sent: 517,176" and "Total bytes received: 2,754".
        if label.starts_with("Total bytes") {
            if let Some(bytes) = parse_bytes(raw) {
                return format!("{label}: {}", silo_size::human_size(bytes));
            }
        }
    }

    // "sent 517,176 bytes  received 2,754 bytes  1,039,860.00 bytes/sec".
    if let Some(rest) = line.strip_prefix("sent ") {
        let tokens: Vec<&str> = rest.split_whitespace().collect();
        if tokens.len() >= 5
            && tokens[1] == "bytes"
            && tokens[2] == "received"
            && tokens[4] == "bytes"
        {
            if let (Some(sent), Some(received)) = (parse_bytes(tokens[0]), parse_bytes(tokens[3])) {
                return format!(
                    "sent {}  received {}  {}",
                    silo_size::human_size(sent),
                    silo_size::human_size(received),
                    tokens[5..].join(" ")
                );
            }
        }
    }

    // "total size is 7,423,077,535  speedup is 14,277.07 (DRY RUN)".
    if let Some(rest) = line.strip_prefix("total size is ") {
        if let Some((value, tail)) = rest.split_once(char::is_whitespace) {
            if let Some(bytes) = parse_bytes(value) {
                return format!(
                    "total size is {} {}",
                    silo_size::human_size(bytes),
                    tail.trim_start()
                );
            }
        }
    }

    line.to_string()
}

/// Parses a byte count that may include thousands separators.
///
/// rsync prints numbers such as `7,423,077,535` by default; the separators
/// are removed so the value parses as a plain integer.
fn parse_bytes(raw: &str) -> Option<u64> {
    raw.replace(',', "").parse().ok()
}

/// Turns a finished sync into the lines shown in the STATUS box.
///
/// Success shows a teal completion message; rsync and engine failures show
/// an orange reason. Any rsync stderr output is appended in orange, trimmed.
fn sync_result_lines(
    result: Result<sync_engine::SyncOutcome, sync_engine::SyncError>,
) -> Vec<StatusLine> {
    match result {
        Ok(sync_engine::SyncOutcome::Success { stderr, .. }) => {
            let mut lines = vec![StatusLine {
                kind: StatusKind::Success,
                text: "Sync complete. You can close this dialog; the application stays open."
                    .to_string(),
            }];
            append_sync_stderr(&mut lines, &stderr);
            lines
        }
        Ok(sync_engine::SyncOutcome::Failure {
            exit_code, stderr, ..
        }) => {
            let reason = match exit_code {
                Some(code) => format!("Sync failed: rsync exited with code {code}"),
                None => "Sync failed: rsync did not exit cleanly".to_string(),
            };
            let mut lines = vec![StatusLine {
                kind: StatusKind::Error,
                text: reason,
            }];
            append_sync_stderr(&mut lines, &stderr);
            lines
        }
        Err(err) => vec![StatusLine {
            kind: StatusKind::Error,
            text: format!("Sync failed: {err}"),
        }],
    }
}

/// Appends rsync's standard error to the status lines, if there is any.
///
/// rsync writes warnings and error details to stderr. They are shown in
/// orange so they stand out from the progress and completion lines.
fn append_sync_stderr(lines: &mut Vec<StatusLine>, stderr: &str) {
    let stderr = stderr.trim();
    if !stderr.is_empty() {
        lines.push(StatusLine {
            kind: StatusKind::Error,
            text: stderr.to_string(),
        });
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
        state.is_populated,
        &state.silo_size,
    ));

    if state.about_open {
        stack = stack.push(about_dialog::view());
    }

    if state.config_dialog_open {
        stack = stack.push(config_silo_dialog::view(
            state.plus_hovered,
            &state.folder_paths,
            &state.folder_sizes,
            state.hovered_chip,
            state.chip_menu,
            state.menu_hovered,
            state.exclude_plus_hovered,
            &state.exclude_patterns,
            state.exclude_menu,
            state.exclude_menu_hovered,
        ));
    }

    if state.sync_dialog_open {
        stack = stack.push(sync_silo_dialog::view(
            state.rsync_dest_path.as_deref(),
            state.dest_plus_hovered,
            state.dest_chip_hovered,
            state.dest_menu_open,
            state.dest_menu_hovered,
            state.dry_run_hovered,
            state.sync_run_hovered,
            &state.sync_status,
        ));
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
