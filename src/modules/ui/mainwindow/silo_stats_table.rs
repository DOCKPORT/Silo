//! SiloStatsTable: the silo statistics table in the STATS box.
//!
//! A two-column table: a grey label on the left and a teal value on the
//! right, one row per summary statistic. Sizes use the shared IEC labels;
//! the largest, smallest, oldest, and newest file rows show the file name
//! plus its size or modified date. Missing statistics show an em dash.

use iced::widget::{Column, Row, Space, container, text};
use iced::{Element, Length, Padding};

use crate::modules::silo_analysis::Stats;
use crate::modules::silo_size;
use crate::modules::ui::scaling::sp;
use crate::modules::ui::theme::{GREY, TEAL, ZEBRA};

use super::Message;

/// The font size of the table labels and values, in reference pixels.
/// Matches the ALLOCATION chart row text size.
const TEXT_SIZE: f32 = 18.0;

/// The height of one table row, in reference pixels. Matches the ALLOCATION
/// chart virtual row height so the zebra stripes look identical.
const ROW_HEIGHT: f32 = 28.0;

/// The vertical gap between two table rows, in reference pixels.
const ROW_SPACING: f32 = 8.0;

/// The padding around the table inside the STATS box, in reference pixels.
const TABLE_PAD: f32 = 14.0;

/// The label of a missing statistic.
const MISSING: &str = "—";

/// Builds the statistics table.
///
/// Renders one row per summary statistic. `summary` is the merged silo-wide
/// statistics; an empty silo shows the counts and em dashes for the rest.
pub fn view(summary: &Stats) -> Element<'static, Message> {
    let entries = [
        ("TOTAL FILES", summary.total_files.to_string()),
        ("TOTAL FOLDERS", summary.total_dirs.to_string()),
        ("TOTAL SIZE", silo_size::human_size(summary.total_size_bytes)),
        ("ZERO-BYTE FILES", summary.zero_byte_files.to_string()),
        (
            "AVERAGE FILE SIZE",
            summary
                .average_file_size_bytes
                .map(silo_size::human_size)
                .unwrap_or_else(|| MISSING.to_string()),
        ),
        (
            "LARGEST FILE",
            summary
                .largest_file
                .as_ref()
                .map(|file| format!("{} · {}", file.name, silo_size::human_size(file.size_bytes)))
                .unwrap_or_else(|| MISSING.to_string()),
        ),
        (
            "SMALLEST FILE",
            summary
                .smallest_file
                .as_ref()
                .map(|file| format!("{} · {}", file.name, silo_size::human_size(file.size_bytes)))
                .unwrap_or_else(|| MISSING.to_string()),
        ),
        (
            "OLDEST FILE",
            summary
                .oldest_file
                .as_ref()
                .map(|file| format!("{} · {}", file.name, format_date(file.modified)))
                .unwrap_or_else(|| MISSING.to_string()),
        ),
        (
            "NEWEST FILE",
            summary
                .newest_file
                .as_ref()
                .map(|file| format!("{} · {}", file.name, format_date(file.modified)))
                .unwrap_or_else(|| MISSING.to_string()),
        ),
    ];

    let mut rows = Column::new().spacing(sp(ROW_SPACING));
    for (index, (label, value)) in entries.into_iter().enumerate() {
        rows = rows.push(label_value(label, value, index));
    }

    container(rows)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: sp(TABLE_PAD),
            left: sp(TABLE_PAD),
            right: sp(TABLE_PAD),
            bottom: sp(TABLE_PAD),
        })
        .into()
}

/// Builds one table row: the label on the left and the value on the right.
/// Zebra striping colors every other row with the dark grey stripe, matching
/// the ALLOCATION chart.
fn label_value(label: &'static str, value: String, index: usize) -> Element<'static, Message> {
    let row: Element<'static, Message> = Row::new()
        .width(Length::Fill)
        .height(Length::Fixed(sp(ROW_HEIGHT)))
        .align_y(iced::alignment::Vertical::Center)
        .push(text(label).size(sp(TEXT_SIZE)).color(GREY))
        .push(Space::new().width(Length::Fill))
        .push(text(value).size(sp(TEXT_SIZE)).color(TEAL))
        .into();

    container(row)
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

/// Formats a Unix timestamp as a date, for example `2024-08-24 14:32`.
/// Returns an em dash when the timestamp is missing.
fn format_date(modified: Option<i64>) -> String {
    match modified {
        Some(secs) => civil_date(secs),
        None => MISSING.to_string(),
    }
}

/// Converts Unix seconds into a `YYYY-MM-DD HH:MM` string.
///
/// Uses Howard Hinnant's days-to-civil algorithm, so no date library is
/// needed. Negative timestamps (before 1970) are handled with floor division.
fn civil_date(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let hour = rem / 3600;
    let minute = (rem % 3600) / 60;

    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };

    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}")
}
