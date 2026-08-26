//! ConfigSiloDialogFolders: the selected source folders list.
//!
//! This module owns the content area of the SELECT FOLDERS box in the Config
//! Silo dialog. It renders one bordered rectangle per selected source folder,
//! showing only the folder's last path component. For example, the path
//! `/home/user/silo/theme/color_theme/` renders as `color_theme`.
//!
//! The folder paths arrive from the app state, loaded once when the dialog
//! opens. This module performs no database access; it renders the chips and
//! their size labels. Hovering a chip turns its border orange; pressing a
//! chip opens the folder in the OS file explorer.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex};

use iced::mouse;
use iced::widget::{Column, MouseArea, Row, Space, container, svg, text};
use iced::{Border, Element, Length, Padding, Task};

use crate::modules::silo_size;
use crate::modules::ui::scaling::sp;
use crate::modules::ui::scrollbar;
use crate::modules::ui::theme::{DETAIL, GREY, ORANGE, TEAL};

use super::Message;
use super::action_area::FOLDER_ICON_BYTES;
use super::config_silo_actions::ConfigMsg;

/// The width of the folder chip borders, in reference pixels.
const CHIP_BORDER_WIDTH: f32 = 3.0;

/// The gap between folder chips, in reference pixels.
const CHIP_SPACING: f32 = 8.0;

/// The font size of the folder chip text, in reference pixels.
const CHIP_TEXT_SIZE: f32 = 19.0;

/// The padding between a chip border and its content, in reference pixels.
const CHIP_PAD: f32 = 12.0;

/// The rendered size of the folder icon inside a chip, in reference pixels.
const CHIP_ICON_SIZE: f32 = 20.0;

/// The gap between the folder icon and the folder name, in reference pixels.
const CHIP_ICON_GAP: f32 = 8.0;

/// The font size of the folder size label, in reference pixels.
const CHIP_SIZE_TEXT_SIZE: f32 = 19.0;

/// The font size of the remove menu text, in reference pixels.
const MENU_TEXT_SIZE: f32 = 16.0;

/// The vertical padding of the remove menu row, in reference pixels.
const MENU_PAD: f32 = 10.0;

/// The total size of the folder in bytes, including all nested files.
///
/// Walks the tree in parallel with a small worker pool, so large folders
/// finish much faster than a single-threaded walk. Symlinks are skipped:
/// directory symlinks cannot loop back, and file symlinks are not
/// double-counted. The root must be readable; any other read failure is
/// recorded and returned after the walk, so the UI can show a fallback such
/// as `N/A`.
fn folder_size_bytes(path: &Path) -> io::Result<u64> {
    // The root must be readable; any failure here aborts.
    fs::read_dir(path)?;

    let total = AtomicU64::new(0);
    let pending = AtomicUsize::new(1);
    let queue = Mutex::new(vec![path.to_path_buf()]);
    let cv = Condvar::new();
    let first_error: Mutex<Option<io::Error>> = Mutex::new(None);
    let first_error = &first_error;

    let workers = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1)
        .min(8);

    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                loop {
                    // Take the next directory to scan, waiting while the queue
                    // is empty but other workers still have work pending.
                    let dir = {
                        let mut queue = queue.lock().unwrap();
                        loop {
                            if let Some(dir) = queue.pop() {
                                break dir;
                            }
                            if pending.load(Ordering::Acquire) == 0 {
                                return;
                            }
                            queue = cv.wait(queue).unwrap();
                        }
                    };

                    match fs::read_dir(&dir) {
                        Ok(entries) => {
                            for entry in entries {
                                let entry = match entry {
                                    Ok(entry) => entry,
                                    Err(err) => {
                                        record(first_error, err);
                                        continue;
                                    }
                                };
                                let file_type = match entry.file_type() {
                                    Ok(file_type) => file_type,
                                    Err(err) => {
                                        record(first_error, err);
                                        continue;
                                    }
                                };

                                if file_type.is_dir() {
                                    pending.fetch_add(1, Ordering::AcqRel);
                                    queue.lock().unwrap().push(entry.path());
                                    cv.notify_one();
                                } else if file_type.is_file() {
                                    match entry.metadata() {
                                        Ok(metadata) => {
                                            total.fetch_add(metadata.len(), Ordering::Relaxed);
                                        }
                                        Err(err) => record(first_error, err),
                                    }
                                }
                                // Symlinks are skipped by `file_type`: they are
                                // neither a file nor a directory for the purposes
                                // of this walk.
                            }
                        }
                        Err(err) => record(first_error, err),
                    }

                    // This directory is done. When the last pending directory
                    // finishes, wake every worker so they can exit.
                    if pending.fetch_sub(1, Ordering::AcqRel) == 1 {
                        cv.notify_all();
                    }
                }
            });
        }
    });

    if let Some(err) = first_error.lock().unwrap().take() {
        return Err(err);
    }

    Ok(total.load(Ordering::Relaxed))
}

