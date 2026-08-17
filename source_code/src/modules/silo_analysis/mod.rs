//! SiloAnalysis: filesystem analysis engine for Silo.
//!
//! This module analyzes a silo (a folder given as a path) and reports
//! statistics about every file and directory inside it, recursively.
//! It supports later reporting features such as:
//! - total size of the silo
//! - largest and smallest file
//! - average file size
//! - file types by extension and their share of the total silo size
//!
//! Design: the module is split into two internal pieces:
//! - [`walk`]: recursive filesystem scan + stats computation
//! - [`error`]: typed errors
//!
//! The path to analyze will later come from `silo_path_data` in the Silo
//! SQLite database (under `~/.local`). That database does not exist yet, so
//! this foundation takes the silo path directly as an argument.

mod error;
mod walk;

use std::path::Path;
use std::path::PathBuf;

pub use error::AnalysisError;

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

/// A reference to a file used by the largest/smallest file statistics.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct FileRef {
    /// The file name.
    pub name: String,
    /// The path of the file relative to the silo root.
    pub relative_path: PathBuf,
    /// The size of the file in bytes.
    pub size_bytes: u64,
}

/// Statistics for one file type (grouped by file extension).
#[derive(Debug, Clone, serde::Serialize)]
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

/// Summary statistics for the analyzed silo.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Stats {
    /// Total number of files inside the silo.
    pub total_files: u64,
    /// Total number of directories inside the silo (excluding the root).
    pub total_dirs: u64,
    /// Total size of all files in bytes.
    pub total_size_bytes: u64,
    /// The largest file, or `None` if the silo has no files.
    pub largest_file: Option<FileRef>,
    /// The smallest file, or `None` if the silo has no files.
    pub smallest_file: Option<FileRef>,
    /// The average file size in bytes (rounded down), or `None` if the silo
    /// has no files.
    pub average_file_size_bytes: Option<u64>,
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
/// The path must exist and be a directory. Returns [`AnalysisError`] for
/// validation failures and root-level walk failures. Individual unreadable
/// sub-entries are recorded in [`AnalysisReport::scan_errors`] and do not
/// abort the scan.
pub fn analyze(path: &Path) -> Result<AnalysisReport, AnalysisError> {
    walk::analyze(path)
}