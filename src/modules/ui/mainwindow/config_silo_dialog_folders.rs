//! ConfigSiloDialogFolders: the selected source folders list.
//!
//! This module owns the content area of the SELECT FOLDERS box in the Config
//! Silo dialog. It renders one bordered rectangle per selected source folder,
//! showing only the folder's last path component. For example, the path
//! `/home/user/silo/theme/color_theme/` renders as `color_theme`.
//!
//! The folder paths arrive from the app state, loaded once when the dialog
//! opens. This module performs no database access; it only renders. Hovering
//! a chip turns its border orange; pressing a chip opens the folder in the
//! OS file explorer.

use std::path::{Path, PathBuf};

use iced::mouse;
use iced::widget::{Column, MouseArea, container, text};
use iced::{Border, Element, Length, Padding};

use crate::modules::ui::scaling::sp;
use crate::modules::ui::theme::{DETAIL, GREY, ORANGE};

use super::Message;

/// The width of the folder chip borders, in reference pixels.
const CHIP_BORDER_WIDTH: f32 = 1.0;

/// The gap between folder chips, in reference pixels.
const CHIP_SPACING: f32 = 6.0;

/// The font size of the folder chip text, in reference pixels.
const CHIP_TEXT_SIZE: f32 = 14.0;

/// The padding between a chip border and its text, in reference pixels.
const CHIP_PAD: f32 = 8.0;

/// Builds the folder list area inside the SELECT FOLDERS box.
///
/// `paths` are the selected source folders, loaded once at dialog open.
/// `hovered_chip` is the index of the chip under the pointer, if any. Each
/// folder renders as one bordered chip showing the folder's last path
/// component.
pub fn view(paths: &[PathBuf], hovered_chip: Option<usize>) -> Element<'static, Message> {
    folder_list(paths, hovered_chip)
}

/// Builds one bordered chip per folder, stacked in a column.
fn folder_list(paths: &[PathBuf], hovered_chip: Option<usize>) -> Element<'static, Message> {
    paths
        .iter()
        .enumerate()
        .fold(
            Column::new().width(Length::Fill).spacing(sp(CHIP_SPACING)),
            |column, (index, path)| {
                column.push(folder_chip(path, index, hovered_chip == Some(index)))
            },
        )
        .into()
}

/// Builds one folder chip: a bordered rectangle showing the folder's last
/// path component. The border uses the DETAIL accent color and turns ORANGE
/// while hovered. Pressing the chip opens the folder in the OS file explorer.
fn folder_chip(path: &Path, index: usize, hovered: bool) -> Element<'static, Message> {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    let chip = container(text(name).size(sp(CHIP_TEXT_SIZE)).color(GREY))
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
        .interaction(mouse::Interaction::Pointer)
        .into()
}
