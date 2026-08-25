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

/// Merge the file-type allocation of several silo folders into one list.
///
/// Runs [`analyze`] on every path with the same exclude patterns and
/// collects the file entries. A folder that cannot be analyzed is skipped, so
/// one bad folder does not empty the whole chart. The merged result is
/// ordered by total bytes descending, matching the per-folder allocation.
pub fn merged_file_types(paths: &[PathBuf], excludes: &[String]) -> Vec<FileTypeStat> {
    let mut files = Vec::new();
    for path in paths {
        if let Ok(report) = analyze(path, excludes) {
            files.extend(report.files);
        }
    }
    let total: u64 = files.iter().map(|file| file.size_bytes).sum();
    file_type_allocation::compute_file_types(&files, total)
}
