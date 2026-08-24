//! SyncSiloDialogElements: the panel boxes inside the Sync Silo dialog.
//!
//! A wide box at the top holds the SYNC DESTINATION. Its title row has a +
//! button at the far right that opens the OS folder picker; the picked folder
//! is saved to the `rsync_dest_path` settings table and shown below the
//! divider as a chip in the same style as the Config dialog folder chips.
//! Hovering the chip turns its border orange; pressing it opens the folder in
//! the OS file explorer; right-pressing it opens a remove menu. Below the box
//! sit the DRY-RUN and SYNC action buttons, and below them a tall STATUS box
//! fills the rest of the dialog. The source folders and excludes panels are
//! added in later steps.

use std::path::Path;

use iced::mouse;
use iced::widget::{Column, MouseArea, Row, Space, container, svg, text};
use iced::{Border, Color, Element, Length, Padding};

use crate::modules::ui::scaling::sp;
use crate::modules::ui::scrollbar;
use crate::modules::ui::theme::{DETAIL, GREY, ORANGE, TEAL};

use super::Message;
use super::status_format::{StatusKind, StatusLine};
use super::sync_progress::SyncProgress;
use super::sync_silo_actions::SyncMsg;

/// The width of the box borders, in reference pixels.
const BOX_BORDER_WIDTH: f32 = 1.0;

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

/// The width of the destination chip border, in reference pixels.
const CHIP_BORDER_WIDTH: f32 = 3.0;

/// The font size of the destination chip text, in reference pixels.
const CHIP_TEXT_SIZE: f32 = 19.0;

/// The padding between a chip border and its content, in reference pixels.
const CHIP_PAD: f32 = 12.0;

/// The rendered size of the folder icon inside a chip, in reference pixels.
const CHIP_ICON_SIZE: f32 = 20.0;

/// The gap between the folder icon and the folder name, in reference pixels.
const CHIP_ICON_GAP: f32 = 8.0;

/// The font size of the remove menu text, in reference pixels.
const MENU_TEXT_SIZE: f32 = 16.0;

/// The vertical padding of the remove menu row, in reference pixels.
const MENU_PAD: f32 = 10.0;

/// The vertical gap between the destination box and the action buttons.
const CONTENT_GAP: f32 = 20.0;

/// The horizontal gap between the DRY-RUN and SYNC buttons.
const BUTTON_SPACING: f32 = 20.0;

/// The font size of the STATUS box lines, in reference pixels.
const STATUS_TEXT_SIZE: f32 = 14.0;

/// The vertical gap between two STATUS box lines, in reference pixels.
const STATUS_LINE_SPACING: f32 = 6.0;

/// The embedded folder icon, compiled into the binary at build time.
const FOLDER_ICON_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/logo/folder_icon/folder-192.svg"
));

/// Builds the sync settings panel area.
///
/// Returns a full-size column holding the SYNC DESTINATION box at the top,
/// the DRY-RUN and SYNC action buttons below it, and a tall STATUS box that
/// fills the rest of the dialog. `dest_path` is the saved destination, or
/// `None` before one is picked. `dest_plus_hovered` reports whether the
/// pointer is over the + button. `dest_chip_hovered` reports whether the
/// pointer is over the destination chip. `dest_menu_open` reports whether the
/// remove menu is open. `dest_menu_hovered` reports whether the pointer is
/// over that menu. `dry_run_hovered` reports whether the pointer is over the
/// DRY-RUN button. `sync_run_hovered` reports whether the pointer is over the
/// SYNC button. `status` holds the lines shown in the STATUS box. `progress`
/// is the live sync progress shown at the top of the STATUS box, or `None`
/// when no sync is running. `busy` is true while a dry run or sync runs and
/// disables the DRY-RUN and SYNC buttons. The source folders and excludes
/// panels are added in later steps.
pub fn view<'a>(
    dest_path: Option<&'a Path>,
    dest_plus_hovered: bool,
    dest_chip_hovered: bool,
    dest_menu_open: bool,
    dest_menu_hovered: bool,
    dry_run_hovered: bool,
    sync_run_hovered: bool,
    status: &'a [StatusLine],
    progress: Option<&SyncProgress>,
    busy: bool,
) -> Element<'a, Message> {
    Column::new()
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(sp(CONTENT_GAP))
        .push(destination_box(
            dest_path,
            dest_plus_hovered,
            dest_chip_hovered,
            dest_menu_open,
            dest_menu_hovered,
        ))
        .push(run_buttons(dry_run_hovered, sync_run_hovered, busy))
        .push(status_box(status, progress))
        .into()
}

