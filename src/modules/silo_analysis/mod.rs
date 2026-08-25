//! SiloAnalysis: filesystem analysis engine for Silo.
//!
//! This module analyzes a silo (a folder given as a path) and reports
//! statistics about every file and directory inside it, recursively.
//! It supports later reporting features such as:
//! - total size of the silo
//! - largest and smallest file
//! - oldest and newest file
//! - average file size
//! - file types by extension and their share of the total silo size
//!
//! Design: the module is split into internal pieces:
//! - [`walk`]: recursive filesystem scan + stats computation
//! - [`file_type_allocation`]: per-extension size breakdown of the silo
//! - [`error`]: typed errors
//!
//! The path to analyze will later come from `silo_path_data` in the Silo
//! SQLite database (under `~/.local`). That database does not exist yet, so
//! this foundation takes the silo path directly as an argument.

pub mod file_type_allocation;

mod error;
mod walk;

use std::path::Path;
use std::path::PathBuf;

pub use error::AnalysisError;
pub use file_type_allocation::FileTypeStat;

/// A single file found inside the silo.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileEntry {
    /// The file name, for example `report.pdf`.
    pub name: String,
    /// The path of the file relative to the silo root, for example `docs/report.pdf`.
    pub relative_path: PathBuf,
    /// The size of the file in bytes.
    pub size_bytes: u64,
    /// The last-modified timestamp of the file, in Unix epoch seconds.
    pub modified: Option<i64>,
}

/// A single directory found inside the silo.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DirEntry {
    /// The directory name, for example `docs`.
    pub name: String,
    /// The path of the directory relative to the silo root, for example `docs/sub`.
    pub relative_path: PathBuf,
}

/// A reference to a file used by the largest/smallest/oldest/newest file
/// statistics.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct FileRef {
    /// The file name.
    pub name: String,
    /// The path of the file relative to the silo root.
    pub relative_path: PathBuf,
    /// The size of the file in bytes.
    pub size_bytes: u64,
    /// The last-modified timestamp of the file, in Unix epoch seconds.
    pub modified: Option<i64>,
}

/// Summary statistics for the analyzed silo.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct Stats {
    /// Total number of files inside the silo.
    pub total_files: u64,
    /// Total number of directories inside the silo (excluding the root).
    pub total_dirs: u64,
    /// Total size of all files in bytes.
    pub total_size_bytes: u64,
    /// Number of files with a size of zero bytes.
    pub zero_byte_files: u64,
    /// The largest file, or `None` if the silo has no files.
    pub largest_file: Option<FileRef>,
    /// The smallest file, or `None` if the silo has no files.
    pub smallest_file: Option<FileRef>,
    /// The average file size in bytes (rounded down), or `None` if the silo
    /// has no files.
    pub average_file_size_bytes: Option<u64>,
    /// The oldest file by last-modified time, or `None` if the silo has no
    /// files with a readable timestamp.
    pub oldest_file: Option<FileRef>,
    /// The newest file by last-modified time, or `None` if the silo has no
    /// files with a readable timestamp.
    pub newest_file: Option<FileRef>,
    /// File types by extension, ordered by total bytes descending.
    pub file_types: Vec<FileTypeStat>,
}

/// The full result of analyzing a silo.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AnalysisReport {
    /// The silo root path that was analyzed.
    pub root: PathBuf,
    /// Summary statistics for the silo.
    pub stats: Stats,
    /// Every file found, in scan order.
    pub files: Vec<FileEntry>,
    /// Every directory found, in scan order.
    pub dirs: Vec<DirEntry>,
    /// Per-entry errors encountered during the walk. A failed entry does not
    /// abort the whole scan; it is recorded here instead.
    pub scan_errors: Vec<String>,
}

/// Analyze a silo: walk every file and directory under `path` and compute
/// statistics about it.
///
/// The path must exist and be a directory. `excludes` holds the exclude
/// patterns; excluded directories and files are skipped, matching the silo
/// size computation. Returns [`AnalysisError`] for validation failures and
/// root-level walk failures. Individual unreadable sub-entries are recorded
/// in [`AnalysisReport::scan_errors`] and do not abort the scan.
pub fn analyze(path: &Path, excludes: &[String]) -> Result<AnalysisReport, AnalysisError> {
    walk::analyze(path, excludes)
}

