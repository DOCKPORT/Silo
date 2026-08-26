//! ConfigSiloDialogExclude: the exclude patterns list.
//!
//! This module owns the content area of the EXCLUDE DATA box in the Config
//! Silo dialog. It renders one chip per exclude pattern; each chip holds a
//! text input where the pattern is typed. Pressing the + button in the box
//! adds a new empty chip. Right-pressing a chip opens a dropdown menu with a
//! delete action, mirroring the folder chips.

use iced::mouse;
use iced::widget::text_input::{self, Status};
use iced::widget::{Column, MouseArea, TextInput, container, text};
use iced::{Background, Border, Element, Length, Padding, Theme};

use crate::modules::ui::scaling::sp;
use crate::modules::ui::scrollbar;
use crate::modules::ui::theme::{BACK, DETAIL, GREY, ORANGE, TEAL};

use super::Message;
use super::config_silo_actions::ConfigMsg;

/// The width of the pattern chip borders, in reference pixels.
const CHIP_BORDER_WIDTH: f32 = 1.0;

/// The gap between pattern chips, in reference pixels.
const CHIP_SPACING: f32 = 8.0;

/// The padding between a chip border and its text input, in reference pixels.
const CHIP_PAD: f32 = 10.0;

/// The padding inside the text input, in reference pixels.
const INPUT_PAD: f32 = 4.0;

/// The font size of the pattern chip text, in reference pixels. Matches the
/// folder chips so the two lists look consistent.
const CHIP_TEXT_SIZE: f32 = 19.0;

/// The font size of the delete menu text, in reference pixels.
const MENU_TEXT_SIZE: f32 = 16.0;

/// The vertical padding of the delete menu row, in reference pixels.
const MENU_PAD: f32 = 10.0;

/// Builds the exclude list area inside the EXCLUDE DATA box.
///
/// `patterns` are the current exclude patterns, one per chip. `exclude_menu`
/// is the index of the chip whose delete menu is open, if any.
/// `exclude_menu_hovered` reports whether the pointer is over that menu.
pub fn view<'a>(
    patterns: &'a [String],
    exclude_menu: Option<usize>,
    exclude_menu_hovered: bool,
) -> Element<'a, Message> {
    let column = patterns.iter().enumerate().fold(
        Column::new().width(Length::Fill).spacing(sp(CHIP_SPACING)),
        |column, (index, pattern)| {
            let mut column = column.push(pattern_chip(pattern, index));
            if exclude_menu == Some(index) {
                column = column.push(delete_menu(pattern, index, exclude_menu_hovered));
            }
            column
        },
    );

    // Pressing or right-pressing empty space dismisses an open menu. The
    // chips' own mouse areas capture the event first, so chip actions take
    // priority over this area.
    MouseArea::new(scrollbar::vertical(column))
        .on_press(Message::Config(ConfigMsg::CloseExcludeMenu))
        .on_right_press(Message::Config(ConfigMsg::CloseExcludeMenu))
        .into()
}

/// Builds one pattern chip: a bordered rectangle holding a text input.
/// Right-pressing the chip opens its delete menu.
fn pattern_chip<'a>(pattern: &'a str, index: usize) -> Element<'a, Message> {
    let input = TextInput::new("", pattern)
        .on_input(move |value| Message::Config(ConfigMsg::ExcludePatternChanged(index, value)))
        .size(sp(CHIP_TEXT_SIZE))
        .padding(sp(INPUT_PAD))
        .style(|_theme: &Theme, _status: Status| text_input::Style {
            background: Background::Color(BACK),
            border: Border::default(),
            icon: GREY,
            placeholder: GREY,
            value: TEAL,
            selection: TEAL,
        });

    let chip = container(input)
        .width(Length::Fill)
        .padding(Padding {
            top: sp(CHIP_PAD),
            left: sp(CHIP_PAD),
            right: sp(CHIP_PAD),
            bottom: sp(CHIP_PAD),
        })
        .style(|_| container::Style {
            background: None,
            border: Border {
                color: DETAIL,
                width: sp(CHIP_BORDER_WIDTH),
                radius: 0.0.into(),
            },
            ..container::Style::default()
        });

    MouseArea::new(chip)
        .on_right_press(Message::Config(ConfigMsg::ExcludeMenuRequested(index)))
        .into()
}

/// Builds the delete menu row shown under a right-clicked chip.
///
/// The row shows "Remove {pattern}", or "Remove" for an empty pattern. The
/// text is grey by default and turns ORANGE while hovered. Pressing the row
/// sends `ConfigMsg::ExcludePatternRemoved`.
fn delete_menu(pattern: &str, index: usize, hovered: bool) -> Element<'static, Message> {
    let label_text = if pattern.is_empty() {
        "Remove".to_string()
    } else {
        format!("Remove {pattern}")
    };

    let label = container(text(label_text).size(sp(MENU_TEXT_SIZE)).color(if hovered {
        ORANGE
    } else {
        GREY
    }))
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
        .on_enter(Message::Config(ConfigMsg::ExcludeMenuHovered(true)))
        .on_exit(Message::Config(ConfigMsg::ExcludeMenuHovered(false)))
        .on_press(Message::Config(ConfigMsg::ExcludePatternRemoved(index)))
        .interaction(mouse::Interaction::Pointer)
        .into()
}
