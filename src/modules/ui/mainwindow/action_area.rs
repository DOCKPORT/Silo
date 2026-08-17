//! ActionArea: an overlay layer shown on top of the main window.
//!
//! This module draws a pair of orange rules across the top of the window and
//! the logo. It is composed onto the main window view through a [`Stack`], so
//! it stays above the base background. As the UI grows, this layer will host
//! the action controls (populate silo, configure sync, run sync, status).

use iced::mouse;
use iced::widget::{container, svg, text, MouseArea, Stack};
use iced::{Color, Element, Length, Padding, Shadow, Vector};

use crate::modules::ui::scaling::sp;
use crate::modules::ui::theme::{DETAIL, ORANGE};

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

/// Builds the ActionArea overlay element.
///
/// Returns a full-size, transparent layer holding two orange rules, the logo,
/// and a thin teal detail line. The first rule sits `TOP_GAP` from the top of
/// the window; the second sits `LINE_SPACING` below it. The logo is placed at
/// the far left, centered between the two rules, and the detail line runs from
/// just right of the logo to the right edge, also centered. Everything is
/// positioned via top/left padding so it stays above the base background and
/// below the scanlines.
pub fn view(logo_hovered: bool) -> Element<'static, Message> {
    Stack::new()
        .push(rule_at(TOP_GAP))
        .push(rule_at(TOP_GAP + LINE_SPACING))
        .push(logo(logo_hovered))
        .push(detail_line())
        .into()
}
