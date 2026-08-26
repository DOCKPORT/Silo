//! SiloStatsTable: the silo statistics table in the STATS box.
//!
//! A two-column table: a grey label on the left and a teal value on the
//! right, one row per summary statistic. Sizes use the shared IEC labels;
//! the largest, smallest, oldest, and newest file rows show the file name
//! plus its size or modified date. Missing statistics show an em dash.
//!
//! Six rows are expandable, like the ALLOCATION chart: pressing them toggles
//! an inline breakdown of the underlying entries. EMPTY FOLDERS lists the
//! empty folder paths, ZERO-BYTE FILES lists the files, and the single-file
//! rows show the full path plus size or date.

use std::path::PathBuf;

use iced::mouse;
use iced::widget::{Column, MouseArea, Row, Space, Stack, container, lazy, text};
use iced::{Color, Element, Length, Padding};

use crate::modules::silo_analysis::{FileRef, Stats};
use crate::modules::silo_size;
use crate::modules::ui::crosshatch;
use crate::modules::ui::scaling::sp;
use crate::modules::ui::scrollbar;
use crate::modules::ui::theme::{BACK, DETAIL, GREY, TEAL, ZEBRA};

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

/// The font size of the expanded breakdown lines, in reference pixels.
const DETAIL_SIZE: f32 = 15.0;

/// The indentation of the expanded breakdown lines, in reference pixels.
const DETAIL_INDENT: f32 = 24.0;

/// The padding around the expanded breakdown block, in reference pixels.
const BLOCK_PAD: f32 = 8.0;

/// The alpha of the subtle background behind an expanded breakdown.
const BLOCK_BG_ALPHA: f32 = 0.06;

/// The horizontal padding between the STATS box edges and the crosshatch
/// pattern, in reference pixels. Matches the table padding so the pattern
/// spans the same width as the table rows.
const CROSSHATCH_SIDE_PAD: f32 = 14.0;

/// The vertical padding above the crosshatch pattern, in reference pixels.
const CROSSHATCH_TOP_PAD: f32 = 40.0;

/// The vertical padding below the crosshatch pattern, in reference pixels.
const CROSSHATCH_BOTTOM_PAD: f32 = 20.0;

/// The label of a missing statistic.
const MISSING: &str = "—";

/// The expandable rows of the STATS table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum StatsRow {
    /// The EMPTY FOLDERS row.
    EmptyFolders,
    /// The ZERO-BYTE FILES row.
    ZeroByteFiles,
    /// The LARGEST FILE row.
    LargestFile,
    /// The SMALLEST FILE row.
    SmallestFile,
    /// The OLDEST FILE row.
    OldestFile,
    /// The NEWEST FILE row.
    NewestFile,
}

/// Builds the statistics table.
///
/// Renders one row per summary statistic. `summary` is the merged silo-wide
/// statistics; an empty silo shows the counts and em dashes for the rest.
/// The row for `selected` is expanded into its breakdown.
pub fn view<'a>(
    summary: &'a Stats,
    selected: Option<StatsRow>,
    generation: u64,
) -> Element<'a, Message> {
    // Rebuild only when the summary data or the expanded row changes,
    // matching the ALLOCATION chart's lazy caching. The summary changes only
    // through `Message::AllocationComputed`, which bumps the generation.
    lazy((generation, selected), move |_| {
        build_table(summary, selected)
    })
    .into()
}

