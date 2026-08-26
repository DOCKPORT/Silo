//! ActionArea: an overlay layer shown on top of the main window.
//!
//! This module draws a pair of orange rules across the top of the window and
//! the logo. It is composed onto the main window view through a [`Stack`], so
//! it stays above the base background. As the UI grows, this layer will host
//! the action controls (populate silo, configure sync, run sync, status).

use iced::mouse;
use iced::widget::{MouseArea, Row, Stack, container, svg, text};
use iced::{Border, Color, Element, Length, Padding};

use crate::modules::ui::scaling::sp;
use crate::modules::ui::theme::{DETAIL, GREY, ORANGE, TEAL};

use super::Message;

/// The embedded dim logo (idle state), compiled into the binary at build time.
const LOGO_DIM_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/logo/silo_logo_dim.svg"
));

/// The embedded bright logo (hover state), compiled into the binary.
const LOGO_BRIGHT_BYTES: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/logo/silo_logo.svg"));

/// The thickness of a rule, in reference-resolution pixels.
const LINE_THICKNESS: f32 = 18.0;

/// The gap between the top of the window and the first rule, in reference px.
const TOP_GAP: f32 = 30.0;

/// The vertical distance between the top of the first rule and the second.
const LINE_SPACING: f32 = 200.0;

/// The rendered size of the square logo, in reference pixels.
const LOGO_SIZE: f32 = 200.0;

/// The gap between the left edge of the window and the logo, in reference px.
const LOGO_LEFT_GAP: f32 = 5.0;

/// The thickness of the thin middle detail line, in reference pixels.
const DETAIL_LINE_THICKNESS: f32 = 1.0;

/// The gap between the logo's right edge and the start of the detail line.
const DETAIL_GAP: f32 = 5.0;

/// The font size of the status labels, in reference pixels.
const TEXT_SIZE: f32 = 30.0;

/// The horizontal spacing between the two status labels, in reference px.
const LABEL_SPACING: f32 = 60.0;

/// The size of the square separator icons, in reference pixels.
const SEPARATOR_SIZE: f32 = 10.0;

/// The font size of the action button text, in reference pixels.
const BUTTON_TEXT_SIZE: f32 = 30.0;

/// The fixed height of the action buttons, in reference pixels.
const BUTTON_HEIGHT: f32 = 50.0;

/// The fixed width shared by all action buttons, in reference pixels.
/// Chosen to comfortably fit the longest label ("CONFIG. SILO").
const BUTTON_WIDTH: f32 = 300.0;

/// The internal horizontal padding of the action buttons, in reference px.
const BUTTON_PAD_H: f32 = 30.0;

/// The horizontal gap between the two action buttons, in reference px.
const BUTTON_SPACING: f32 = 20.0;

/// The width of the action button border, in reference pixels.
const BUTTON_BORDER_WIDTH: f32 = 5.0;

/// The vertical center of the band between the two orange rules.
fn band_center() -> f32 {
    TOP_GAP + (LINE_THICKNESS + LINE_SPACING) / 2.0
}

/// The bottom edge of the second orange rule, in reference pixels.
///
/// The content below the action area (the SILO ANALYSIS panel) starts below
/// this line.
pub(super) fn content_bottom() -> f32 {
    TOP_GAP + LINE_SPACING + LINE_THICKNESS
}

/// Builds a single orange rule, the single source of truth for the line look.
///
/// Returns a full-width, orange bar. Its thickness is scaled through [`sp`]
/// so it stays consistent across window sizes. Call this once per line; any
/// change to the line's look is made here.
fn rule() -> Element<'static, Message> {
    container(text(""))
        .width(Length::Fill)
        .height(sp(LINE_THICKNESS))
        .style(|_| container::Style {
            background: Some(ORANGE.into()),
            ..container::Style::default()
        })
        .into()
}

/// Positions a [`rule`] so its top sits `top` reference-pixels from the window.
fn rule_at(top: f32) -> Element<'static, Message> {
    container(rule())
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: sp(top),
            bottom: 0.0,
            left: 0.0,
            right: 0.0,
        })
        .into()
}

