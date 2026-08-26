//! SyncProgressBar: the sync progress bar for the Sync dialog STATUS box.
//!
//! Renders a thin progress track with a filled portion that shows the current
//! sync progress, followed by the status labels and square separators. The bar
//! is a normal flow widget: the Sync dialog composes it into the STATUS box,
//! below the divider and above the text lines.

use iced::widget::{Row, Stack, container, text};
use iced::{Element, Length};

use crate::modules::ui::scaling::sp;
use crate::modules::ui::theme::{DETAIL, TEAL};

use super::Message;
use super::sync_progress::SyncProgress;

/// The height of the progress bar track, in reference pixels.
const BAR_HEIGHT: f32 = 13.0;

/// The width of the progress bar track, in reference pixels.
const BAR_WIDTH: f32 = 175.0;

/// The font size of the labels next to the bar, in reference pixels.
/// Matches the STATUS box line text size.
const TEXT_SIZE: f32 = 18.0;

/// The horizontal spacing between the bar and the labels, in reference px.
const LABEL_SPACING: f32 = 30.0;

/// Builds the sync progress bar.
///
/// `progress` is the live sync progress, or `None` when no sync is running.
/// When idle, the labels show placeholders and the bar stays empty. The bar
/// and its labels are horizontally centered in the Sync dialog STATUS box.
pub fn view(progress: Option<&SyncProgress>) -> Element<'static, Message> {
    // The live labels, or placeholders while no sync is running.
    let (fraction, eta, sizes, percent) = match progress {
        Some(p) => (p.fraction(), p.eta_text(), p.sizes_text(), p.percent_text()),
        None => (
            0.0,
            "ETA: --".to_string(),
            "-- / --".to_string(),
            "--".to_string(),
        ),
    };

    // The track: the full bar in the detail teal-blue.
    let track = container(text(""))
        .width(Length::Fixed(sp(BAR_WIDTH)))
        .height(Length::Fixed(sp(BAR_HEIGHT)))
        .style(|_| container::Style {
            background: Some(DETAIL.into()),
            ..container::Style::default()
        });

    // The fill: a bright teal portion showing the sync progress.
    let fill = container(text(""))
        .width(Length::Fixed(sp(BAR_WIDTH * fraction)))
        .height(Length::Fixed(sp(BAR_HEIGHT)))
        .style(|_| container::Style {
            background: Some(TEAL.into()),
            ..container::Style::default()
        });

    // Stack places both children at the top-left origin, so the narrower fill
    // stays left-anchored inside the full-width track.
    let bar = Stack::new()
        .width(Length::Fixed(sp(BAR_WIDTH)))
        .height(Length::Fixed(sp(BAR_HEIGHT)))
        .push(track)
        .push(fill);

    // The bar, the labels, and the square separators in one row, centered in
    // the STATUS box.
    container(
        Row::new()
            .align_y(iced::alignment::Vertical::Center)
            .spacing(sp(LABEL_SPACING))
            .push(text(eta).size(sp(TEXT_SIZE)).color(TEAL))
            .push(super::action_area::separator())
            .push(text(sizes).size(sp(TEXT_SIZE)).color(TEAL))
            .push(super::action_area::separator())
            .push(text(percent).size(sp(TEXT_SIZE)).color(TEAL))
            .push(bar),
    )
    .width(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .into()
}
