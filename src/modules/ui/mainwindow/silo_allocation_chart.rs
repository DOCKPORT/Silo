//! SiloAllocationChart: the file-type allocation chart for the ALLOCATION box.
//!
//! A vertical list of rows, one per file type. Each row shows the file
//! extension and its share of the total silo size as a percentage. The input
//! is ordered by total bytes descending, so the biggest share sits on top.
//!
//! Pressing a row expands it inline into the file breakdown for that
//! extension. The breakdown data is prepared off the UI thread by [`prepare`]
//! and cached in memory, so expanding an already-opened extension is instant.
//!
//! The whole chart is a virtualized list: every line (extension rows plus the
//! breakdown's folder headers and file rows) is a fixed-height virtual row,
//! and only the rows visible in the scrollable viewport are built. Spacers
//! above and below keep the content height exact, so the scrollbar range is
//! correct while the per-frame layout stays small even for huge extensions.
//! Many file types scroll through the shared app scrollbar.

use std::collections::BTreeMap;

use iced::mouse;
use iced::widget::{Column, Id, MouseArea, Row, Space, container, lazy, text};
use iced::{Color, Element, Length, Padding};

use crate::modules::silo_analysis::{AllocationFile, FileTypeStat};
use crate::modules::silo_size;
use crate::modules::ui::scaling::sp;
use crate::modules::ui::scrollbar;
use crate::modules::ui::theme::{DETAIL, GREY, TEAL, ZEBRA};

use super::Message;

/// The id of the chart scrollable, used to reset it to the top when the open
/// breakdown changes.
pub const SCROLL_ID: &str = "allocation-chart";

/// The font size of the chart row text, in reference pixels.
const ROW_TEXT_SIZE: f32 = 18.0;

/// The uniform height of one virtual row, in reference pixels.
const VIRTUAL_ROW_HEIGHT: f32 = 28.0;

/// The viewport height used before the first scroll event reports the real
/// value, in reference pixels.
const DEFAULT_VIEWPORT_HEIGHT: f32 = 700.0;

/// The horizontal padding between the chart area and the ALLOCATION box
/// edges, in reference pixels.
const CHART_SIDE_PAD: f32 = 50.0;

/// The font size of the folder headers inside an expanded breakdown, in
/// reference pixels.
const BREAKDOWN_DIR_SIZE: f32 = 15.0;

/// The font size of the file rows inside an expanded breakdown, in reference
/// pixels.
const BREAKDOWN_FILE_SIZE: f32 = 15.0;

/// The indentation of the file rows under their folder header, in reference
/// pixels.
const BREAKDOWN_INDENT: f32 = 24.0;

/// The alpha of the subtle background behind the expanded breakdown rows.
const BREAKDOWN_BG_ALPHA: f32 = 0.06;

/// One line of the chart, addressed by index in the flattened row list.
enum VirtualRow {
    /// A file-type row for the extension at this position in `allocation`.
    Extension(usize),
    /// The preparing indicator under a pending extension row.
    Pending,
    /// A breakdown folder header at this folder index.
    FolderHeader(usize),
    /// A breakdown file row at these folder and file indices.
    File(usize, usize),
}

/// One file of a prepared breakdown: its name and its size label.
#[derive(Debug, Clone)]
pub struct PreparedFile {
    /// The file name.
    pub name: String,
    /// The human-readable size label, for example `89.0 MiB`.
    pub size: String,
}

/// One folder of a prepared breakdown: the folder label and its files.
#[derive(Debug, Clone)]
pub struct PreparedFolder {
    /// The folder label, for example `DOCK-HQ/DEV/project/.git/objects/aa`.
    pub dir: String,
    /// The files in this folder, ordered by path.
    pub files: Vec<PreparedFile>,
}

/// A prepared file breakdown for one extension, cached in memory.
#[derive(Debug, Clone)]
pub struct PreparedBreakdown {
    /// The extension this breakdown covers.
    pub extension: String,
    /// The folders, ordered by label.
    pub folders: Vec<PreparedFolder>,
}

/// Prepares the breakdown data for one extension from the silo files.
///
/// Groups the files by folder and formats one entry per file. This runs off
/// the UI thread; the result is cached so re-expanding the extension is
/// instant.
pub fn prepare(files: &[AllocationFile], extension: &str) -> PreparedBreakdown {
    let mut dirs: BTreeMap<String, Vec<&AllocationFile>> = BTreeMap::new();
    for file in files.iter().filter(|file| file.extension == extension) {
        dirs.entry(folder_label(file)).or_default().push(file);
    }

    let folders = dirs
        .into_iter()
        .map(|(dir, mut dir_files)| {
            dir_files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

            let files = dir_files
                .iter()
                .map(|file| PreparedFile {
                    name: file_name(file),
                    size: silo_size::human_size(file.size_bytes),
                })
                .collect();

            PreparedFolder { dir, files }
        })
        .collect();

    PreparedBreakdown {
        extension: extension.to_string(),
        folders,
    }
}

