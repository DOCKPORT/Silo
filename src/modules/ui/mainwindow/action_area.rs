//! ActionArea: an overlay layer shown on top of the main window.
//!
//! This module draws a pair of orange rules across the top of the window and
//! the logo. It is composed onto the main window view through a [`Stack`], so
//! it stays above the base background. As the UI grows, this layer will host
//! the action controls (populate silo, configure sync, run sync, status).

use iced::mouse;
use iced::widget::{MouseArea, Row, Stack, container, svg, text};
use iced::{Border, Color, Element, Length, Padding, Shadow, Vector};

use crate::modules::ui::scaling::sp;
use crate::modules::ui::theme::{DETAIL, ORANGE, TEAL};

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
const LINE_THICKNESS: f32 = 20.0;

/// The gap between the top of the window and the first rule, in reference px.
const TOP_GAP: f32 = 30.0;

/// The vertical distance between the top of the first rule and the second.
const LINE_SPACING: f32 = 200.0;

/// The soft blur radius of the glow, in reference pixels.
const GLOW_BLUR: f32 = 10.0;

/// The alpha of the glow color, giving a subtle halo rather than a hard shadow.
const GLOW_ALPHA: f32 = 0.95;

/// The rendered size of the square logo, in reference pixels.
const LOGO_SIZE: f32 = 200.0;

/// The gap between the left edge of the window and the logo, in reference px.
const LOGO_LEFT_GAP: f32 = 5.0;

/// The thickness of the thin teal detail line, in reference pixels.
const DETAIL_LINE_THICKNESS: f32 = 2.0;

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

/// The gap between the right window edge and the action buttons, in reference px.
const BUTTON_RIGHT_PADDING: f32 = 30.0;

/// The width of the action button border, in reference pixels.
const BUTTON_BORDER_WIDTH: f32 = 5.0;

/// The glow alpha for the action buttons, softer than the shared glow.
const BUTTON_GLOW_ALPHA: f32 = 0.55;

/// The glow blur radius for the action buttons, in reference pixels.
const BUTTON_GLOW_BLUR: f32 = 7.0;

/// The vertical center of the band between the two orange rules.
fn band_center() -> f32 {
    TOP_GAP + (LINE_THICKNESS + LINE_SPACING) / 2.0
}

/// Builds a single orange rule, the single source of truth for the line look.
///
/// Returns a full-width, orange bar with the shared glow styling. Its
/// thickness and glow are scaled through [`sp`] so they stay consistent across
/// window sizes. Call this once per line; any change to the line's look is
/// made here.
fn rule() -> Element<'static, Message> {
    container(text(""))
        .width(Length::Fill)
        .height(sp(LINE_THICKNESS))
        .style(|_| container::Style {
            background: Some(ORANGE.into()),
            shadow: Shadow {
                color: Color {
                    r: 1.0,
                    g: 0xBF as f32 / 255.0,
                    b: 0.0,
                    a: GLOW_ALPHA,
                },
                offset: Vector::ZERO,
                blur_radius: sp(GLOW_BLUR),
            },
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

/// Builds the thin teal detail line, centered with the logo and filling the
/// width to the right of it.
fn detail_line() -> Element<'static, Message> {
    let center = band_center();

    let line = container(text(""))
        .width(Length::Fill)
        .height(sp(DETAIL_LINE_THICKNESS))
        .style(|_| container::Style {
            background: Some(DETAIL.into()),
            shadow: Shadow {
                color: Color {
                    a: GLOW_ALPHA,
                    ..DETAIL
                },
                offset: Vector::ZERO,
                blur_radius: sp(GLOW_BLUR),
            },
            ..container::Style::default()
        });

    container(line)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: sp(center - DETAIL_LINE_THICKNESS / 2.0),
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

/// Builds the status labels, vertically centered above the center line.
///
/// The labels sit to the right of the logo (same left offset as the detail
/// line) and are centered in the space between the top orange rule and the
/// center line. `is_populated` selects the live STATUS label: "POPULATED"
/// when the silo has at least one source folder, "NOT POPULATED" otherwise.
/// `silo_size` is the live total size label, for example "5.46GB".
fn status_labels(is_populated: bool, silo_size: &str) -> Element<'static, Message> {
    let region_top = TOP_GAP + LINE_THICKNESS;
    let region_bottom = band_center() - DETAIL_LINE_THICKNESS / 2.0;
    let label_top = region_top + (region_bottom - region_top - TEXT_SIZE) / 2.0;
    let left = LOGO_LEFT_GAP + LOGO_SIZE + DETAIL_GAP;

    let status = if is_populated {
        "STATUS: POPULATED"
    } else {
        "STATUS: NOT POPULATED"
    };

    let row = Row::new()
        .align_y(iced::alignment::Vertical::Center)
        .spacing(sp(LABEL_SPACING))
        .push(status_label(status.to_string()))
        .push(separator())
        .push(status_label("LAST SYNC: --/--/----".to_string()))
        .push(separator())
        .push(status_label(format!("SILO SIZE: {silo_size}")));

    container(row)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: sp(label_top),
            bottom: 0.0,
            left: sp(left),
            right: 0.0,
        })
        .into()
}

