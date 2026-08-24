//! ConfigSiloActions: the CONFIG. SILO dialog logic.
//!
//! Owns the dialog's state and messages: the folder picker, the folder chips,
//! the exclude patterns, and the background size walks. The dialog box drawing
//! lives in [`super::config_silo_dialog`].

use std::path::PathBuf;

use iced::Task;

use crate::modules::config;

use super::app::{open_in_file_explorer, pick_folder, set_hovered};
use super::{Message, SiloApp};

/// The Config Silo dialog state.
///
/// The folder and exclude rows mirror the settings database tables while the
/// dialog stays open; every change is persisted immediately.
#[derive(Debug, Default)]
pub(super) struct ConfigState {
    /// Whether the pointer is currently over the + button in the folder box.
    pub(super) plus_hovered: bool,
    /// The selected source folders, loaded once when the dialog opens.
    pub(super) folder_paths: Vec<PathBuf>,
    /// The size label of each folder, parallel to `folder_paths`. Filled in
    /// asynchronously when the dialog opens.
    pub(super) folder_sizes: Vec<String>,
    /// The index of the folder chip the pointer is currently over, if any.
    pub(super) hovered_chip: Option<usize>,
    /// The index of the folder chip whose remove menu is open, if any.
    pub(super) chip_menu: Option<usize>,
    /// Whether the pointer is over the open remove menu row.
    pub(super) menu_hovered: bool,
    /// Whether the pointer is currently over the + button in the exclude box.
    pub(super) exclude_plus_hovered: bool,
    /// The exclude patterns, one string per pattern chip.
    pub(super) exclude_patterns: Vec<String>,
    /// The index of the exclude chip whose delete menu is open, if any.
    pub(super) exclude_menu: Option<usize>,
    /// Whether the pointer is over the open exclude delete menu.
    pub(super) exclude_menu_hovered: bool,
}

impl ConfigState {
    /// Resets the dialog interaction flags.
    pub(super) fn reset(&mut self) {
        self.plus_hovered = false;
        self.hovered_chip = None;
        self.chip_menu = None;
        self.menu_hovered = false;
        self.exclude_plus_hovered = false;
        self.exclude_menu = None;
        self.exclude_menu_hovered = false;
    }
}

/// Messages that drive the Config Silo dialog.
#[derive(Debug, Clone)]
pub(super) enum ConfigMsg {
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
}

/// Opens the dialog: resets the interaction flags, reloads the saved rows, and
/// starts the folder size walks.
pub(super) fn open(state: &mut SiloApp) -> Task<Message> {
    state.config.reset();
    // Load the saved settings once at open. Reloading on every redraw would
    // read the database on each frame.
    match config::load() {
        Ok(settings) => {
            state.config.folder_paths = settings.silo_data_paths;
            state.config.exclude_patterns = settings.excludes;
        }
        Err(err) => {
            eprintln!("silo: could not load the saved settings: {err}");
            state.config.folder_paths = Vec::new();
            state.config.exclude_patterns = Vec::new();
        }
    };
    // Show a placeholder size label per folder and compute the real sizes
    // asynchronously, so large folders do not freeze the UI.
    state.config.folder_sizes = vec!["...".to_string(); state.config.folder_paths.len()];
    Task::batch(super::config_silo_dialog_folders::size_tasks(
        &state.config.folder_paths,
    ))
}