/// Builds the full table for the current summary.
fn build_table(summary: &Stats, selected: Option<StatsRow>) -> Element<'static, Message> {
    let mut rows = Column::new().spacing(sp(ROW_SPACING));

    rows = rows.push(label_value(
        "TOTAL FILES",
        summary.total_files.to_string(),
        0,
    ));
    rows = rows.push(label_value(
        "TOTAL FOLDERS",
        summary.total_dirs.to_string(),
        1,
    ));
    rows = rows.push(expandable_row(
        "EMPTY FOLDERS",
        summary.empty_folder_paths.len().to_string(),
        2,
        StatsRow::EmptyFolders,
        selected,
        summary,
    ));
    rows = rows.push(label_value(
        "TOTAL SIZE",
        silo_size::human_size(summary.total_size_bytes),
        3,
    ));
    rows = rows.push(expandable_row(
        "ZERO-BYTE FILES",
        summary.zero_byte_files.len().to_string(),
        4,
        StatsRow::ZeroByteFiles,
        selected,
        summary,
    ));
    rows = rows.push(label_value(
        "AVERAGE FILE SIZE",
        summary
            .average_file_size_bytes
            .map(silo_size::human_size)
            .unwrap_or_else(|| MISSING.to_string()),
        5,
    ));
    rows = rows.push(expandable_row(
        "LARGEST FILE",
        file_value(summary.largest_file.as_ref(), |file| {
            silo_size::human_size(file.size_bytes)
        }),
        6,
        StatsRow::LargestFile,
        selected,
        summary,
    ));
    rows = rows.push(expandable_row(
        "SMALLEST FILE",
        file_value(summary.smallest_file.as_ref(), |file| {
            silo_size::human_size(file.size_bytes)
        }),
        7,
        StatsRow::SmallestFile,
        selected,
        summary,
    ));
    rows = rows.push(expandable_row(
        "OLDEST FILE",
        file_value(summary.oldest_file.as_ref(), |file| {
            format_date(file.modified)
        }),
        8,
        StatsRow::OldestFile,
        selected,
        summary,
    ));
    rows = rows.push(expandable_row(
        "NEWEST FILE",
        file_value(summary.newest_file.as_ref(), |file| {
            format_date(file.modified)
        }),
        9,
        StatsRow::NewestFile,
        selected,
        summary,
    ));

    // The table sits on the box background, so the crosshatch pattern only
    // shows in the empty bottom below the rows. The table scrolls when an
    // expansion is longer than the STATS box.
    let table = container(rows)
        .width(Length::Fill)
        .padding(Padding {
            top: sp(TABLE_PAD),
            left: sp(TABLE_PAD),
            right: sp(TABLE_PAD),
            bottom: sp(TABLE_PAD),
        })
        .style(|_| container::Style {
            background: Some(BACK.into()),
            ..container::Style::default()
        });

    // The crosshatch fills the empty bottom of the STATS box, behind the
    // scrollable, padded on every side.
    let crosshatch_layer = container(crosshatch::overlay())
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: sp(CROSSHATCH_TOP_PAD),
            left: sp(CROSSHATCH_SIDE_PAD),
            right: sp(CROSSHATCH_SIDE_PAD),
            bottom: sp(CROSSHATCH_BOTTOM_PAD),
        });

    Stack::new()
        .push(crosshatch_layer)
        .push(scrollbar::vertical(table))
        .into()
}

/// Builds one static table row: the label on the left and the value on the
/// right. Zebra striping colors every other row with the dark grey stripe,
/// matching the ALLOCATION chart.
fn label_value(label: &'static str, value: String, index: usize) -> Element<'static, Message> {
    let row: Element<'static, Message> = Row::new()
        .width(Length::Fill)
        .height(Length::Fixed(sp(ROW_HEIGHT)))
        .align_y(iced::alignment::Vertical::Center)
        .push(text(label).size(sp(TEXT_SIZE)).color(GREY))
        .push(Space::new().width(Length::Fill))
        .push(text(value).size(sp(TEXT_SIZE)).color(TEAL))
        .into();

    zebra(row, index)
}

/// Builds one expandable table row: the label with an expand indicator on the
/// left and the value on the right. Pressing the row toggles its breakdown;
/// the open row shows a teal indicator with the breakdown below it.
fn expandable_row(
    label: &'static str,
    value: String,
    index: usize,
    kind: StatsRow,
    selected: Option<StatsRow>,
    summary: &Stats,
) -> Element<'static, Message> {
    let is_open = selected == Some(kind);

    let content: Element<'static, Message> = Row::new()
        .width(Length::Fill)
        .height(Length::Fixed(sp(ROW_HEIGHT)))
        .align_y(iced::alignment::Vertical::Center)
        .push(
            text(format!("{} {}", if is_open { "▾" } else { "▸" }, label))
                .size(sp(TEXT_SIZE))
                .color(if is_open { TEAL } else { GREY }),
        )
        .push(Space::new().width(Length::Fill))
        .push(text(value).size(sp(TEXT_SIZE)).color(TEAL))
        .into();

    let header = MouseArea::new(zebra(content, index))
        .on_press(Message::StatsRowPressed(kind))
        .interaction(mouse::Interaction::Pointer);

    if is_open {
        Column::new()
            .spacing(sp(ROW_SPACING))
            .push(header)
            .push(expansion(summary, kind))
            .into()
    } else {
        header.into()
    }
}

