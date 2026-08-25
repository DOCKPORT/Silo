//! SiloAllocationChart: the file-type allocation chart for the ALLOCATION box.
//!
//! A vertical list of rows, one per file type. Each row shows the file
//! extension and its share of the total silo size as a percentage. The input
//! is ordered by total bytes descending, so the biggest share sits on top.
//! Many file types scroll through the shared app scrollbar.

use iced::widget::{Column, Row, Space, container, text};
use iced::{Element, Length, Padding};

use crate::modules::silo_analysis::FileTypeStat;
use crate::modules::ui::scaling::sp;
use crate::modules::ui::scrollbar;
use crate::modules::ui::theme::{GREY, TEAL, ZEBRA};

use super::Message;

/// The font size of the chart row text, in reference pixels.
const ROW_TEXT_SIZE: f32 = 16.0;

/// The vertical gap between two chart rows, in reference pixels.
const ROW_SPACING: f32 = 6.0;

/// The vertical padding between the chart area and the ALLOCATION box border,
/// in reference pixels.
const CHART_PAD: f32 = 10.0;

/// The horizontal padding between the chart area and the ALLOCATION box
/// edges, in reference pixels.
const CHART_SIDE_PAD: f32 = 30.0;

/// Builds the allocation chart.
///
/// Renders one row per file type in the input order, which is already ordered
/// by total bytes descending, so the biggest share is at the top. Each row
/// shows the extension and its percentage of the total silo size. The rows
/// scroll when they do not fit the box.
pub fn view(allocation: &[FileTypeStat]) -> Element<'static, Message> {
    let mut rows = Column::new().spacing(sp(ROW_SPACING));

    for (index, stat) in allocation.iter().enumerate() {
        rows = rows.push(row(stat, index));
    }

    // The padding insets the rows only; the scrollable fills the whole box, so
    // the scrollbar stays flush with the ALLOCATION box edge.
    let chart = container(rows)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: sp(CHART_PAD),
            left: sp(CHART_SIDE_PAD),
            right: sp(CHART_SIDE_PAD),
            bottom: sp(CHART_PAD),
        });

    scrollbar::vertical(chart).into()
}

/// Builds one chart row: the extension on the left and its share percentage
/// on the right. Zebra striping colors every other row with the dark grey
/// stripe; the rows in between keep the box background.
fn row(stat: &FileTypeStat, index: usize) -> Element<'static, Message> {
    let content: Element<'static, Message> = Row::new()
        .width(Length::Fill)
        .align_y(iced::alignment::Vertical::Center)
        .push(
            text(extension_label(&stat.extension))
                .size(sp(ROW_TEXT_SIZE))
                .color(GREY),
        )
        .push(Space::new().width(Length::Fill))
        .push(
            text(percent_label(stat.percent_of_total_bytes))
                .size(sp(ROW_TEXT_SIZE))
                .color(TEAL),
        )
        .into();

    container(content)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: if index % 2 == 1 {
                Some(ZEBRA.into())
            } else {
                None
            },
            ..container::Style::default()
        })
        .into()
}

/// The display label for a share percentage, with 2 decimals. A value that
/// rounds to zero at that precision shows as `< 0.00%` instead, so tiny
/// shares never look exactly empty.
fn percent_label(percent: f64) -> String {
    let rounded = (percent * 100.0).round() / 100.0;
    if rounded > 0.0 {
        format!("{rounded:.2}%")
    } else {
        "< 0.00%".to_string()
    }
}

/// The display label for a file extension, for example `.MOV`. The reserved
/// `no-extension` group renders as `(no extension)`.
fn extension_label(extension: &str) -> String {
    if extension == "no-extension" {
        return "(no extension)".to_string();
    }
    format!(".{}", extension.to_uppercase())
}
