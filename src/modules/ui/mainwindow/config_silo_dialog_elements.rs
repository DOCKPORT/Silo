//! ConfigSiloDialogElements: the panel boxes inside the Config Silo dialog.
//!
//! Two vertical rectangles laid side by side: the folder box (70% width) and
//! the exclude box (30% width), each with a `#b8b8b8` border and a title at
//! the top left. Each box has a + button at the far right of its title row.
//! The folder box hosts the selected source folders via
//! [`config_silo_dialog_folders`]; the exclude box hosts the exclude patterns
//! via [`config_silo_dialog_exclude`].

use std::path::PathBuf;

use iced::widget::{Column, Row, Space, container, text};
use iced::{Border, Element, Length, Padding};

use crate::modules::ui::scaling::sp;
use crate::modules::ui::theme::GREY;

use super::Message;
use super::action_area::{HEADER_HEIGHT, divider, plus_button};
use super::config_silo_actions::ConfigMsg;

/// The width of the box borders, in reference pixels.
const BOX_BORDER_WIDTH: f32 = 1.0;

/// The gap between the folder box and the exclude box, in reference pixels.
const BOX_SPACING: f32 = 10.0;

/// The width portion of the folder box.
const FOLDER_PART: u16 = 7;

/// The width portion of the exclude box.
const EXCLUDE_PART: u16 = 3;

/// The font size of the box titles, in reference pixels.
const TITLE_SIZE: f32 = 15.0;

/// The padding between the box border and its title, in reference pixels.
const BOX_PAD: f32 = 10.0;

/// The gap between the box title and its divider line, in ref px.
const TITLE_SPACING: f32 = 8.0;

/// Builds the two panel boxes side by side.
///
/// Returns a full-size row: the folder box takes 70% of the width and the
/// exclude box the remaining 30%, separated by a small gap. `folder_paths`
/// are the selected source folders shown in the folder box. `folder_sizes`
/// holds each folder's size label, parallel to `folder_paths`. `hovered_chip`
/// is the index of the folder chip under the pointer, if any. `chip_menu` is
/// the index of the chip whose remove menu is open, if any. `menu_hovered`
/// reports whether the pointer is over the open remove menu.
/// `exclude_plus_hovered` reports whether the pointer is over the + button in
/// the exclude box. `exclude_patterns` holds the current exclude patterns,
/// one string per pattern chip. `exclude_menu` is the index of the exclude
/// chip whose delete menu is open, if any. `exclude_menu_hovered` reports
/// whether the pointer is over that menu.
pub fn view<'a>(
    plus_hovered: bool,
    folder_paths: &[PathBuf],
    folder_sizes: &[String],
    hovered_chip: Option<usize>,
    chip_menu: Option<usize>,
    menu_hovered: bool,
    exclude_plus_hovered: bool,
    exclude_patterns: &'a [String],
    exclude_menu: Option<usize>,
    exclude_menu_hovered: bool,
) -> Element<'a, Message> {
    Row::new()
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(sp(BOX_SPACING))
        .push(folder_box(
            plus_hovered,
            folder_paths,
            folder_sizes,
            hovered_chip,
            chip_menu,
            menu_hovered,
        ))
        .push(exclude_box(
            exclude_plus_hovered,
            exclude_patterns,
            exclude_menu,
            exclude_menu_hovered,
        ))
        .into()
}

/// Builds the folder box: a bordered rectangle filling 70% of the width, with
/// a + button at the far right of the title row and the folder list from
/// [`config_silo_dialog_folders`] below the divider line.
fn folder_box<'a>(
    plus_hovered: bool,
    folder_paths: &[PathBuf],
    folder_sizes: &[String],
    hovered_chip: Option<usize>,
    chip_menu: Option<usize>,
    menu_hovered: bool,
) -> Element<'a, Message> {
    let header = Row::new()
        .width(Length::Fill)
        .height(Length::Fixed(sp(HEADER_HEIGHT)))
        .align_y(iced::alignment::Vertical::Center)
        .push(text("SELECT FOLDERS").size(sp(TITLE_SIZE)).color(GREY))
        .push(Space::new().width(Length::Fill))
        .push(plus_button(
            plus_hovered,
            Message::Config(ConfigMsg::PlusHovered(true)),
            Message::Config(ConfigMsg::PlusHovered(false)),
            Message::Config(ConfigMsg::PlusPressed),
        ))
        .into();

    boxed(
        Length::FillPortion(FOLDER_PART),
        header,
        super::config_silo_dialog_folders::view(
            folder_paths,
            folder_sizes,
            hovered_chip,
            chip_menu,
            menu_hovered,
        ),
    )
}

/// Builds the exclude box: a bordered rectangle filling 30% of the width, with
/// a + button at the far right of the title row and the exclude pattern list
/// from [`config_silo_dialog_exclude`] below the divider line.
fn exclude_box<'a>(
    exclude_plus_hovered: bool,
    patterns: &'a [String],
    exclude_menu: Option<usize>,
    exclude_menu_hovered: bool,
) -> Element<'a, Message> {
    let header = Row::new()
        .width(Length::Fill)
        .height(Length::Fixed(sp(HEADER_HEIGHT)))
        .align_y(iced::alignment::Vertical::Center)
        .push(text("EXCLUDE DATA").size(sp(TITLE_SIZE)).color(GREY))
        .push(Space::new().width(Length::Fill))
        .push(plus_button(
            exclude_plus_hovered,
            Message::Config(ConfigMsg::ExcludePlusHovered(true)),
            Message::Config(ConfigMsg::ExcludePlusHovered(false)),
            Message::Config(ConfigMsg::ExcludePlusPressed),
        ))
        .into();

    boxed(
        Length::FillPortion(EXCLUDE_PART),
        header,
        super::config_silo_dialog_exclude::view(patterns, exclude_menu, exclude_menu_hovered),
    )
}

/// Builds one bordered rectangle box with the given width, header, and body,
/// filling the height. The header sits at the top left of the box, with a
/// divider line below it that matches the box border style. The body fills
/// the area below the divider.
fn boxed<'a>(
    width: Length,
    header: Element<'a, Message>,
    body: Element<'a, Message>,
) -> Element<'a, Message> {
    let content = Column::new()
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(sp(TITLE_SPACING))
        .push(header)
        .push(divider())
        .push(body);

    container(content)
        .width(width)
        .height(Length::Fill)
        .padding(Padding {
            top: sp(BOX_PAD),
            left: sp(BOX_PAD),
            right: sp(BOX_PAD),
            bottom: sp(BOX_PAD),
        })
        .style(|_| container::Style {
            background: None,
            border: Border {
                color: GREY,
                width: sp(BOX_BORDER_WIDTH),
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}