/// Builds the allocation chart.
///
/// Renders the visible window of the virtualized list: the extension rows,
/// plus the expanded breakdown lines. `expanded` is the prepared breakdown to
/// show, if any; `pending` is the extension being prepared. `generation`
/// bumps whenever the chart data changes. `scroll_offset` and
/// `viewport_height` come from the scrollable and select the visible window.
pub fn view<'a>(
    allocation: &'a [FileTypeStat],
    expanded: Option<&'a PreparedBreakdown>,
    pending: Option<&'a str>,
    generation: u64,
    scroll_offset: f32,
    viewport_height: f32,
) -> Element<'a, Message> {
    let open = expanded
        .map(|breakdown| breakdown.extension.as_str())
        .or(pending);
    let dependency = (
        generation,
        open.map(str::to_string),
        scroll_offset as i32,
        viewport_height as i32,
    );

    lazy(dependency, move |_| {
        build_chart(
            allocation,
            expanded,
            pending,
            open,
            scroll_offset,
            viewport_height,
        )
    })
    .into()
}

/// Builds the full chart for the current scroll window.
fn build_chart(
    allocation: &[FileTypeStat],
    expanded: Option<&PreparedBreakdown>,
    pending: Option<&str>,
    open: Option<&str>,
    scroll_offset: f32,
    viewport_height: f32,
) -> Element<'static, Message> {
    let row_h = sp(VIRTUAL_ROW_HEIGHT);

    // Flatten every line into a virtual row index.
    let mut rows: Vec<VirtualRow> = Vec::new();
    for (index, stat) in allocation.iter().enumerate() {
        rows.push(VirtualRow::Extension(index));
        if let Some(breakdown) = expanded.filter(|b| b.extension == stat.extension) {
            for (folder_index, folder) in breakdown.folders.iter().enumerate() {
                rows.push(VirtualRow::FolderHeader(folder_index));
                for (file_index, _file) in folder.files.iter().enumerate() {
                    rows.push(VirtualRow::File(folder_index, file_index));
                }
            }
        } else if pending == Some(stat.extension.as_str()) {
            rows.push(VirtualRow::Pending);
        }
    }

    if rows.is_empty() {
        return container(text("").width(Length::Fill).height(Length::Fill)).into();
    }

    // The visible window, in virtual-row indices. The offset is clamped to
    // the content height, so a stale offset from a previous, taller list can
    // never show a blank window.
    let viewport_h = if viewport_height > 0.0 {
        viewport_height
    } else {
        sp(DEFAULT_VIEWPORT_HEIGHT)
    };
    let max_offset = (rows.len() as f32 * row_h - viewport_h).max(0.0);
    let offset = scroll_offset.clamp(0.0, max_offset);
    let first = (offset / row_h).floor() as usize;
    let last = (first + (viewport_h / row_h).ceil() as usize + 1).min(rows.len());

    let mut content = Column::new().width(Length::Fill);
    content = content.push(
        Space::new()
            .width(Length::Fill)
            .height(Length::Fixed(row_h * first as f32)),
    );

    for row in &rows[first..last] {
        content = content.push(render_row(row, allocation, expanded, open, row_h));
    }

    content = content.push(
        Space::new()
            .width(Length::Fill)
            .height(Length::Fixed(row_h * (rows.len() - last) as f32)),
    );

    // The side padding insets the rows only; the scrollable fills the whole
    // box, so the scrollbar stays flush with the ALLOCATION box edge.
    let chart = container(content).width(Length::Fill).padding(Padding {
        left: sp(CHART_SIDE_PAD),
        right: sp(CHART_SIDE_PAD),
        top: 0.0,
        bottom: 0.0,
    });

    scrollbar::vertical(chart)
        .id(Id::new(SCROLL_ID))
        .on_scroll(|viewport| Message::BreakdownScrolled {
            offset: viewport.absolute_offset().y,
            viewport_height: viewport.bounds().height,
        })
        .into()
}

/// Renders one virtual row as a fixed-height element.
fn render_row(
    row: &VirtualRow,
    allocation: &[FileTypeStat],
    expanded: Option<&PreparedBreakdown>,
    open: Option<&str>,
    row_h: f32,
) -> Element<'static, Message> {
    match row {
        VirtualRow::Extension(index) => extension_row(&allocation[*index], *index, open, row_h),
        VirtualRow::Pending => preparing_row(row_h),
        VirtualRow::FolderHeader(folder_index) => match expanded {
            Some(breakdown) => folder_header(&breakdown.folders[*folder_index], row_h),
            None => empty_row(row_h),
        },
        VirtualRow::File(folder_index, file_index) => match expanded {
            Some(breakdown) => {
                file_row(&breakdown.folders[*folder_index].files[*file_index], row_h)
            }
            None => empty_row(row_h),
        },
    }
}