/// Builds an action button: the single source of truth for the button look.
///
/// An outlined rectangle with no fill, a `DETAIL` border and glow, and orange
/// text. When `hovered` is true, the border and glow switch to `ORANGE` and the
/// pointer cursor is shown. Pressing emits `on_press`; hovering emits
/// `on_enter`/`on_exit`. The caller supplies the messages so every button
/// shares one implementation while staying wired to its own state.
fn silo_button(
    label: &'static str,
    hovered: bool,
    on_press: Message,
    on_enter: Message,
    on_exit: Message,
) -> Element<'static, Message> {
    // The accent color: teal-blue border normally, orange on hover.
    let accent = if hovered { ORANGE } else { DETAIL };

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
            shadow: Shadow {
                color: Color {
                    a: BUTTON_GLOW_ALPHA,
                    ..accent
                },
                offset: Vector::ZERO,
                blur_radius: sp(BUTTON_GLOW_BLUR),
            },
            ..container::Style::default()
        });

    MouseArea::new(button)
        .on_press(on_press)
        .on_enter(on_enter)
        .on_exit(on_exit)
        .interaction(mouse::Interaction::Pointer)
        .into()
}

/// Positions a button on the far right, vertically centered in a band.
///
/// `region_top` and `region_bottom` bound the band (in reference pixels) the
/// button is centered within. The button is pinned to the right edge with the
/// shared right padding.
fn button_area(
    button: Element<'static, Message>,
    region_top: f32,
    region_bottom: f32,
) -> Element<'static, Message> {
    let button_top = region_top + (region_bottom - region_top - BUTTON_HEIGHT) / 2.0;

    container(button)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Right)
        .padding(Padding {
            top: sp(button_top),
            right: sp(BUTTON_RIGHT_PADDING),
            left: 0.0,
            bottom: 0.0,
        })
        .into()
}

/// Builds the ActionArea overlay element.
///
/// Returns a full-size, transparent layer holding two orange rules, the logo,
/// a thin teal detail line, and the status labels. The first rule sits
/// `TOP_GAP` from the top of the window; the second sits `LINE_SPACING` below
/// it. The logo is placed at the far left, centered between the two rules, and
/// the detail line runs from just right of the logo to the right edge, also
/// centered. The status labels sit above the center line, and the CONFIG
/// button is pinned to the far right on the same band. `is_populated` and
/// `silo_size` feed the live STATUS and SILO SIZE labels. The SYNC button
/// sits below the center line, between it and the bottom orange rule.
/// Everything is positioned via top/left padding so it stays above the base
/// background and below the scanlines.
pub fn view(
    logo_hovered: bool,
    config_hovered: bool,
    sync_hovered: bool,
    is_populated: bool,
    silo_size: &str,
) -> Element<'static, Message> {
    // The bands above and below the center line.
    let upper_top = TOP_GAP + LINE_THICKNESS;
    let upper_bottom = band_center() - DETAIL_LINE_THICKNESS / 2.0;
    let lower_top = band_center() + DETAIL_LINE_THICKNESS / 2.0;
    let lower_bottom = TOP_GAP + LINE_SPACING;

    // The progress bar: centered in the lower band, left-aligned with the
    // status labels.
    let bar_center_y = lower_top + (lower_bottom - lower_top) / 2.0;
    let bar_left = LOGO_LEFT_GAP + LOGO_SIZE + DETAIL_GAP;

    Stack::new()
        .push(rule_at(TOP_GAP))
        .push(rule_at(TOP_GAP + LINE_SPACING))
        .push(logo(logo_hovered))
        .push(detail_line())
        .push(status_labels(is_populated, silo_size))
        .push(super::sync_progress_bar::view(0.25, bar_center_y, bar_left))
        .push(button_area(
            silo_button(
                "CONFIG. SILO",
                config_hovered,
                Message::OpenConfigSiloDialog,
                Message::ConfigHovered(true),
                Message::ConfigHovered(false),
            ),
            upper_top,
            upper_bottom,
        ))
        .push(button_area(
            silo_button(
                "SYNC SILO",
                sync_hovered,
                Message::OpenSyncSiloDialog,
                Message::SyncHovered(true),
                Message::SyncHovered(false),
            ),
            lower_top,
            lower_bottom,
        ))
        .into()
}