/// Record the first error encountered during a parallel walk.
fn record(first_error: &Mutex<Option<io::Error>>, err: io::Error) {
    let mut slot = first_error.lock().unwrap();
    if slot.is_none() {
        *slot = Some(err);
    }
}

/// The folder's total data size as a human-readable string.
///
/// Returns the size formatted by [`silo_size::human_size`], or the first
/// [`io::Error`] when the folder cannot be read. Used by the folder chip size
/// label.
fn folder_size(path: &Path) -> Result<String, io::Error> {
    Ok(silo_size::human_size(folder_size_bytes(path)?))
}

/// Builds one async task per folder to compute its size label.
///
/// Each task walks its folder and maps the result to
/// `ConfigMsg::FolderSizeComputed(path, label)`. The label is matched back to
/// its chip by path when it arrives, so list changes during the walk cannot
/// misplace it. An unreadable folder maps to the label `N/A`.
pub fn size_tasks(paths: &[PathBuf]) -> Vec<Task<Message>> {
    paths
        .iter()
        .map(|path| {
            let path = path.clone();
            Task::perform(
                async move {
                    let label = folder_size(&path).unwrap_or_else(|_| "N/A".to_string());
                    Message::Config(ConfigMsg::FolderSizeComputed(path, label))
                },
                std::convert::identity,
            )
        })
        .collect()
}

/// Builds the folder list area inside the SELECT FOLDERS box.
///
/// `paths` are the selected source folders, loaded once at dialog open.
/// `sizes` holds each folder's size label, parallel to `paths`. `hovered_chip`
/// is the index of the chip under the pointer, if any. `chip_menu` is the
/// index of the chip whose remove menu is open, if any. Each folder renders
/// as one bordered chip showing the folder's last path component and size.
pub fn view<'a>(
    paths: &[PathBuf],
    sizes: &[String],
    hovered_chip: Option<usize>,
    chip_menu: Option<usize>,
    menu_hovered: bool,
) -> Element<'a, Message> {
    folder_list(paths, sizes, hovered_chip, chip_menu, menu_hovered)
}

/// Builds the remove menu row shown under a right-clicked chip.
///
/// The row shows "Remove folder (name) from Silo". The text is grey by
/// default and turns ORANGE while hovered. Pressing the row sends
/// `ConfigMsg::RemoveFolder`.
fn remove_menu<'a>(path: &Path, index: usize, hovered: bool) -> Element<'a, Message> {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    let label = container(
        text(format!("Remove folder ({name}) from Silo"))
            .size(sp(MENU_TEXT_SIZE))
            .color(if hovered { ORANGE } else { GREY }),
    )
    .width(Length::Fill)
    .padding(Padding {
        top: sp(MENU_PAD),
        left: sp(CHIP_PAD),
        right: sp(CHIP_PAD),
        bottom: sp(MENU_PAD),
    })
    .style(|_| container::Style {
        background: None,
        border: Border {
            color: GREY,
            width: sp(CHIP_BORDER_WIDTH),
            radius: 0.0.into(),
        },
        ..container::Style::default()
    });

    MouseArea::new(label)
        .on_enter(Message::Config(ConfigMsg::MenuHovered(true)))
        .on_exit(Message::Config(ConfigMsg::MenuHovered(false)))
        .on_press(Message::Config(ConfigMsg::RemoveFolder(index)))
        .interaction(mouse::Interaction::Pointer)
        .into()
}