/// Builds the logo, centered vertically between the two rules and at the left.
///
/// When `hovered` is true, the bright logo is shown; otherwise the dim one.
/// Hovering over the logo emits [`Message::LogoHovered`] and shows a pointer
/// cursor.
fn logo(hovered: bool) -> Element<'static, Message> {
    // The vertical center of the band between the two rules.
    let logo_top = band_center() - LOGO_SIZE / 2.0;

    let bytes = if hovered {
        LOGO_BRIGHT_BYTES
    } else {
        LOGO_DIM_BYTES
    };

    let artwork = svg::Svg::new(svg::Handle::from_memory(bytes))
        .width(Length::Fixed(sp(LOGO_SIZE)))
        .height(Length::Fixed(sp(LOGO_SIZE)));

    let area = MouseArea::new(artwork)
        .on_press(Message::LogoPressed)
        .on_enter(Message::LogoHovered(true))
        .on_exit(Message::LogoHovered(false))
        .interaction(mouse::Interaction::Pointer);

    container(area)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: sp(logo_top),
            bottom: 0.0,
            left: sp(LOGO_LEFT_GAP),
            right: 0.0,
        })
        .into()
}

/// Builds the thin grey detail line, centered with the logo and filling the
/// width to the right of it.
fn detail_line() -> Element<'static, Message> {
    let center = band_center();

    // At small window sizes the scaled thickness can round below one real
    // pixel, which makes the line vanish. Clamp it so the line always shows.
    let thickness = sp(DETAIL_LINE_THICKNESS).max(1.0);

    let line = container(text(""))
        .width(Length::Fill)
        .height(thickness)
        .style(|_| container::Style {
            // 50% opacity so the line reads as a divider, not a border.
            background: Some(Color { a: 0.5, ..GREY }.into()),
            ..container::Style::default()
        });

    container(line)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: sp(center) - thickness / 2.0,
            bottom: 0.0,
            left: sp(LOGO_LEFT_GAP + LOGO_SIZE + DETAIL_GAP),
            right: 0.0,
        })
        .into()
}

/// Builds a teal status label, sized for the action area.
fn status_label(content: String) -> Element<'static, Message> {
    text(content).size(sp(TEXT_SIZE)).color(TEAL).into()
}

/// Builds a small filled square used as a separator between labels.
///
/// Shared with the other main-window overlays through `pub(super)`.
pub(super) fn separator() -> Element<'static, Message> {
    container(text(""))
        .width(Length::Fixed(sp(SEPARATOR_SIZE)))
        .height(Length::Fixed(sp(SEPARATOR_SIZE)))
        .style(|_| container::Style {
            background: Some(DETAIL.into()),
            ..container::Style::default()
        })
        .into()
}

/// Builds the SILO SIZE label, vertically centered above the center line.
///
/// The label row is centered horizontally in the window and vertically in
/// the space between the top orange rule and the center line. `silo_size`
/// is the live total size label, for example "5.5 GiB".
fn status_labels(silo_size: &str) -> Element<'static, Message> {
    let region_top = TOP_GAP + LINE_THICKNESS;
    let region_bottom = band_center() - DETAIL_LINE_THICKNESS / 2.0;
    let label_top = region_top + (region_bottom - region_top - TEXT_SIZE) / 2.0;

    let row = Row::new()
        .align_y(iced::alignment::Vertical::Center)
        .spacing(sp(LABEL_SPACING))
        .push(status_label(format!("SILO SIZE: {silo_size}")));

    container(row)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .padding(Padding {
            top: sp(label_top),
            bottom: 0.0,
            left: 0.0,
            right: 0.0,
        })
        .into()
}

/// Builds an action button: the single source of truth for the button look.
///
/// An outlined rectangle with no fill, a `DETAIL` border, and teal text. When
/// `hovered` is true, the border switches to `ORANGE` and the pointer cursor
/// is shown. When `enabled` is false, the button keeps its normal look and
/// ignores presses, but it still tracks hover so the flag stays accurate
/// when the button is enabled again. Pressing emits `on_press`; hovering
/// emits `on_enter`/`on_exit`. The caller supplies the messages so every
/// button shares one implementation while staying wired to its own state.
pub(crate) fn silo_button(
    label: &'static str,
    hovered: bool,
    enabled: bool,
    on_press: Message,
    on_enter: Message,
    on_exit: Message,
) -> Element<'static, Message> {
    // The accent color: teal-blue border normally, orange on hover. A busy
    // button keeps its normal look but does not hover or respond.
    let accent = if enabled && hovered { ORANGE } else { DETAIL };

    let button = container(text(label).size(sp(BUTTON_TEXT_SIZE)).color(TEAL))
        .width(Length::Fixed(sp(BUTTON_WIDTH)))
        .height(Length::Fixed(sp(BUTTON_HEIGHT)))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .padding(Padding {
            left: sp(BUTTON_PAD_H),
            right: sp(BUTTON_PAD_H),
            top: 0.0,
            bottom: 0.0,
        })
        .style(move |_| container::Style {
            background: None,
            border: Border {
                color: accent,
                width: sp(BUTTON_BORDER_WIDTH),
                radius: 0.0.into(),
            },
            ..container::Style::default()
        });

    let mut area = MouseArea::new(button).on_enter(on_enter).on_exit(on_exit);
    if enabled {
        area = area
            .on_press(on_press)
            .interaction(mouse::Interaction::Pointer);
    }
    area.into()
}