/// The expanded breakdown for one stat row.
fn expansion(summary: &Stats, kind: StatsRow) -> Element<'static, Message> {
    match kind {
        StatsRow::EmptyFolders => block(path_rows(&summary.empty_folder_paths)),
        StatsRow::ZeroByteFiles => block(zero_byte_rows(&summary.zero_byte_files)),
        StatsRow::LargestFile => single_file_block(summary.largest_file.as_ref(), |file| {
            silo_size::human_size(file.size_bytes)
        }),
        StatsRow::SmallestFile => single_file_block(summary.smallest_file.as_ref(), |file| {
            silo_size::human_size(file.size_bytes)
        }),
        StatsRow::OldestFile => single_file_block(summary.oldest_file.as_ref(), |file| {
            format_date(file.modified)
        }),
        StatsRow::NewestFile => single_file_block(summary.newest_file.as_ref(), |file| {
            format_date(file.modified)
        }),
    }
}

/// Builds one detail line per empty folder path.
fn path_rows(paths: &[PathBuf]) -> Element<'static, Message> {
    let mut column = Column::new().spacing(sp(ROW_SPACING));
    for path in paths {
        column = column.push(detail_row(path.to_string_lossy().into_owned(), None));
    }
    column.into()
}

/// Builds one detail line per zero-byte file, with its full path.
fn zero_byte_rows(files: &[FileRef]) -> Element<'static, Message> {
    let mut column = Column::new().spacing(sp(ROW_SPACING));
    for file in files {
        column = column.push(detail_row(
            full_path(file),
            Some(silo_size::human_size(file.size_bytes)),
        ));
    }
    column.into()
}

/// Builds the one-line expansion for a single-file stat: the full path with
/// its size or date on the right.
fn single_file_block(
    file: Option<&FileRef>,
    extra: impl Fn(&FileRef) -> String,
) -> Element<'static, Message> {
    match file {
        Some(file) => block(
            Column::new()
                .spacing(sp(ROW_SPACING))
                .push(detail_row(full_path(file), Some(extra(file))))
                .into(),
        ),
        None => block(Column::new().spacing(sp(ROW_SPACING)).into()),
    }
}

/// Wraps an expansion column in the subtle tinted breakdown block.
fn block(content: Element<'static, Message>) -> Element<'static, Message> {
    container(content)
        .width(Length::Fill)
        .padding(Padding {
            top: sp(BLOCK_PAD),
            left: sp(BLOCK_PAD),
            right: sp(BLOCK_PAD),
            bottom: sp(BLOCK_PAD),
        })
        .style(|_| container::Style {
            background: Some(
                Color {
                    a: BLOCK_BG_ALPHA,
                    ..DETAIL
                }
                .into(),
            ),
            ..container::Style::default()
        })
        .into()
}

/// Builds one breakdown line: the text on the left, indented, with an
/// optional value on the right.
fn detail_row(left: String, right: Option<String>) -> Element<'static, Message> {
    let mut row = Row::new()
        .width(Length::Fill)
        .align_y(iced::alignment::Vertical::Center);
    row = row.push(Space::new().width(Length::Fixed(sp(DETAIL_INDENT))));
    row = row.push(text(left).size(sp(DETAIL_SIZE)).color(GREY));
    if let Some(value) = right {
        row = row
            .push(Space::new().width(Length::Fill))
            .push(text(value).size(sp(DETAIL_SIZE)).color(TEAL));
    }

    let row: Element<'static, Message> = row.into();
    row
}

/// Applies the zebra stripe background to a row: odd rows get the dark grey
/// fill, even rows keep the box background.
fn zebra(content: Element<'static, Message>, index: usize) -> Element<'static, Message> {
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

/// The table value for a single-file row: `name · extra`.
fn file_value(file: Option<&FileRef>, extra: impl Fn(&FileRef) -> String) -> String {
    match file {
        Some(file) => format!("{} · {}", file.name, extra(file)),
        None => MISSING.to_string(),
    }
}

/// The full path of a file: its root joined with its relative path.
fn full_path(file: &FileRef) -> String {
    file.root
        .join(&file.relative_path)
        .to_string_lossy()
        .into_owned()
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