/// Builds one bordered chip per folder, stacked in a column inside the app
/// scrollbar. The remove menu row is inserted under the right-clicked chip.
/// The list scrolls when the chips overflow the folder box.
fn folder_list<'a>(
    paths: &[PathBuf],
    sizes: &[String],
    hovered_chip: Option<usize>,
    chip_menu: Option<usize>,
    menu_hovered: bool,
) -> Element<'a, Message> {
    let column = paths.iter().zip(sizes.iter()).enumerate().fold(
        Column::new().width(Length::Fill).spacing(sp(CHIP_SPACING)),
        |column, (index, (path, size))| {
            let mut column = column.push(folder_chip(
                path,
                index,
                hovered_chip == Some(index),
                size.as_str(),
            ));
            if chip_menu == Some(index) {
                column = column.push(remove_menu(path, index, menu_hovered));
            }
            column
        },
    );

    // Pressing or right-pressing empty space dismisses an open menu. The
    // chips' own mouse areas capture the event first, so chip actions still
    // take priority over this area.
    MouseArea::new(scrollbar::vertical(column))
        .on_press(Message::Config(ConfigMsg::CloseChipMenu))
        .on_right_press(Message::Config(ConfigMsg::CloseChipMenu))
        .into()
}

/// Builds one folder chip: a bordered rectangle showing the folder icon, the
/// folder's last path component, and the size label on the right. The border
/// uses the DETAIL accent color and turns ORANGE while hovered. Pressing the
/// chip opens the folder in the OS file explorer; right-pressing it opens the
/// remove menu.
fn folder_chip<'a>(path: &Path, index: usize, hovered: bool, size: &str) -> Element<'a, Message> {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    let content = Row::new()
        .align_y(iced::alignment::Vertical::Center)
        .spacing(sp(CHIP_ICON_GAP))
        .push(
            svg::Svg::new(svg::Handle::from_memory(FOLDER_ICON_BYTES))
                .width(Length::Fixed(sp(CHIP_ICON_SIZE)))
                .height(Length::Fixed(sp(CHIP_ICON_SIZE))),
        )
        .push(text(name).size(sp(CHIP_TEXT_SIZE)).color(TEAL))
        .push(Space::new().width(Length::Fill))
        .push(
            text(size.to_string())
                .size(sp(CHIP_SIZE_TEXT_SIZE))
                .color(GREY),
        );

    let chip = container(content)
        .width(Length::Fill)
        .padding(Padding {
            top: sp(CHIP_PAD),
            left: sp(CHIP_PAD),
            right: sp(CHIP_PAD),
            bottom: sp(CHIP_PAD),
        })
        .style(move |_| container::Style {
            background: None,
            border: Border {
                color: if hovered { ORANGE } else { DETAIL },
                width: sp(CHIP_BORDER_WIDTH),
                radius: 0.0.into(),
            },
            ..container::Style::default()
        });

    MouseArea::new(chip)
        .on_enter(Message::Config(ConfigMsg::ChipHovered(index, true)))
        .on_exit(Message::Config(ConfigMsg::ChipHovered(index, false)))
        .on_press(Message::Config(ConfigMsg::ChipPressed(path.to_path_buf())))
        .on_right_press(Message::Config(ConfigMsg::ChipMenuRequested(index)))
        .interaction(mouse::Interaction::Pointer)
        .into()
}