/// Builds the CONFIG. SILO and SYNC SILO buttons in one centered row.
///
/// The row sits in the band below the center line, centered horizontally in
/// the window.
fn action_buttons(config_hovered: bool, sync_hovered: bool) -> Element<'static, Message> {
    let lower_top = band_center() + DETAIL_LINE_THICKNESS / 2.0;
    let lower_bottom = TOP_GAP + LINE_SPACING;
    let button_top = lower_top + (lower_bottom - lower_top - BUTTON_HEIGHT) / 2.0;

    let row = Row::new()
        .spacing(sp(BUTTON_SPACING))
        .push(silo_button(
            "CONFIG. SILO",
            config_hovered,
            true,
            Message::OpenConfigSiloDialog,
            Message::ConfigHovered(true),
            Message::ConfigHovered(false),
        ))
        .push(silo_button(
            "SYNC SILO",
            sync_hovered,
            true,
            Message::OpenSyncSiloDialog,
            Message::SyncHovered(true),
            Message::SyncHovered(false),
        ));

    container(row)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .padding(Padding {
            top: sp(button_top),
            left: 0.0,
            right: 0.0,
            bottom: 0.0,
        })
        .into()
}

/// Builds the ActionArea overlay element.
///
/// Returns a full-size, transparent layer holding two orange rules, the logo,
/// a thin grey detail line, the SILO SIZE label, and the two action buttons.
/// The first rule sits `TOP_GAP` from the top of the window; the second sits
/// `LINE_SPACING` below it. The logo is placed at the far left, centered
/// between the two rules, and the detail line runs from just right of the
/// logo to the right edge, also centered. The SILO SIZE label sits above the
/// center line. The CONFIG. SILO and SYNC SILO buttons share one row below
/// the center line, centered horizontally. `silo_size` feeds the live
/// SILO SIZE label. Everything is positioned via top/left padding so it
/// stays above the base background and below the scanlines.
pub fn view(
    logo_hovered: bool,
    config_hovered: bool,
    sync_hovered: bool,
    silo_size: &str,
) -> Element<'static, Message> {
    Stack::new()
        .push(rule_at(TOP_GAP))
        .push(rule_at(TOP_GAP + LINE_SPACING))
        .push(logo(logo_hovered))
        .push(detail_line())
        .push(status_labels(silo_size))
        .push(action_buttons(config_hovered, sync_hovered))
        .into()
}

// ---- Shared dialog helpers -------------------------------------------------

/// The height of a dialog panel box header band, in reference pixels. Matches
/// the line height of the 15 px titles so the divider lines stay aligned.
pub(super) const HEADER_HEIGHT: f32 = 18.0;

/// The font size of the + button text in dialog panel boxes, in reference
/// pixels. Larger than the titles but does not change the header band height.
pub(super) const PLUS_TEXT_SIZE: f32 = 22.0;

/// The embedded folder icon used by the dialog folder chips, compiled into
/// the binary at build time.
pub(super) const FOLDER_ICON_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/logo/folder_icon/folder-192.svg"
));

/// The thickness of a divider line under a box title, in reference pixels.
/// Matches the 1 px dialog box borders.
const DIVIDER_THICKNESS: f32 = 1.0;

/// Builds the horizontal divider line under a box title, matching the box
/// border style.
pub(super) fn divider() -> Element<'static, Message> {
    container(text(""))
        .width(Length::Fill)
        .height(sp(DIVIDER_THICKNESS))
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
pub(super) fn plus_button(
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