/// Builds the SYNC DESTINATION box: a wide bordered rectangle with the title
/// at the top left, a + button at the far right of the title row, and a
/// divider line below. The saved destination renders below the divider as one
/// chip in the Config dialog folder chip style. Right-pressing the chip opens
/// the remove menu under it; pressing empty space dismisses the menu.
fn destination_box<'a>(
    dest_path: Option<&'a Path>,
    dest_plus_hovered: bool,
    dest_chip_hovered: bool,
    dest_menu_open: bool,
    dest_menu_hovered: bool,
) -> Element<'a, Message> {
    let header: Element<'a, Message> = Row::new()
        .width(Length::Fill)
        .height(Length::Fixed(sp(HEADER_HEIGHT)))
        .align_y(iced::alignment::Vertical::Center)
        .push(text("SYNC DESTINATION").size(sp(TITLE_SIZE)).color(GREY))
        .push(Space::new().width(Length::Fill))
        .push(plus_button(
            dest_plus_hovered,
            Message::Sync(SyncMsg::DestPlusHovered(true)),
            Message::Sync(SyncMsg::DestPlusHovered(false)),
            Message::Sync(SyncMsg::DestPlusPressed),
        ))
        .into();

    let mut body = Column::new().width(Length::Fill);
    if let Some(path) = dest_path {
        body = body
            .spacing(sp(TITLE_SPACING))
            .push(destination_chip(path, dest_chip_hovered));
        if dest_menu_open {
            body = body.push(remove_menu(path, dest_menu_hovered));
        }
    }

    // Pressing or right-pressing empty space dismisses an open menu. The
    // chip's own mouse area captures the event first, so chip actions still
    // take priority over this area.
    let body = MouseArea::new(body)
        .on_press(Message::Sync(SyncMsg::CloseDestMenu))
        .on_right_press(Message::Sync(SyncMsg::CloseDestMenu));

    let content = Column::new()
        .width(Length::Fill)
        .spacing(sp(TITLE_SPACING))
        .push(header)
        .push(divider())
        .push(body);

    container(content)
        .width(Length::Fill)
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

/// Builds the saved destination chip: a bordered rectangle showing the folder
/// icon, the destination's last path component in TEAL, and the full path in
/// GREY, matching the Config dialog folder chip style. The border uses the
/// DETAIL accent color and turns ORANGE while hovered. Pressing the chip
/// opens the folder in the OS file explorer; right-pressing it opens the
/// remove menu.
fn destination_chip(path: &Path, hovered: bool) -> Element<'static, Message> {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    // The full path, with a trailing slash like a directory listing. A path
    // that already ends with a slash is kept as is.
    let raw_path = path.to_string_lossy();
    let full_path = if raw_path.ends_with('/') {
        raw_path.into_owned()
    } else {
        format!("{raw_path}/")
    };

    let content = Row::new()
        .align_y(iced::alignment::Vertical::Center)
        .spacing(sp(CHIP_ICON_GAP))
        .push(
            svg::Svg::new(svg::Handle::from_memory(FOLDER_ICON_BYTES))
                .width(Length::Fixed(sp(CHIP_ICON_SIZE)))
                .height(Length::Fixed(sp(CHIP_ICON_SIZE))),
        )
        .push(text(name).size(sp(CHIP_TEXT_SIZE)).color(TEAL))
        .push(text(full_path).size(sp(CHIP_TEXT_SIZE)).color(GREY));

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
        .on_enter(Message::Sync(SyncMsg::DestChipHovered(true)))
        .on_exit(Message::Sync(SyncMsg::DestChipHovered(false)))
        .on_press(Message::Sync(SyncMsg::DestChipPressed))
        .on_right_press(Message::Sync(SyncMsg::DestChipMenuRequested))
        .interaction(mouse::Interaction::Pointer)
        .into()
}

