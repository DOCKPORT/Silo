//! Recursive filesystem scan and statistics computation for SiloAnalysis.
//!
//! The silo root is validated first, then walked recursively with
//! [`std::fs::read_dir`]. Every file and directory is collected with its
//! relative path, and statistics are computed from the collected entries.
//!
//! Failure policy: an unreadable sub-entry is recorded in
//! `AnalysisReport::scan_errors` and the walk continues. Only a root-level
//! failure (the root itself cannot be read) aborts the scan.

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use super::error::AnalysisError;
use super::file_type_allocation;
use super::{AnalysisReport, DirEntry, FileEntry, FileRef, Stats};

/// Analyze the silo rooted at `root`.
///
/// Validates the root, then walks it and computes statistics. The list of
/// per-entry errors is always present on the report, even when empty.
pub(crate) fn analyze(root: &Path) -> Result<AnalysisReport, AnalysisError> {
    validate_root(root)?;

    let mut files = Vec::new();
    let mut dirs = Vec::new();
    let mut scan_errors = Vec::new();

    walk_dir(root, root, &mut files, &mut dirs, &mut scan_errors)
        .map_err(AnalysisError::WalkRoot)?;

    let stats = compute_stats(&files, dirs.len() as u64);

    Ok(AnalysisReport {
        root: root.to_path_buf(),
        stats,
        files,
        dirs,
        scan_errors,
    })
}

/// Validate that the silo root exists and is a directory.
fn validate_root(root: &Path) -> Result<(), AnalysisError> {
    if !root.exists() {
        return Err(AnalysisError::PathDoesNotExist(root.to_path_buf()));
    }
    if !root.is_dir() {
        return Err(AnalysisError::PathNotADirectory(root.to_path_buf()));
    }
    Ok(())
}

/// Recursively walk `dir` and collect every file and directory under it.
///
/// A failed sub-entry is recorded as a string in `scan_errors`; the walk
/// continues with the next entry. Only a read failure on `dir` itself is
/// propagated to the caller.
fn walk_dir(
    dir: &Path,
    root: &Path,
    files: &mut Vec<FileEntry>,
    dirs: &mut Vec<DirEntry>,
    scan_errors: &mut Vec<String>,
) -> Result<(), io::Error> {
    let mut entries = fs::read_dir(dir)?;

    while let Some(entry) = entries.next() {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                scan_errors.push(format!(
                    "could not read an entry in {}: {err}",
                    dir.display()
                ));
                continue;
            }
        };

        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();

        // Cycle protection: directory symlinks are skipped so the walk cannot
        // loop back on itself. File symlinks are followed via `fs::metadata`.
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(err) => {
                scan_errors.push(format!("could not inspect {}: {err}", path.display()));
                continue;
            }
        };

        if file_type.is_dir() {
            let rel = relative_path(&path, root);
            dirs.push(DirEntry {
                name,
                relative_path: rel,
            });
            walk_dir(&path, root, files, dirs, scan_errors)?;
            continue;
        }

        if file_type.is_file() || file_type.is_symlink() {
            let metadata = match fs::metadata(&path) {
                Ok(m) => m,
                Err(err) => {
                    scan_errors.push(format!(
                        "could not read metadata for {}: {err}",
                        path.display()
                    ));
                    continue;
                }
            };
            // A symlink pointing to a directory is skipped, so the walk
            // cannot loop back on itself. Symlinks to files are recorded.
            if metadata.is_dir() {
                continue;
            }
            record_file(&path, root, &name, &metadata, files);
        }
    }

    Ok(())
}

/// Collect a single file entry with its metadata.
fn record_file(
    path: &Path,
    root: &Path,
    name: &str,
    metadata: &fs::Metadata,
    files: &mut Vec<FileEntry>,
) {
    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64);

    files.push(FileEntry {
        name: name.to_string(),
        relative_path: relative_path(path, root),
        size_bytes: metadata.len(),
        modified,
    });
}

/// Compute the path relative to the silo root.
fn relative_path(path: &Path, root: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

/// Compute summary statistics from the collected files.
fn compute_stats(files: &[FileEntry], total_dirs: u64) -> Stats {
    let total_files = files.len() as u64;
    let total_size_bytes: u64 = files.iter().map(|f| f.size_bytes).sum();

    let largest_file = files
        .iter()
        .max_by(|a, b| a.size_bytes.cmp(&b.size_bytes))
        .map(file_ref);
    let smallest_file = files
        .iter()
        .min_by(|a, b| a.size_bytes.cmp(&b.size_bytes))
        .map(file_ref);

    let average_file_size_bytes = if total_files == 0 {
        None
    } else {
        Some(total_size_bytes / total_files)
    };

    // Oldest and newest are based on the last-modified timestamp. Files with
    // no readable timestamp are excluded from both comparisons.
    let oldest_file = files
        .iter()
        .filter(|f| f.modified.is_some())
        .min_by(|a, b| a.modified.cmp(&b.modified))
        .map(file_ref);
    let newest_file = files
        .iter()
        .filter(|f| f.modified.is_some())
        .max_by(|a, b| a.modified.cmp(&b.modified))
        .map(file_ref);

    let file_types = file_type_allocation::compute_file_types(files, total_size_bytes);

    Stats {
        total_files,
        total_dirs,
        total_size_bytes,
        largest_file,
        smallest_file,
        average_file_size_bytes,
        oldest_file,
        newest_file,
        file_types,
    }
}

/// Build a `FileRef` from a file entry.
fn file_ref(entry: &FileEntry) -> FileRef {
    FileRef {
        name: entry.name.clone(),
        relative_path: entry.relative_path.clone(),
        size_bytes: entry.size_bytes,
        modified: entry.modified,
    }
}
