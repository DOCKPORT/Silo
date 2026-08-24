//! SyncSiloDialog: the SYNC SILO overlay dialog.
//!
//! Shown when the SYNC SILO button in the action area is pressed. A
//! centered dialog box sits on a dimmed full-window backdrop. Pressing the
//! backdrop or the CLOSE button closes the dialog. The dialog box holds the
//! sync settings elements; the destination, source folders, and excludes
//! panels are added in a later step.

use std::path::Path;

use iced::mouse;
use iced::widget::{Column, MouseArea, Stack, container, text};
use iced::{Border, Color, Element, Length, Padding, Shadow, Vector};

use crate::modules::ui::scaling::{Scaling, sp};
use crate::modules::ui::theme::{BACK, DETAIL, TEAL};

use super::{Message, StatusLine};

/// The width of the dialog box, in reference pixels.
const DIALOG_WIDTH: f32 = 900.0;

/// The height of the dialog box, in reference pixels.
const DIALOG_HEIGHT: f32 = 660.0;

/// The previous dialog height that anchors the top edge, in reference pixels.
/// The box keeps the top position it had at this height and extends downward.
const TOP_ANCHOR_HEIGHT: f32 = 600.0;

/// The gap between the bottom of the box and the CLOSE button, in ref px.
const BOTTOM_PAD: f32 = 40.0;

/// The gap between the panel boxes and the CLOSE button, in reference pixels.
const CONTENT_SPACING: f32 = 20.0;

/// The padding between the dialog box border and its content, in ref px.
const CONTENT_PAD: f32 = 30.0;

/// The width of the dialog border, in reference pixels.
const BORDER_WIDTH: f32 = 2.0;

/// The alpha of the dimmed backdrop behind the dialog box.
const BACKDROP_ALPHA: f32 = 0.90;

/// The font size of the title label above the box, in reference pixels.
const TITLE_SIZE: f32 = 30.0;

/// The gap between the title label and the top of the dialog box, in ref px.
const TITLE_GAP: f32 = 7.0;

/// The horizontal padding of the title label, in reference pixels.
const TITLE_PAD_H: f32 = 40.0;

/// The font size of the CLOSE button text, in reference pixels.
const TEXT_SIZE: f32 = 22.0;

/// The height of the CLOSE button, in reference pixels.
const CLOSE_HEIGHT: f32 = 44.0;

/// The horizontal padding of the CLOSE button, in reference pixels.
const CLOSE_PAD_H: f32 = 28.0;