/// Builds the remove menu row shown under the right-clicked destination chip.
///
/// The row shows "Remove destination (name) from Silo". The text is grey by
/// default and turns ORANGE while hovered. Pressing the row sends
/// `SyncMsg::RemoveDestPath`.
fn remove_menu<'a>(path: &Path, hovered: bool) -> Element<'a, Message> {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    let label = container(
        text(format!("Remove sync destination ({name})"))
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
        .on_enter(Message::Sync(SyncMsg::DestMenuHovered(true)))
        .on_exit(Message::Sync(SyncMsg::DestMenuHovered(false)))
        .on_press(Message::Sync(SyncMsg::RemoveDestPath))
        .interaction(mouse::Interaction::Pointer)
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

/// Builds a + button: a plain + text, larger than the title but keeping the
/// header band height unchanged. The fixed height with vertical centering
/// keeps the + centered in the band. The + turns white when hovered. The
/// enter, exit, and press messages are supplied by the caller.
fn plus_button(
    hovered: bool,
    on_enter: Message,
    on_exit: Message,
    on_press: Message,
) -> Element<'static, Message> {
    let plus = text("+")
        .size(sp(PLUS_TEXT_SIZE))
        .height(Length::Fixed(sp(HEADER_HEIGHT)))
        .align_y(iced::alignment::Vertical::Center)
        .color(if hovered { Color::WHITE } else { GREY });

    MouseArea::new(plus)
        .on_enter(on_enter)
        .on_exit(on_exit)
        .on_press(on_press)
        .interaction(mouse::Interaction::Pointer)
        .into()
}

/// Builds the DRY-RUN and SYNC buttons side by side, centered under the
/// destination box. Both reuse the shared `action_area::silo_button` look.
/// While `busy` is true (a dry run or sync runs), both buttons are disabled
/// so no duplicate process can start.
fn run_buttons(
    dry_run_hovered: bool,
    sync_run_hovered: bool,
    busy: bool,
) -> Element<'static, Message> {
    let row = Row::new()
        .spacing(sp(BUTTON_SPACING))
        .push(super::action_area::silo_button(
            "DRY-RUN",
            dry_run_hovered,
            !busy,
            Message::Sync(SyncMsg::DryRunPressed),
            Message::Sync(SyncMsg::DryRunHovered(true)),
            Message::Sync(SyncMsg::DryRunHovered(false)),
        ))
        .push(super::action_area::silo_button(
            "SYNC",
            sync_run_hovered,
            !busy,
            Message::Sync(SyncMsg::SyncRunPressed),
            Message::Sync(SyncMsg::SyncRunHovered(true)),
            Message::Sync(SyncMsg::SyncRunHovered(false)),
        ));

    container(row)
        .width(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .into()
}

/// Builds the STATUS box: a bordered rectangle the same width as the
/// destination box but filling the remaining dialog height. It shows the
/// "STATUS" title at the top left with a divider line below, then the sync
/// progress bar, then `lines` as a scrollable list. Each line is colored by
/// its kind: grey for progress, teal for success, orange for failure.
/// `progress` is the live sync progress, or `None` when no sync is running.
fn status_box<'a>(
    lines: &'a [StatusLine],
    progress: Option<&SyncProgress>,
) -> Element<'a, Message> {
    let header: Element<'a, Message> = Row::new()
        .width(Length::Fill)
        .height(Length::Fixed(sp(HEADER_HEIGHT)))
        .align_y(iced::alignment::Vertical::Center)
        .push(text("STATUS").size(sp(TITLE_SIZE)).color(GREY))
        .into();

    let mut list = Column::new().spacing(sp(STATUS_LINE_SPACING));
    for line in lines {
        list = list.push(
            text(line.text.as_str())
                .size(sp(STATUS_TEXT_SIZE))
                .color(status_color(line.kind)),
        );
    }

    let content = Column::new()
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(sp(TITLE_SPACING))
        .push(header)
        .push(divider())
        .push(super::sync_progress_bar::view(progress))
        .push(scrollbar::vertical(list));

    container(content)
        .width(Length::Fill)
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

/// Maps a status line kind to its theme color.
fn status_color(kind: StatusKind) -> Color {
    match kind {
        StatusKind::Info => GREY,
        StatusKind::Success => TEAL,
        StatusKind::Error => ORANGE,
    }
}
