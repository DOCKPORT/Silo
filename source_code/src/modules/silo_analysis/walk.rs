//! Recursive filesystem scan and statistics computation for SiloAnalysis.
//!
//! The silo root is validated first, then walked recursively with
//! [`std::fs::read_dir`]. Every file and directory is collected with its
//! relative path, and statistics are computed from the collected entries.
//!
//! Failure policy: an unreadable sub-entry is recorded in
//! `AnalysisReport::scan_errors` and the walk continues. Only a root-level
//! failure (the root itself cannot be read) aborts the scan.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use super::error::AnalysisError;
use super::{AnalysisReport, DirEntry, FileEntry, FileRef, FileTypeStat, Stats};

/// The reserved extension name for files that have no extension.
const NO_EXTENSION: &str = "no-extension";

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
                scan_errors.push(format!("could not read an entry in {}: {err}", dir.display()));
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

    let file_types = compute_file_types(files, total_size_bytes);

    Stats {
        total_files,
        total_dirs,
        total_size_bytes,
        largest_file,
        smallest_file,
        average_file_size_bytes,
        file_types,
    }
}

/// Build a `FileRef` from a file entry.
fn file_ref(entry: &FileEntry) -> FileRef {
    FileRef {
        name: entry.name.clone(),
        relative_path: entry.relative_path.clone(),
        size_bytes: entry.size_bytes,
    }
}

/// Group files by extension and compute each group's share of the silo size.
///
/// Groups are ordered by total bytes descending, as required by the UI.
fn compute_file_types(files: &[FileEntry], total_size_bytes: u64) -> Vec<FileTypeStat> {
    let mut groups: BTreeMap<String, (u64, u64)> = BTreeMap::new();

    for file in files {
        let extension = file
            .relative_path
            .extension()
            .and_then(|e| e.to_str())
            .filter(|e| !e.is_empty())
            .map(|e| e.to_string())
            .unwrap_or_else(|| NO_EXTENSION.to_string());

        let entry = groups.entry(extension).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += file.size_bytes;
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
    // order, which BTreeMap already provides.
    stats.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Create a unique temporary directory for a test.
    fn temp_root(tag: &str) -> PathBuf {
        let unique = format!(
            "silo_analysis_test_{}_{}",
            tag,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock before UNIX epoch")
                .as_nanos()
        );
        std::env::temp_dir().join(unique)
    }

    /// Build a sample silo and return its root path.
    ///
    /// Layout:
    /// - a.txt        (5 bytes)
    /// - b.bin        (10 bytes)
    /// - docs/
    ///   - c.txt      (15 bytes)
    ///   - sub/
    ///     - d.log    (20 bytes)
    fn sample_silo() -> PathBuf {
        let root = temp_root("sample");
        fs::create_dir_all(root.join("docs/sub")).expect("create silo layout");

        fs::write(root.join("a.txt"), b"aaaaa").expect("write a.txt");
        fs::write(root.join("b.bin"), b"bbbbbbbbbb").expect("write b.bin");
        fs::write(root.join("docs/c.txt"), b"ccccccccccccccc").expect("write c.txt");
        fs::write(root.join("docs/sub/d.log"), b"dddddddddddddddddddd").expect("write d.log");

        root
    }

    /// Remove a temp tree if it still exists. Best effort.
    fn cleanup(root: &Path) {
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn analyzes_recursively_and_reports_files_and_dirs() {
        let root = sample_silo();
        let report = analyze(&root).expect("analysis must succeed");

        assert_eq!(report.root, root);
        assert_eq!(report.files.len(), 4);
        assert_eq!(report.dirs.len(), 2);
        assert_eq!(report.stats.total_dirs, 2);
        assert!(report.scan_errors.is_empty());

        cleanup(&root);
    }

    #[test]
    fn computes_totals_largest_smallest_and_average() {
        let root = sample_silo();
        let report = analyze(&root).expect("analysis must succeed");
        let stats = &report.stats;

        assert_eq!(stats.total_files, 4);
        assert_eq!(stats.total_size_bytes, 50);

        let largest = stats.largest_file.as_ref().expect("largest file");
        assert_eq!(largest.name, "d.log");
        assert_eq!(largest.size_bytes, 20);

        let smallest = stats.smallest_file.as_ref().expect("smallest file");
        assert_eq!(smallest.name, "a.txt");
        assert_eq!(smallest.size_bytes, 5);

        assert_eq!(stats.average_file_size_bytes, Some(12));

        cleanup(&root);
    }

    #[test]
    fn computes_file_types_with_size_share() {
        let root = sample_silo();
        let report = analyze(&root).expect("analysis must succeed");
        let types = &report.stats.file_types;

        // Expected by total bytes descending: log=20, txt=20, bin=10.
        // Ties keep BTreeMap alphabetical order, so log precedes txt.
        assert_eq!(types.len(), 3);
        assert_eq!(types[0].extension, "log");
        assert_eq!(types[0].count, 1);
        assert_eq!(types[0].total_bytes, 20);
        assert_eq!(types[0].percent_of_total_bytes, 40.0);

        assert_eq!(types[1].extension, "txt");
        assert_eq!(types[1].count, 2);
        assert_eq!(types[1].total_bytes, 20);
        assert_eq!(types[1].percent_of_total_bytes, 40.0);

        assert_eq!(types[2].extension, "bin");
        assert_eq!(types[2].count, 1);
        assert_eq!(types[2].total_bytes, 10);
        assert_eq!(types[2].percent_of_total_bytes, 20.0);

        cleanup(&root);
    }

    #[test]
    fn rejects_a_missing_path() {
        let missing = temp_root("missing").join("does_not_exist");
        match analyze(&missing) {
            Err(AnalysisError::PathDoesNotExist(p)) => assert_eq!(p, missing),
            other => panic!("expected PathDoesNotExist, got {other:?}"),
        }
    }

    #[test]
    fn rejects_a_file_path() {
        let root = temp_root("file");
        fs::create_dir_all(&root).expect("create temp root");
        let file = root.join("plain.txt");
        fs::write(&file, b"x").expect("write file");

        match analyze(&file) {
            Err(AnalysisError::PathNotADirectory(p)) => assert_eq!(p, file),
            other => panic!("expected PathNotADirectory, got {other:?}"),
        }

        cleanup(&root);
    }

    #[test]
    fn empty_silo_reports_zero_stats_and_no_file_types() {
        let root = temp_root("empty");
        fs::create_dir_all(&root).expect("create empty silo");

        let report = analyze(&root).expect("analysis must succeed");
        assert_eq!(report.files.len(), 0);
        assert_eq!(report.dirs.len(), 0);
        assert_eq!(report.stats.total_files, 0);
        assert_eq!(report.stats.total_size_bytes, 0);
        assert_eq!(report.stats.largest_file, None);
        assert_eq!(report.stats.smallest_file, None);
        assert_eq!(report.stats.average_file_size_bytes, None);
        assert!(report.stats.file_types.is_empty());

        cleanup(&root);
    }

    #[test]
    fn files_without_extension_are_grouped() {
        let root = temp_root("noext");
        fs::create_dir_all(&root).expect("create temp root");
        fs::write(root.join("Makefile"), b"all:").expect("write Makefile");
        fs::write(root.join(".gitignore"), b"target/").expect("write .gitignore");

        let report = analyze(&root).expect("analysis must succeed");
        assert_eq!(report.stats.total_files, 2);

        let no_ext = report
            .stats
            .file_types
            .iter()
            .find(|t| t.extension == NO_EXTENSION)
            .expect("no-extension group");

        assert_eq!(no_ext.count, 2);

        cleanup(&root);
    }
}