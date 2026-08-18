//! SyncProgressBar: the sync progress bar overlay.
//!
//! Renders a thin progress track with a filled portion that shows the current
//! sync progress, followed by the status labels and square separators. The
//! caller supplies the vertical center and left offset (in reference pixels)
//! so the shared layout constants stay owned by the action area. The bar is
//! composed into the main window through the action area's [`Stack`].

use iced::widget::{Row, Stack, container, text};
use iced::{Color, Element, Length, Padding, Shadow, Vector};

use crate::modules::ui::scaling::sp;
use crate::modules::ui::theme::{DETAIL, TEAL};

use super::Message;

/// The height of the progress bar track, in reference pixels.
pub const BAR_HEIGHT: f32 = 20.0;

/// The width of the progress bar track, in reference pixels.
pub const BAR_WIDTH: f32 = 420.0;

/// The font size of the labels next to the bar, in reference pixels.
const TEXT_SIZE: f32 = 30.0;

/// The horizontal spacing between the bar and the labels, in reference px.
/// Matches the status label row above (action area's `LABEL_SPACING`).
const LABEL_SPACING: f32 = 60.0;

/// The alpha of the fill glow, giving a subtle halo rather than a hard shadow.
const FILL_GLOW_ALPHA: f32 = 0.5;

/// The blur radius of the fill glow, in reference pixels.
const FILL_GLOW_BLUR: f32 = 6.0;

/// Builds the sync progress bar.
///
/// `progress` is the fraction of the bar that is filled, clamped to
/// `0.0..=1.0`. `center_y` and `left` are reference-pixel offsets: the row
/// (bar plus labels) is vertically centered on `center_y` and starts at
/// `left`.
pub fn view(progress: f32, center_y: f32, left: f32) -> Element<'static, Message> {
    let progress = progress.clamp(0.0, 1.0);

    // The track: the full bar in the detail teal-blue.
    let track = container(text(""))
        .width(Length::Fixed(sp(BAR_WIDTH)))
        .height(Length::Fixed(sp(BAR_HEIGHT)))
        .style(|_| container::Style {
            background: Some(DETAIL.into()),
            ..container::Style::default()
        });

    // The fill: a bright teal portion showing the sync progress, with glow.
    let fill = container(text(""))
        .width(Length::Fixed(sp(BAR_WIDTH * progress)))
        .height(Length::Fixed(sp(BAR_HEIGHT)))
        .style(|_| container::Style {
            background: Some(TEAL.into()),
            shadow: Shadow {
                color: Color {
                    a: FILL_GLOW_ALPHA,
                    ..TEAL
                },
                offset: Vector::ZERO,
                blur_radius: sp(FILL_GLOW_BLUR),
            },
            ..container::Style::default()
        });

    // Stack places both children at the top-left origin, so the narrower fill
    // stays left-anchored inside the full-width track.
    let bar = Stack::new()
        .width(Length::Fixed(sp(BAR_WIDTH)))
        .height(Length::Fixed(sp(BAR_HEIGHT)))
        .push(track)
        .push(fill);

    // The bar, the labels, and the square separators in one centered row.
    let row = Row::new()
        .align_y(iced::alignment::Vertical::Center)
        .spacing(sp(LABEL_SPACING))
        .push(text("ETA: 00:00:00").size(sp(TEXT_SIZE)).color(TEAL))
        .push(super::action_area::separator())
        .push(text("1.99GB/5.46GB").size(sp(TEXT_SIZE)).color(TEAL))
        .push(super::action_area::separator())
        .push(text("25.55%").size(sp(TEXT_SIZE)).color(TEAL))
        .push(bar);

    // The text line height dominates the row height. Center the whole row on
    // `center_y` so the bar and labels sit together in the band.
    let row_height = TEXT_SIZE * 1.2;
    let top = center_y - row_height / 2.0;

    container(row)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: sp(top),
            left: sp(left),
            bottom: 0.0,
            right: 0.0,
        })
        .into()
}