/// Builds the Sync Silo dialog overlay.
///
/// `dest_path` is the saved rsync destination, or `None` before one is picked.
/// `dest_plus_hovered` reports whether the pointer is over the + button in the
/// destination box. `dest_chip_hovered` reports whether the pointer is over
/// the destination chip. `dest_menu_open` reports whether the remove menu is
/// open. `dest_menu_hovered` reports whether the pointer is over that menu.
/// `dry_run_hovered` reports whether the pointer is over the DRY-RUN button.
/// `sync_run_hovered` reports whether the pointer is over the SYNC button.
/// `status` holds the lines shown in the STATUS box.
///
/// Returns a full-window overlay: a dimmed backdrop that closes the dialog on
/// press, and a dialog box holding the sync settings elements and the CLOSE
/// button. A "Sync Silo" title label sits just above the box. The box keeps
/// the top position of a `TOP_ANCHOR_HEIGHT`-tall centered box, so the extra
/// height extends only downward. The source folders and excludes panels are
/// added in a later step.
pub fn view<'a>(
    dest_path: Option<&'a Path>,
    dest_plus_hovered: bool,
    dest_chip_hovered: bool,
    dest_menu_open: bool,
    dest_menu_hovered: bool,
    dry_run_hovered: bool,
    sync_run_hovered: bool,
    status: &'a [StatusLine],
) -> Element<'a, Message> {
    // The dimmed backdrop. Pressing it closes the dialog.
    let backdrop = MouseArea::new(
        container(text(""))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(
                    Color {
                        a: BACKDROP_ALPHA,
                        ..BACK
                    }
                    .into(),
                ),
                ..container::Style::default()
            }),
    )
    .on_press(Message::CloseSyncSiloDialog);

    // The dialog content: the sync settings elements on top and the CLOSE
    // button at the bottom of the box.
    let content = Column::new()
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .spacing(sp(CONTENT_SPACING))
        .push(super::sync_silo_dialog_elements::view(
            dest_path,
            dest_plus_hovered,
            dest_chip_hovered,
            dest_menu_open,
            dest_menu_hovered,
            dry_run_hovered,
            sync_run_hovered,
            status,
        ))
        .push(close_button());

    let dialog_box = container(content)
        .width(Length::Fixed(sp(DIALOG_WIDTH)))
        .height(Length::Fixed(sp(DIALOG_HEIGHT)))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Bottom)
        .padding(Padding {
            top: sp(CONTENT_PAD),
            left: sp(CONTENT_PAD),
            right: sp(CONTENT_PAD),
            bottom: sp(BOTTOM_PAD),
        })
        .style(|_| container::Style {
            background: Some(BACK.into()),
            border: Border {
                color: DETAIL,
                width: sp(BORDER_WIDTH),
                radius: 0.0.into(),
            },
            shadow: Shadow {
                color: Color { a: 0.5, ..DETAIL },
                offset: Vector::ZERO,
                blur_radius: sp(12.0),
            },
            ..container::Style::default()
        });

    // Absorbs clicks inside the box so they do not reach the backdrop. Only
    // the backdrop and the CLOSE button close the dialog.
    let box_area = MouseArea::new(dialog_box).on_press(Message::NoOp);

    // Keep the box's top edge where a `TOP_ANCHOR_HEIGHT`-tall centered box
    // would sit, so the taller box grows only downward.
    let window_height = Scaling::global().screen_size.height;
    let top = (window_height - sp(TOP_ANCHOR_HEIGHT)) / 2.0;

    // The title label, centered just above the box. Its top edge sits
    // `TITLE_SIZE + TITLE_GAP` above the box's top edge, so the label text
    // lands with a `TITLE_GAP` gap between its bottom and the box. The label
    // is a small box of its own, filled and bordered like the dialog box.
    let title_top = top - sp(TITLE_SIZE + TITLE_GAP);
    let title_label = container(text("SYNC SILO").size(sp(TITLE_SIZE)).color(TEAL))
        .padding(Padding {
            left: sp(TITLE_PAD_H),
            right: sp(TITLE_PAD_H),
            top: 0.0,
            bottom: 0.0,
        })
        .style(|_| container::Style {
            background: Some(BACK.into()),
            border: Border {
                color: DETAIL,
                width: sp(BORDER_WIDTH),
                radius: 0.0.into(),
            },
            ..container::Style::default()
        });

    let title = container(title_label)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .padding(Padding {
            top: title_top,
            left: 0.0,
            right: 0.0,
            bottom: 0.0,
        });

    Stack::new()
        .push(backdrop)
        .push(
            container(box_area)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .padding(Padding {
                    top,
                    left: 0.0,
                    right: 0.0,
                    bottom: 0.0,
                }),
        )
        .push(title)
        .into()
}

/// Builds the CLOSE button that closes the dialog.
fn close_button() -> Element<'static, Message> {
    let label = container(text("CLOSE").size(sp(TEXT_SIZE)).color(TEAL))
        .height(Length::Fixed(sp(CLOSE_HEIGHT)))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .padding(Padding {
            left: sp(CLOSE_PAD_H),
            right: sp(CLOSE_PAD_H),
            top: 0.0,
            bottom: 0.0,
        })
        .style(|_| container::Style {
            background: None,
            border: Border {
                color: DETAIL,
                width: sp(BORDER_WIDTH),
                radius: 0.0.into(),
            },
            ..container::Style::default()
        });

    MouseArea::new(label)
        .on_press(Message::CloseSyncSiloDialog)
        .interaction(mouse::Interaction::Pointer)
        .into()
}