/// Builds one file-type row: the extension on the left and its share
/// percentage on the right. Zebra striping colors every other row with the
/// dark grey stripe. Pressing the row toggles its breakdown; the open row
/// shows a teal indicator.
fn extension_row(
    stat: &FileTypeStat,
    index: usize,
    open: Option<&str>,
    row_h: f32,
) -> Element<'static, Message> {
    let is_open = open == Some(stat.extension.as_str());

    let content: Element<'static, Message> = Row::new()
        .width(Length::Fill)
        .height(Length::Fixed(row_h))
        .align_y(iced::alignment::Vertical::Center)
        .push(
            text(format!(
                "{} {}",
                if is_open { "▾" } else { "▸" },
                extension_label(&stat.extension)
            ))
            .size(sp(ROW_TEXT_SIZE))
            .color(if is_open { TEAL } else { GREY }),
        )
        .push(Space::new().width(Length::Fill))
        .push(
            text(percent_label(stat.percent_of_total_bytes))
                .size(sp(ROW_TEXT_SIZE))
                .color(TEAL),
        )
        .into();

    let styled = container(content)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: if index % 2 == 1 {
                Some(ZEBRA.into())
            } else {
                None
            },
            ..container::Style::default()
        });

    MouseArea::new(styled)
        .on_press(Message::AllocationRowPressed(stat.extension.clone()))
        .interaction(mouse::Interaction::Pointer)
        .into()
}

/// Builds one breakdown folder header row.
fn folder_header(folder: &PreparedFolder, row_h: f32) -> Element<'static, Message> {
    container(
        text(folder.dir.clone())
            .size(sp(BREAKDOWN_DIR_SIZE))
            .color(DETAIL),
    )
    .width(Length::Fill)
    .height(Length::Fixed(row_h))
    .align_y(iced::alignment::Vertical::Center)
    .style(|_| container::Style {
        background: Some(
            Color {
                a: BREAKDOWN_BG_ALPHA,
                ..DETAIL
            }
            .into(),
        ),
        ..container::Style::default()
    })
    .into()
}

/// Builds one breakdown file row: the name on the left, indented under its
/// folder header, and the size flush to the right edge.
fn file_row(file: &PreparedFile, row_h: f32) -> Element<'static, Message> {
    container(
        Row::new()
            .width(Length::Fill)
            .height(Length::Fixed(row_h))
            .align_y(iced::alignment::Vertical::Center)
            .push(Space::new().width(Length::Fixed(sp(BREAKDOWN_INDENT))))
            .push(
                text(file.name.clone())
                    .size(sp(BREAKDOWN_FILE_SIZE))
                    .color(GREY),
            )
            .push(Space::new().width(Length::Fill))
            .push(
                text(file.size.clone())
                    .size(sp(BREAKDOWN_FILE_SIZE))
                    .color(TEAL),
            ),
    )
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(
            Color {
                a: BREAKDOWN_BG_ALPHA,
                ..DETAIL
            }
            .into(),
        ),
        ..container::Style::default()
    })
    .into()
}

/// Builds the preparing indicator row shown under a pending extension row.
fn preparing_row(row_h: f32) -> Element<'static, Message> {
    container(text("PREPARING…").size(sp(BREAKDOWN_FILE_SIZE)).color(TEAL))
        .width(Length::Fill)
        .height(Length::Fixed(row_h))
        .align_y(iced::alignment::Vertical::Center)
        .padding(Padding {
            left: sp(BREAKDOWN_INDENT),
            right: sp(BREAKDOWN_INDENT),
            top: 0.0,
            bottom: 0.0,
        })
        .into()
}

/// An empty fixed-height row, used as a safe fallback for a missing
/// breakdown.
fn empty_row(row_h: f32) -> Element<'static, Message> {
    container(text(""))
        .width(Length::Fill)
        .height(Length::Fixed(row_h))
        .into()
}

/// The display label for the folder that holds a file: the root's name plus
/// the parent directory relative to that root.
fn folder_label(file: &AllocationFile) -> String {
    let root_name = file
        .root
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let parent = file
        .relative_path
        .parent()
        .map(|parent| parent.to_string_lossy().into_owned())
        .unwrap_or_default();
    if parent.is_empty() {
        root_name
    } else {
        format!("{root_name}/{parent}")
    }
}

/// The file name of an allocation file.
fn file_name(file: &AllocationFile) -> String {
    file.relative_path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
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
