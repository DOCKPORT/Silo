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

use crate::modules::silo_size::is_excluded;

use super::error::AnalysisError;
use super::{AnalysisReport, DirEntry, FileEntry, FileRef, Stats};

/// Analyze the silo rooted at `root`, honoring the exclude patterns.
///
/// Validates the root, then walks it and computes statistics. Excluded
/// directories are skipped entirely, so their subtree never counts; excluded
/// files are skipped too. The list of per-entry errors is always present on
/// the report, even when empty.
pub(crate) fn analyze(root: &Path, excludes: &[String]) -> Result<AnalysisReport, AnalysisError> {
    validate_root(root)?;

    let mut files = Vec::new();
    let mut dirs = Vec::new();
    let mut scan_errors = Vec::new();
    let mut empty_folders: Vec<PathBuf> = Vec::new();

    // The root itself is never counted as an empty folder.
    walk_dir(
        root,
        root,
        excludes,
        &mut files,
        &mut dirs,
        &mut scan_errors,
        &mut empty_folders,
    )
    .map_err(AnalysisError::WalkRoot)?;

    let stats = compute_stats(root, &files, dirs.len() as u64, &empty_folders);

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
/// Excluded directories are skipped entirely, so their subtree never counts;
/// excluded files are skipped too. A directory with no non-excluded children
/// has its full path pushed to `empty_folders`. Returns the number of
/// non-excluded children of `dir`, so the caller can tell whether the
/// directory was empty. A failed sub-entry is recorded as a string in
/// `scan_errors`; the walk continues with the next entry. Only a read failure
/// on `dir` itself is propagated.
fn walk_dir(
    dir: &Path,
    root: &Path,
    excludes: &[String],
    files: &mut Vec<FileEntry>,
    dirs: &mut Vec<DirEntry>,
    scan_errors: &mut Vec<String>,
    empty_folders: &mut Vec<PathBuf>,
) -> Result<u64, io::Error> {
    let mut entries = fs::read_dir(dir)?;
    let mut child_count: u64 = 0;

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

        let is_dir = file_type.is_dir();
        if is_excluded(&name, is_dir, excludes) {
            continue;
        }

        if is_dir {
            child_count += 1;
            let rel = relative_path(&path, root);
            dirs.push(DirEntry {
                name,
                relative_path: rel,
            });
            let sub_children = walk_dir(
                &path,
                root,
                excludes,
                files,
                dirs,
                scan_errors,
                empty_folders,
            )?;
            // A directory with no non-excluded children is empty.
            if sub_children == 0 {
                empty_folders.push(path);
            }
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
            child_count += 1;
            record_file(&path, root, &name, &metadata, files);
        }
    }

    Ok(child_count)
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
fn compute_stats(
    root: &Path,
    files: &[FileEntry],
    total_dirs: u64,
    empty_folders: &[PathBuf],
) -> Stats {
    let total_files = files.len() as u64;
    let total_size_bytes: u64 = files.iter().map(|f| f.size_bytes).sum();
    let zero_byte_files = files
        .iter()
        .filter(|f| f.size_bytes == 0)
        .map(|f| file_ref(root, f))
        .collect();

    let largest_file = files
        .iter()
        .max_by(|a, b| a.size_bytes.cmp(&b.size_bytes))
        .map(|f| file_ref(root, f));
    // Zero-byte files never win: the smallest file must hold at least one
    // byte. Zero-byte files have their own stat.
    let smallest_file = files
        .iter()
        .filter(|f| f.size_bytes >= 1)
        .min_by(|a, b| a.size_bytes.cmp(&b.size_bytes))
        .map(|f| file_ref(root, f));

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
        .map(|f| file_ref(root, f));
    let newest_file = files
        .iter()
        .filter(|f| f.modified.is_some())
        .max_by(|a, b| a.modified.cmp(&b.modified))
        .map(|f| file_ref(root, f));

    Stats {
        total_files,
        total_dirs,
        empty_folder_paths: empty_folders.to_vec(),
        total_size_bytes,
        zero_byte_files,
        largest_file,
        smallest_file,
        average_file_size_bytes,
        oldest_file,
        newest_file,
        // The per-folder file-type breakdown is never consumed: the caller
        // recomputes one global breakdown over every folder, so it stays empty.
        file_types: Vec::new(),
    }
}

/// Build a `FileRef` from a file entry.
fn file_ref(root: &Path, entry: &FileEntry) -> FileRef {
    FileRef {
        name: entry.name.clone(),
        root: root.to_path_buf(),
        relative_path: entry.relative_path.clone(),
        size_bytes: entry.size_bytes,
        modified: entry.modified,
    }
}