/// Handles a Config Silo dialog message.
pub(super) fn update(state: &mut SiloApp, message: ConfigMsg) -> Task<Message> {
    match message {
        ConfigMsg::PlusHovered(hovered) => set_hovered(&mut state.config.plus_hovered, hovered),
        ConfigMsg::PlusPressed => {
            // Open the OS native folder picker. The picked folder arrives as
            // `ConfigMsg::FolderPicked`; the folder list is added in a later
            // step.
            pick_folder("Select a source folder", |selection| {
                Message::Config(ConfigMsg::FolderPicked(selection))
            })
        }
        ConfigMsg::FolderPicked(selection) => {
            if let Some(path) = selection {
                // Append the picked folder to the settings database: one row
                // per folder, never replacing existing rows. Duplicate paths
                // are ignored by the unique index on the `path` column.
                match config::add_data_path(&path) {
                    Ok(()) => {
                        // Keep the in-memory list in sync with the database
                        // while the dialog stays open. Duplicates are skipped,
                        // mirroring the database rule.
                        if !state.config.folder_paths.contains(&path) {
                            state.config.folder_paths.push(path.clone());
                            state.config.folder_sizes.push("...".to_string());
                            // Compute the size label for the new folder.
                            return super::config_silo_dialog_folders::size_tasks(&[path])
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
        ConfigMsg::FolderSizeComputed(path, size) => {
            // Look up the folder by path: the list may have changed while the
            // walk ran, so an index captured at task creation could point to
            // the wrong chip.
            if let Some(index) = state.config.folder_paths.iter().position(|p| p == &path) {
                if let Some(slot) = state.config.folder_sizes.get_mut(index) {
                    *slot = size;
                }
            }
            Task::none()
        }
        ConfigMsg::ChipHovered(index, hovered) => {
            if hovered {
                state.config.hovered_chip = Some(index);
            } else if state.config.hovered_chip == Some(index) {
                state.config.hovered_chip = None;
            }
            Task::none()
        }
        ConfigMsg::ChipPressed(path) => {
            // Open the folder in the native OS file explorer.
            open_in_file_explorer(&path);
            state.config.chip_menu = None;
            state.config.menu_hovered = false;
            Task::none()
        }
        ConfigMsg::ChipMenuRequested(index) => {
            // Right-pressing the same chip again collapses the menu.
            state.config.chip_menu = if state.config.chip_menu == Some(index) {
                None
            } else {
                Some(index)
            };
            state.config.menu_hovered = false;
            Task::none()
        }
        ConfigMsg::RemoveFolder(index) => {
            if let Some(path) = state.config.folder_paths.get(index) {
                match config::remove_data_path(path) {
                    Ok(()) => {
                        state.config.folder_paths.remove(index);
                        state.config.folder_sizes.remove(index);
                    }
                    Err(err) => {
                        eprintln!("silo: could not remove the folder {path:?}: {err}");
                    }
                }
            }
            state.config.chip_menu = None;
            state.config.hovered_chip = None;
            state.config.menu_hovered = false;
            Task::none()
        }
        ConfigMsg::CloseChipMenu => {
            state.config.chip_menu = None;
            state.config.menu_hovered = false;
            Task::none()
        }
        ConfigMsg::MenuHovered(hovered) => set_hovered(&mut state.config.menu_hovered, hovered),
        ConfigMsg::ExcludePlusHovered(hovered) => {
            set_hovered(&mut state.config.exclude_plus_hovered, hovered)
        }
        ConfigMsg::ExcludePlusPressed => {
            // Add a new empty pattern chip and persist the list.
            state.config.exclude_patterns.push(String::new());
            save_excludes(state);
            Task::none()
        }
        ConfigMsg::ExcludePatternChanged(index, value) => {
            if let Some(slot) = state.config.exclude_patterns.get_mut(index) {
                *slot = value;
            }
            save_excludes(state);
            Task::none()
        }
        ConfigMsg::ExcludeMenuRequested(index) => {
            // Right-pressing the same chip again collapses the menu.
            state.config.exclude_menu = if state.config.exclude_menu == Some(index) {
                None
            } else {
                Some(index)
            };
            state.config.exclude_menu_hovered = false;
            Task::none()
        }
        ConfigMsg::ExcludePatternRemoved(index) => {
            if index < state.config.exclude_patterns.len() {
                state.config.exclude_patterns.remove(index);
            }
            state.config.exclude_menu = None;
            state.config.exclude_menu_hovered = false;
            save_excludes(state);
            Task::none()
        }
        ConfigMsg::CloseExcludeMenu => {
            state.config.exclude_menu = None;
            state.config.exclude_menu_hovered = false;
            Task::none()
        }
        ConfigMsg::ExcludeMenuHovered(hovered) => {
            set_hovered(&mut state.config.exclude_menu_hovered, hovered)
        }
    }
}

/// Persist the current exclude patterns to the database.
fn save_excludes(state: &SiloApp) {
    if let Err(err) = config::replace_excludes(&state.config.exclude_patterns) {
        eprintln!("silo: could not save the exclude patterns: {err}");
    }
}
