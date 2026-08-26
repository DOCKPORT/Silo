//! ConfigSiloDialog: the CONFIG. SILO overlay dialog.
//!
//! Shown when the CONFIG. SILO button in the action area is pressed. A
//! centered dialog box sits on a dimmed full-window backdrop. Pressing the
//! backdrop or the CLOSE button closes the dialog. The dialog box holds the
//! folder and exclude panel boxes; a ? help button at the bottom left shows a
//! tooltip explaining both panels. The silo settings fields are added in a
//! later step.

use iced::mouse;
use iced::widget::{Column, MouseArea, Row, Stack, container, text};
use iced::{Border, Color, Element, Length, Padding};

use crate::modules::ui::crosshatch;
use crate::modules::ui::scaling::{Scaling, sp};
use crate::modules::ui::theme::{BACK, DETAIL, GREY, ORANGE, TEAL};

use super::Message;
use super::config_silo_actions::{ConfigMsg, ConfigState};

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

/// The horizontal padding of the ? help button, in reference pixels.
const HELP_PAD_H: f32 = 18.0;

/// The gap between the dialog box and the help tooltip, in reference pixels.
const HELP_TOOLTIP_GAP: f32 = 10.0;

/// The padding inside the help tooltip box, in reference pixels.
const HELP_TOOLTIP_PAD: f32 = 12.0;

/// The width of the help tooltip content, in reference pixels. The fixed
/// width wraps the explanation text into short lines.
const HELP_TOOLTIP_WIDTH: f32 = 360.0;

/// The font size of the help tooltip text, in reference pixels.
const HELP_TEXT_SIZE: f32 = 16.0;

/// The gap between the two help tooltip paragraphs, in reference pixels.
const HELP_TEXT_SPACING: f32 = 6.0;

/// Builds the Config Silo dialog overlay.
///
/// `state` holds the dialog's rows and interaction flags: the folder chips,
/// their size labels, the open menus, and the exclude pattern chips. The view
/// reads the flags directly from the state group instead of taking each flag
/// as a separate argument.
///
/// Returns a full-window overlay: a dimmed backdrop that closes the dialog on
/// press, and a dialog box holding the folder and exclude panel boxes plus
/// the ? help and CLOSE buttons. A "Configure Silo" title label sits just
/// above the box.
/// The box keeps the top position of a `TOP_ANCHOR_HEIGHT`-tall centered box,
/// so the extra height extends only downward. The settings fields are added
/// in a later step.
pub fn view<'a>(state: &'a ConfigState) -> Element<'a, Message> {
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
    .on_press(Message::CloseConfigSiloDialog)
    .interaction(mouse::Interaction::Idle);

    // The dialog content: the panel boxes on top and the help and CLOSE
    // buttons at the bottom of the box.
    let content = Column::new()
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .spacing(sp(CONTENT_SPACING))
        .push(super::config_silo_dialog_elements::view(
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
        ))
        .push(
            Row::new()
                .width(Length::Fill)
                .push(help_row(state.help_hovered))
                .push(close_row(state.close_hovered)),
        );

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
            ..container::Style::default()
        });

    // Absorbs clicks inside the box so they do not reach the backdrop. Only
    // the backdrop and the CLOSE button close the dialog.
    let box_area = MouseArea::new(dialog_box).on_press(Message::NoOp);

    // Keep the box's top edge where a `TOP_ANCHOR_HEIGHT`-tall centered box
    // would sit, so the taller box grows only downward. The anchor follows the
    // live window height, so the box stays centered as the window shrinks.
    let window_height = Scaling::global().window_height();
    let top = ((window_height - sp(TOP_ANCHOR_HEIGHT)) / 2.0).max(0.0);

    // The title label, centered just above the box. Its top edge sits
    // `TITLE_SIZE + TITLE_GAP` above the box's top edge, so the label text
    // lands with a `TITLE_GAP` gap between its bottom and the box. The label
    // is a small box of its own, filled and bordered like the dialog box.
    let title_top = top - sp(TITLE_SIZE + TITLE_GAP);
    let title_label = container(text("CONFIGURE SILO").size(sp(TITLE_SIZE)).color(TEAL))
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

    let mut stack = Stack::new()
        .push(backdrop)
        .push(crosshatch::overlay())
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
        .push(title);

    // The help tooltip floats just left of the dialog box, so it never
    // changes the box's internal layout. It is part of the dialog's own
    // layer, so the scanline overlay covers it like every dialog element.
    if state.help_hovered {
        stack = stack.push(help_tooltip_layer(top));
    }

    stack.into()
}

