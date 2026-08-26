//! File type allocation: group files by extension and compute each group's
//! share of the total silo size.
//!
//! This module turns the collected file entries into a per-extension
//! breakdown. Each group reports how many files it contains, their combined
//! byte size, and what percentage of the total silo size that group holds.
//! Groups are ordered by total bytes descending.

use std::collections::HashMap;
use std::path::Path;

use serde::Serialize;

/// The reserved extension name for files that have no extension.
const NO_EXTENSION: &str = "no-extension";

/// The extension of `path` without the dot, or the reserved `no-extension`
/// name when the path has no extension.
pub(crate) fn extension_of(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .filter(|e| !e.is_empty())
        .map(|e| e.to_string())
        .unwrap_or_else(|| NO_EXTENSION.to_string())
}

/// Statistics for one file type (grouped by file extension).
#[derive(Debug, Clone, Serialize)]
pub struct FileTypeStat {
    /// The extension without the dot, for example `pdf`. Files without an
    /// extension are grouped under the reserved name `no-extension`.
    pub extension: String,
    /// How many files have this extension.
    pub count: u64,
    /// The total size in bytes of all files with this extension.
    pub total_bytes: u64,
    /// The share of the total silo size in bytes, as a percentage rounded
    /// to 2 decimal places. For example `42.57` means 42.57%.
    pub percent_of_total_bytes: f64,
}

/// Group files by extension and compute each group's share of the silo size.
///
/// Takes one `(extension, size_bytes)` pair per file, so callers that already
/// know the extension do not compute it twice. Groups are ordered by total
/// bytes descending. Ties keep alphabetical extension order.
pub(crate) fn compute_file_types(
    files: impl IntoIterator<Item = (String, u64)>,
    total_size_bytes: u64,
) -> Vec<FileTypeStat> {
    let mut groups: HashMap<String, (u64, u64)> = HashMap::new();

    for (extension, size_bytes) in files {
        let entry = groups.entry(extension).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += size_bytes;
    }

    let mut stats: Vec<FileTypeStat> = groups
        .into_iter()
        .map(|(extension, (count, total_bytes))| FileTypeStat {
            extension,
            count,
            total_bytes,
            percent_of_total_bytes: percentage(total_bytes, total_size_bytes),
        })
        .collect();

    // Order: highest total bytes first. Ties fall back to extension name
    // order, alphabetically.
    stats.sort_by(|a, b| {
        b.total_bytes
            .cmp(&a.total_bytes)
            .then_with(|| a.extension.cmp(&b.extension))
    });

    stats
}

/// The percentage of `part` within `whole`, rounded to 2 decimals.
///
/// When `whole` is zero (an empty silo), this returns 0.0.
fn percentage(part: u64, whole: u64) -> f64 {
    if whole == 0 {
        return 0.0;
    }
    let percent = (part as f64 * 100.0) / whole as f64;
    (percent * 100.0).round() / 100.0
}