/// A single file behind the allocation chart, with the folder that holds it.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct AllocationFile {
    /// The extension without the dot, or the reserved `no-extension` name.
    pub extension: String,
    /// The silo root folder that contains this file.
    pub root: PathBuf,
    /// The path relative to `root`.
    pub relative_path: PathBuf,
    /// The size of the file in bytes.
    pub size_bytes: u64,
}

/// The file-type allocation of the silo: the per-extension summary plus the
/// files behind it.
#[derive(Debug, Clone, Default)]
pub struct Allocation {
    /// The per-extension summary, ordered by total bytes descending.
    pub stats: Vec<FileTypeStat>,
    /// Every non-excluded file with its extension, root, and relative path.
    pub files: Vec<AllocationFile>,
    /// The merged silo-wide statistics, shown in the STATS table.
    pub summary: Stats,
}

/// Analyze every silo folder and build the full file-type allocation.
///
/// Runs [`analyze`] on every path with the same exclude patterns. A folder
/// that cannot be analyzed is skipped, so one bad folder does not empty the
/// result. The stats are ordered by total bytes descending; `files` carries
/// every file with its full location for the breakdown view.
pub fn silo_allocation(paths: &[PathBuf], excludes: &[String]) -> Allocation {
    let mut entries: Vec<FileEntry> = Vec::new();
    let mut files: Vec<AllocationFile> = Vec::new();
    let mut summaries: Vec<Stats> = Vec::new();
    for root in paths {
        if let Ok(report) = analyze(root, excludes) {
            for entry in report.files {
                files.push(AllocationFile {
                    extension: file_type_allocation::extension_of(&entry.relative_path),
                    root: root.clone(),
                    relative_path: entry.relative_path.clone(),
                    size_bytes: entry.size_bytes,
                });
                entries.push(entry);
            }
            summaries.push(report.stats);
        }
    }
    let total: u64 = entries.iter().map(|entry| entry.size_bytes).sum();
    let stats = file_type_allocation::compute_file_types(&entries, total);

    let mut summary = merge_stats(summaries);
    summary.file_types = stats.clone();

    Allocation { stats, files, summary }
}

/// Merges the per-folder statistics into one silo-wide summary.
///
/// Counts and sizes are summed; the largest, smallest, oldest, and newest
/// files are the extremes across every folder. The average is recomputed from
/// the merged totals.
fn merge_stats(summaries: Vec<Stats>) -> Stats {
    let total_files = summaries.iter().map(|s| s.total_files).sum();
    let total_dirs = summaries.iter().map(|s| s.total_dirs).sum();
    let total_size_bytes = summaries.iter().map(|s| s.total_size_bytes).sum();
    let zero_byte_files = summaries.iter().map(|s| s.zero_byte_files).sum();

    let largest_file = summaries
        .iter()
        .filter_map(|s| s.largest_file.clone())
        .max_by(|a, b| a.size_bytes.cmp(&b.size_bytes));
    let smallest_file = summaries
        .iter()
        .filter_map(|s| s.smallest_file.clone())
        .min_by(|a, b| a.size_bytes.cmp(&b.size_bytes));
    let oldest_file = summaries
        .iter()
        .filter_map(|s| s.oldest_file.clone())
        .min_by(|a, b| a.modified.cmp(&b.modified));
    let newest_file = summaries
        .iter()
        .filter_map(|s| s.newest_file.clone())
        .max_by(|a, b| a.modified.cmp(&b.modified));

    let average_file_size_bytes = if total_files == 0 {
        None
    } else {
        Some(total_size_bytes / total_files)
    };

    Stats {
        total_files,
        total_dirs,
        total_size_bytes,
        zero_byte_files,
        largest_file,
        smallest_file,
        average_file_size_bytes,
        oldest_file,
        newest_file,
        // The merged file types are filled in by the caller.
        file_types: Vec::new(),
    }
}
