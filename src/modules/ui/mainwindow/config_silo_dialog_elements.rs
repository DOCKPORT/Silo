//! ConfigSiloDialogElements: the panel boxes inside the Config Silo dialog.
//!
//! Two vertical rectangles laid side by side: the folder box (70% width) and
//! the exclude box (30% width), each with a `#b8b8b8` border and a title at
//! the top left. The folder box has a + button at the far right of its title
//! row. The boxes will later host the selected source folders and the exclude
//! patterns.

use iced::mouse;
use iced::widget::{Column, MouseArea, Row, Space, container, text};
use iced::{Border, Color, Element, Length, Padding};

use crate::modules::ui::scaling::sp;
use crate::modules::ui::theme::GREY;

use super::Message;

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

/// The height of the box header band, in reference pixels. Matches the line
/// height of the 15 px titles so the divider lines stay aligned.
const HEADER_HEIGHT: f32 = 18.0;

/// The font size of the + button text, in reference pixels. The + is larger
/// than the titles but does not change the header band height.
const PLUS_TEXT_SIZE: f32 = 22.0;

/// Builds the two panel boxes side by side.
///
/// Returns a full-size row: the folder box takes 70% of the width and the
/// exclude box the remaining 30%, separated by a small gap.
pub fn view(plus_hovered: bool) -> Element<'static, Message> {
    Row::new()
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(sp(BOX_SPACING))
        .push(folder_box(plus_hovered))
        .push(exclude_box())
        .into()
}

/// Builds the folder box: a bordered rectangle filling 70% of the width, with
/// a + button at the far right of the title row.
fn folder_box(plus_hovered: bool) -> Element<'static, Message> {
    let header = Row::new()
        .width(Length::Fill)
        .height(Length::Fixed(sp(HEADER_HEIGHT)))
        .align_y(iced::alignment::Vertical::Center)
        .push(text("SELECT FOLDERS").size(sp(TITLE_SIZE)).color(GREY))
        .push(Space::new().width(Length::Fill))
        .push(plus_button(plus_hovered))
        .into();

    boxed(Length::FillPortion(FOLDER_PART), header)
}

/// Builds the exclude box: a bordered rectangle filling 30% of the width.
fn exclude_box() -> Element<'static, Message> {
    let header = Row::new()
        .width(Length::Fill)
        .height(Length::Fixed(sp(HEADER_HEIGHT)))
        .align_y(iced::alignment::Vertical::Center)
        .push(text("EXCLUDE DATA").size(sp(TITLE_SIZE)).color(GREY))
        .into();

    boxed(Length::FillPortion(EXCLUDE_PART), header)
}

/// Builds one bordered rectangle box with the given width and header, filling
/// the height. The header sits at the top left of the box, with a divider
/// line below it that matches the box border style.
fn boxed(width: Length, header: Element<'static, Message>) -> Element<'static, Message> {
    let content = Column::new()
        .width(Length::Fill)
        .spacing(sp(TITLE_SPACING))
        .push(header)
        .push(divider());

    container(content)
        .width(width)
        .height(Length::Fill)
        .padding(Padding {
            top: sp(BOX_PAD),
            left: sp(BOX_PAD),
            right: sp(BOX_PAD),
            bottom: 0.0,
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

/// Builds the horizontal divider line under a box title, matching the box
/// border style.
fn divider() -> Element<'static, Message> {
    container(text(""))
        .width(Length::Fill)
        .height(sp(BOX_BORDER_WIDTH))
        .style(|_| container::Style {
            background: Some(GREY.into()),
            ..container::Style::default()
        })
        .into()
}

/// Builds the + button: a plain + text, larger than the title but keeping the
/// header band height unchanged. The fixed height with vertical centering
/// keeps the + centered in the band. The + turns white when hovered. Style
/// only, no action.
fn plus_button(hovered: bool) -> Element<'static, Message> {
    let plus = text("+")
        .size(sp(PLUS_TEXT_SIZE))
        .height(Length::Fixed(sp(HEADER_HEIGHT)))
        .align_y(iced::alignment::Vertical::Center)
        .color(if hovered { Color::WHITE } else { GREY });

    MouseArea::new(plus)
        .on_enter(Message::PlusHovered(true))
        .on_exit(Message::PlusHovered(false))
        .interaction(mouse::Interaction::Pointer)
        .into()
}
