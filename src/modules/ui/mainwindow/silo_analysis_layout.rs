//! SiloAnalysisLayout: the SILO ANALYSIS panel below the action area.
//!
//! A bordered box in the same style as the panel boxes inside the Sync and
//! Config dialogs: a transparent fill with a 1 px `GREY` border, a grey title
//! at the top left, and a divider line below it. The panel starts below the
//! action area's bottom rule and takes most of the remaining window space with
//! side and bottom padding. While the silo is empty, a message with the
//! crosshatch pattern below it fills the box. When the silo holds data, two
//! side-by-side boxes, ALLOCATION and STATS, fill the box instead.

use iced::widget::{Column, Row, container, text};
use iced::{Border, Element, Length, Padding};

use crate::modules::silo_analysis::FileTypeStat;
use crate::modules::ui::crosshatch;
use crate::modules::ui::scaling::sp;
use crate::modules::ui::theme::GREY;

use super::Message;

/// The width of the box border, in reference pixels. Matches the panel box
/// border width inside the dialogs.
const BOX_BORDER_WIDTH: f32 = 1.0;

/// The vertical gap between the bottom of the action area and the top of the
/// panel, in reference pixels.
const ACTION_GAP: f32 = 20.0;

/// The font size of the box title, in reference pixels. Two steps larger
/// than the panel box title size inside the dialogs.
const TITLE_SIZE: f32 = 17.0;

/// The gap between the box title and its divider line, in reference pixels.
const TITLE_SPACING: f32 = 8.0;

/// The height of the box header band, in reference pixels. Matches the line
/// height of the 17 px title so the divider line stays aligned.
const HEADER_HEIGHT: f32 = 20.0;

/// The padding between the box border and its content, in reference pixels.
const BOX_PAD: f32 = 10.0;

/// The font size of the placeholder label, in reference pixels. Matches the
/// action button text size (the SYNC SILO button).
const BODY_TEXT_SIZE: f32 = 30.0;

/// The vertical gap between the divider and the placeholder label, in
/// reference pixels.
const BODY_TOP_GAP: f32 = 20.0;

/// The vertical gap between the placeholder label and the crosshatch pattern,
/// in reference pixels.
const LABEL_GAP: f32 = 20.0;

/// The horizontal padding between the box edges and the crosshatch pattern, in
/// reference pixels.
const CROSSHATCH_SIDE_PAD: f32 = 20.0;

/// The horizontal gap between the ALLOCATION and STATS boxes, in reference
/// pixels.
const BOXES_SPACING: f32 = 20.0;

/// The horizontal padding between the window edges and the panel box, in
/// reference pixels.
const SIDE_PAD: f32 = 60.0;

/// The vertical gap between the bottom of the panel box and the window edge,
/// in reference pixels.
const BOTTOM_PAD: f32 = 30.0;

/// Builds the SILO ANALYSIS panel.
///
/// Returns a full-window element: a bordered box with the "SILO ANALYSIS"
/// title at the top left and a divider line below it. The panel starts just
/// below the action area's bottom rule. The box fills the remaining window
/// width (minus the side padding) and height (minus the bottom padding), so it
/// takes most of the space below the action area. `silo_size` is the live
/// total size label; the empty-state group shows only while it reads `0 B`.
/// `allocation` feeds the ALLOCATION box chart.
pub fn view(silo_size: &str, allocation: &[FileTypeStat]) -> Element<'static, Message> {
    // The panel top: the bottom edge of the action area plus a gap.
    let top = super::action_area::content_bottom() + sp(ACTION_GAP);

    let header: Element<'static, Message> = Row::new()
        .width(Length::Fill)
        .height(Length::Fixed(sp(HEADER_HEIGHT)))
        .align_y(iced::alignment::Vertical::Center)
        .push(text("SILO ANALYSIS").size(sp(TITLE_SIZE)).color(GREY))
        .into();

    // The placeholder label sits at the top of the box, until the analysis
    // content is added.
    let label: Element<'static, Message> = container(
        text("SILO IS NOT POPULATED, PLEASE ENTER YOUR CONFIGURATIONS IN CONFIG. SILO")
            .size(sp(BODY_TEXT_SIZE))
            .color(GREY),
    )
    .width(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .padding(Padding {
        top: sp(BODY_TOP_GAP),
        left: 0.0,
        right: 0.0,
        bottom: sp(LABEL_GAP),
    })
    .into();

    // The crosshatch pattern fills the area below the label, inset from the
    // box edges on all sides.
    let crosshatch_area = container(crosshatch::overlay())
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: 0.0,
            left: sp(CROSSHATCH_SIDE_PAD),
            right: sp(CROSSHATCH_SIDE_PAD),
            bottom: sp(CROSSHATCH_SIDE_PAD),
        });

    let mut content = Column::new()
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(sp(TITLE_SPACING))
        .push(header)
        .push(divider());

    // The label and the crosshatch form one group that shows only while the
    // silo is empty (a total size of `0 B`). Otherwise the ALLOCATION and
    // STATS boxes fill the box side by side.
    if silo_size == "0 B" {
        content = content.push(label).push(crosshatch_area);
    } else {
        let boxes_row = Row::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(sp(BOXES_SPACING))
            .push(analysis_box(
                "ALLOCATION",
                super::silo_allocation_chart::view(allocation),
            ))
            .push(analysis_box("STATS", text("").into()));
        content = content.push(boxes_row);
    }

    let boxed = container(content)
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
        });

    container(boxed)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top,
            left: sp(SIDE_PAD),
            right: sp(SIDE_PAD),
            bottom: sp(BOTTOM_PAD),
        })
        .into()
}

/// Builds one analysis box: a bordered rectangle with the given title at the
/// top left and a divider line below it. `body` fills the area below the
/// divider; the box fills the available space.
fn analysis_box(title: &'static str, body: Element<'static, Message>) -> Element<'static, Message> {
    let header: Element<'static, Message> = Row::new()
        .width(Length::Fill)
        .height(Length::Fixed(sp(HEADER_HEIGHT)))
        .align_y(iced::alignment::Vertical::Center)
        .push(text(title).size(sp(TITLE_SIZE)).color(GREY))
        .into();

    let content = Column::new()
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(sp(TITLE_SPACING))
        .push(header)
        .push(divider())
        .push(body);

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

/// Builds the horizontal divider line under the box title, matching the box
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