/// Builds the CLOSE button that closes the dialog.
///
/// The border is teal by default and turns ORANGE while the pointer hovers
/// over the button.
fn close_button(hovered: bool) -> Element<'static, Message> {
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
        .style(move |_| container::Style {
            background: None,
            border: Border {
                color: if hovered { ORANGE } else { DETAIL },
                width: sp(BORDER_WIDTH),
                radius: 0.0.into(),
            },
            ..container::Style::default()
        });

    MouseArea::new(label)
        .on_press(Message::CloseConfigSiloDialog)
        .on_enter(Message::Config(ConfigMsg::CloseHovered(true)))
        .on_exit(Message::Config(ConfigMsg::CloseHovered(false)))
        .interaction(mouse::Interaction::Pointer)
        .into()
}

/// Builds the CLOSE button row, pinned to the right side of the dialog.
///
/// Matches the Sync dialog, where the bottom buttons sit on the right edge.
fn close_row(hovered: bool) -> Element<'static, Message> {
    container(close_button(hovered))
        .width(Length::Fill)
        .align_x(iced::alignment::Horizontal::Right)
        .into()
}

/// Builds the ? help button that reveals the dialog help tooltip.
///
/// The button matches the CLOSE button look: teal text in a bordered
/// rectangle. The border turns ORANGE while the pointer hovers. The button
/// has no press action; it only reveals the tooltip on hover.
fn help_button(hovered: bool) -> Element<'static, Message> {
    let label = container(text("?").size(sp(TEXT_SIZE)).color(TEAL))
        .height(Length::Fixed(sp(CLOSE_HEIGHT)))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .padding(Padding {
            left: sp(HELP_PAD_H),
            right: sp(HELP_PAD_H),
            top: 0.0,
            bottom: 0.0,
        })
        .style(move |_| container::Style {
            background: None,
            border: Border {
                color: if hovered { ORANGE } else { DETAIL },
                width: sp(BORDER_WIDTH),
                radius: 0.0.into(),
            },
            ..container::Style::default()
        });

    MouseArea::new(label)
        .on_enter(Message::Config(ConfigMsg::HelpHovered(true)))
        .on_exit(Message::Config(ConfigMsg::HelpHovered(false)))
        .interaction(mouse::Interaction::Pointer)
        .into()
}

/// Builds the help tooltip box: two paragraphs explaining the SELECT FOLDERS
/// and EXCLUDE DATA panels.
///
/// The fixed width wraps each paragraph into short lines. The box is styled
/// to match the dialog: a dark fill with a DETAIL border.
fn help_tooltip_content() -> Element<'static, Message> {
    container(
        Column::new()
            .width(Length::Fixed(sp(HELP_TOOLTIP_WIDTH)))
            .spacing(sp(HELP_TEXT_SPACING))
            .push(
                text(
                    "SELECT FOLDER: Select source folders that make up \
                     your silo. They are mirrored to the destination.",
                )
                .size(sp(HELP_TEXT_SIZE))
                .color(GREY),
            )
            .push(
                text(
                    "EXCLUDE DATA: To skip files or folders during \
                     sync, type folder name or file type.",
                )
                .size(sp(HELP_TEXT_SIZE))
                .color(GREY),
            ),
    )
    .padding(sp(HELP_TOOLTIP_PAD))
    .style(|_| container::Style {
        background: Some(BACK.into()),
        border: Border {
            color: DETAIL,
            width: sp(BORDER_WIDTH),
            radius: 0.0.into(),
        },
        ..container::Style::default()
    })
    .into()
}

/// Builds the help tooltip layer, floating just left of the dialog box.
///
/// The tooltip box sits at the left of the box, vertically aligned with the
/// bottom button row, so it never changes the box's internal layout. It is
/// part of the dialog's own layer, so the scanline overlay covers it like
/// every dialog element. `top` is the dialog box's top edge, in pixels.
fn help_tooltip_layer(top: f32) -> Element<'static, Message> {
    // The dialog box is centered, so its left edge is half the window width
    // away from the box width.
    let window_width = Scaling::global().window_width();
    let box_left = ((window_width - sp(DIALOG_WIDTH)) / 2.0).max(0.0);
    let tooltip_width = sp(HELP_TOOLTIP_WIDTH) + 2.0 * sp(HELP_TOOLTIP_PAD);
    let left = box_left - sp(HELP_TOOLTIP_GAP) - tooltip_width;

    // Align the tooltip's top with the bottom button row's top edge.
    let button_top = top + sp(DIALOG_HEIGHT) - sp(BOTTOM_PAD) - sp(CLOSE_HEIGHT);

    container(MouseArea::new(help_tooltip_content()).on_press(Message::NoOp))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Left)
        .align_y(iced::alignment::Vertical::Top)
        .padding(Padding {
            top: button_top,
            left: left.max(0.0),
            right: 0.0,
            bottom: 0.0,
        })
        .into()
}

/// Builds the ? help button row, pinned to the left side of the dialog.
fn help_row(hovered: bool) -> Element<'static, Message> {
    container(help_button(hovered))
        .width(Length::Fill)
        .align_x(iced::alignment::Horizontal::Left)
        .into()
}
