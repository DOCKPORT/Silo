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

use iced::mouse;
use iced::widget::{Column, MouseArea, Row, Space, container, svg, text};
use iced::{Border, Element, Length, Padding, Task};

use crate::modules::ui::scaling::sp;
use crate::modules::ui::scrollbar;
use crate::modules::ui::theme::{DETAIL, GREY, ORANGE, TEAL};

use super::Message;

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

/// The embedded folder icon, compiled into the binary at build time.
const FOLDER_ICON_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/logo/folder_icon/folder-192.svg"
));

/// The total size of the folder in bytes, including all nested files.
///
/// Walks the folder recursively with an explicit stack, so deep trees cannot
/// overflow the call stack. Symlinks are skipped: directory symlinks cannot
/// loop back, and file symlinks are not double-counted. Any unreadable file
/// or sub-directory aborts the walk with the first [`io::Error`]; the UI can
/// then show a fallback such as `N/A`.
fn folder_size_bytes(path: &Path) -> io::Result<u64> {
    let mut total = 0u64;
    let mut pending = vec![path.to_path_buf()];

    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let file_type = entry.file_type()?;

            if file_type.is_dir() {
                pending.push(entry.path());
            } else if file_type.is_file() {
                total += entry.metadata()?.len();
            }
            // Symlinks are skipped by `file_type`: they are neither a file
            // nor a directory for the purposes of this walk.
        }
    }

    Ok(total)
}

/// Format a byte count as a human-readable size string.
///
/// Uses binary units: 1 KB = 1024 bytes. Whole bytes print without a
/// decimal; the larger units print with one decimal place.
fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];

    if bytes < 1024 {
        return format!("{bytes} B");
    }

    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    format!("{value:.1} {}", UNITS[unit])
}

/// The folder's total data size as a human-readable string.
///
/// Returns the size formatted by [`human_size`], or the first [`io::Error`]
/// when the folder cannot be read. Used by the folder chip size label.
fn folder_size(path: &Path) -> Result<String, io::Error> {
    Ok(human_size(folder_size_bytes(path)?))
}

/// Builds one async task per folder to compute its size label.
///
/// Each task walks its folder and maps the result to
/// `Message::FolderSizeComputed(index, label)`. An unreadable folder maps to
/// the label `N/A`.
pub fn size_tasks(paths: &[PathBuf]) -> Vec<Task<Message>> {
    paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            let path = path.clone();
            Task::perform(async move { folder_size(&path) }, move |result| {
                let label = result.unwrap_or_else(|_| "N/A".to_string());
                Message::FolderSizeComputed(index, label)
            })
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
pub fn view(
    paths: &[PathBuf],
    sizes: &[String],
    hovered_chip: Option<usize>,
    chip_menu: Option<usize>,
    menu_hovered: bool,
) -> Element<'static, Message> {
    folder_list(paths, sizes, hovered_chip, chip_menu, menu_hovered)
}

/// Builds the remove menu row shown under a right-clicked chip.
///
/// The row shows "Remove folder (name) from Silo". The text is grey by
/// default and turns ORANGE while hovered. Pressing the row sends
/// `Message::RemoveFolder`.
fn remove_menu(path: &Path, index: usize, hovered: bool) -> Element<'static, Message> {
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
        .on_enter(Message::MenuHovered(true))
        .on_exit(Message::MenuHovered(false))
        .on_press(Message::RemoveFolder(index))
        .interaction(mouse::Interaction::Pointer)
        .into()
}

/// Builds one bordered chip per folder, stacked in a column inside the app
/// scrollbar. The remove menu row is inserted under the right-clicked chip.
/// The list scrolls when the chips overflow the folder box.
fn folder_list(
    paths: &[PathBuf],
    sizes: &[String],
    hovered_chip: Option<usize>,
    chip_menu: Option<usize>,
    menu_hovered: bool,
) -> Element<'static, Message> {
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
        .on_press(Message::CloseChipMenu)
        .on_right_press(Message::CloseChipMenu)
        .into()
}

/// Builds one folder chip: a bordered rectangle showing the folder icon, the
/// folder's last path component, and the size label on the right. The border
/// uses the DETAIL accent color and turns ORANGE while hovered. Pressing the
/// chip opens the folder in the OS file explorer; right-pressing it opens the
/// remove menu.
fn folder_chip(path: &Path, index: usize, hovered: bool, size: &str) -> Element<'static, Message> {
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
        .on_enter(Message::ChipHovered(index, true))
        .on_exit(Message::ChipHovered(index, false))
        .on_press(Message::ChipPressed(path.to_path_buf()))
        .on_right_press(Message::ChipMenuRequested(index))
        .interaction(mouse::Interaction::Pointer)
        .into()
}
