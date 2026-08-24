//! SiloSize: computes the total size of the silo.
//!
//! This module loads the source folders and the exclude patterns from the
//! settings database, walks every source folder, and sums the size of every
//! file that is not excluded. The result is the on-disk size of the silo as
//! it will be mirrored, so it honors the same excludes that rsync uses.
//!
//! Exclude matching is keyword-based and simple:
//! - a keyword without a leading dot matches the exact folder or file name,
//!   at any depth
//! - a keyword starting with a dot is an extension keyword: it matches every
//!   file with that extension, ignoring upper and lower case
//! - a leading `*` on an extension keyword is sugar, so `*.mov` works like
//!   `.mov`
//! - an excluded directory prunes its whole subtree
//!
//! Failure policy: an unreadable sub-entry is skipped and the walk continues.
//! Only a source root that is missing, not a directory, or unreadable aborts
//! the computation.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::modules::config;

/// Errors produced while computing the total silo size.
#[derive(Debug)]
pub enum SiloSizeError {
    /// The settings could not be loaded from the database.
    Settings(config::ConfigError),
    /// A source folder does not exist.
    PathDoesNotExist(PathBuf),
    /// A source folder is not a directory.
    PathNotADirectory(PathBuf),
    /// A source folder could not be read.
    Walk(PathBuf, io::Error),
}

impl fmt::Display for SiloSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SiloSizeError::Settings(err) => write!(f, "could not load the settings: {err}"),
            SiloSizeError::PathDoesNotExist(path) => {
                write!(f, "source folder does not exist: {}", path.display())
            }
            SiloSizeError::PathNotADirectory(path) => {
                write!(f, "source folder is not a directory: {}", path.display())
            }
            SiloSizeError::Walk(path, err) => {
                write!(f, "could not read source folder {}: {err}", path.display())
            }
        }
    }
}

impl std::error::Error for SiloSizeError {}

impl From<config::ConfigError> for SiloSizeError {
    fn from(err: config::ConfigError) -> Self {
        SiloSizeError::Settings(err)
    }
}

/// Load the settings from the database and compute the total silo size.
///
/// Reads `silo_data_paths` and `exclude` through [`config::load`], then sums
/// every non-excluded file under every source folder.
pub fn compute() -> Result<u64, SiloSizeError> {
    let settings = config::load()?;
    total_size_bytes(&settings.silo_data_paths, &settings.excludes)
}

/// The total size of the silo in bytes, ignoring excluded entries.
///
/// Sums the size of every non-excluded file under every path in `paths`.
/// Excludes are matched against each entry's name and extension, following
/// the rules in the module docs.
pub fn total_size_bytes(paths: &[PathBuf], excludes: &[String]) -> Result<u64, SiloSizeError> {
    let mut total: u64 = 0;
    for path in paths {
        total += path_size_bytes(path, excludes)?;
    }
    Ok(total)
}

/// The size of the data that the next sync transfers to the destination.
///
/// Walks every source folder and sums the size of each non-excluded file
/// that fails rsync's quick check against its mirror at the destination: the
/// destination copy is missing, or its size or modification time differs.
/// This is the delta that rsync reports as "Total transferred file size" in
/// a dry run, so the sync progress bar measures the right total. Deletions
/// and directory attribute changes transfer no file data.
pub fn sync_delta_bytes(
    sources: &[PathBuf],
    excludes: &[String],
    destination: &Path,
) -> Result<u64, SiloSizeError> {
    let mut total: u64 = 0;
    for path in sources {
        if !path.exists() {
            return Err(SiloSizeError::PathDoesNotExist(path.to_path_buf()));
        }
        if !path.is_dir() {
            return Err(SiloSizeError::PathNotADirectory(path.to_path_buf()));
        }

        // The destination mirrors each source folder under its own name, for
        // example `/dest/<source folder>/<relative path>`.
        let Some(folder) = path.file_name() else {
            continue;
        };
        let dest_root = destination.join(folder);

        walk_delta(path, &dest_root, excludes, &mut total)
            .map_err(|err| SiloSizeError::Walk(path.to_path_buf(), err))?;
    }
    Ok(total)
}

/// The total silo size as a human-readable label.
///
/// Calls [`compute`] and formats the result through [`human_size`]. Any error
/// maps to the label `N/A`, so the UI can show a fallback.
pub fn silo_size_label() -> String {
    match compute() {
        Ok(bytes) => human_size(bytes),
        Err(_) => "N/A".to_string(),
    }
}

/// Format a byte count as a human-readable size string.
///
/// Uses binary units: 1 KiB = 1024 bytes. A zero count renders as `0 B`;
/// everything else prints with one decimal place and a space, so 6.91 GiB
/// render as `6.9 GiB`.
pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    format!("{value:.1} {}", UNITS[unit])
}

/// The size of one source folder in bytes, ignoring excluded entries.
///
/// The root must exist, be a directory, and be readable. Unreadable
/// sub-entries are skipped, matching the module's failure policy.
fn path_size_bytes(path: &Path, excludes: &[String]) -> Result<u64, SiloSizeError> {
    if !path.exists() {
        return Err(SiloSizeError::PathDoesNotExist(path.to_path_buf()));
    }
    if !path.is_dir() {
        return Err(SiloSizeError::PathNotADirectory(path.to_path_buf()));
    }

    let mut total: u64 = 0;
    walk(path, excludes, &mut total).map_err(|err| SiloSizeError::Walk(path.to_path_buf(), err))?;
    Ok(total)
}

/// Recursively walk `dir` and add the size of every non-excluded file.
///
/// An excluded directory is skipped entirely, so its subtree never counts.
/// Symlinks are skipped: directory symlinks cannot loop back, and file
/// symlinks are not double-counted.
fn walk(dir: &Path, excludes: &[String], total: &mut u64) -> io::Result<()> {
    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();

        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };

        if file_type.is_dir() {
            if is_excluded(&name, true, excludes) {
                continue;
            }
            walk(&path, excludes, total)?;
        } else if file_type.is_file() {
            if is_excluded(&name, false, excludes) {
                continue;
            }
            match entry.metadata() {
                Ok(metadata) => *total += metadata.len(),
                Err(_) => continue,
            }
        }
    }

    Ok(())
}

/// Recursively walk `dir` and add the size of every non-excluded file that
/// the next sync will transfer to `dest_dir`.
///
/// A file transfers when its copy at `dest_dir` is missing or differs in
/// size or modification time. Excluded directories are skipped entirely.
/// Symlinks are skipped: they carry no file data.
fn walk_delta(dir: &Path, dest_dir: &Path, excludes: &[String], total: &mut u64) -> io::Result<()> {
    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        let dest_path = dest_dir.join(&name);

        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };

        if file_type.is_dir() {
            if is_excluded(&name_str, true, excludes) {
                continue;
            }
            walk_delta(&path, &dest_path, excludes, total)?;
        } else if file_type.is_file() {
            if is_excluded(&name_str, false, excludes) {
                continue;
            }
            if needs_transfer(&path, &dest_path) {
                match fs::metadata(&path) {
                    Ok(metadata) => *total += metadata.len(),
                    Err(_) => continue,
                }
            }
        }
    }

    Ok(())
}

/// True when rsync would transfer the file: the destination copy is missing,
/// or its size or modification time differs.
fn needs_transfer(source: &Path, dest: &Path) -> bool {
    let Ok(dest_meta) = fs::metadata(dest) else {
        return true;
    };
    if !dest_meta.is_file() {
        return true;
    }
    let Ok(source_meta) = fs::metadata(source) else {
        return true;
    };
    if source_meta.len() != dest_meta.len() {
        return true;
    }
    mtime_secs(&source_meta) != mtime_secs(&dest_meta)
}

/// The modification time in whole seconds, matching rsync's quick check.
fn mtime_secs(metadata: &fs::Metadata) -> u64 {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

/// Whether an entry is excluded by any pattern.
///
/// A pattern without a leading dot matches the entry's exact name, at any
/// depth. A pattern starting with a dot is an extension keyword: it matches
/// every file whose extension equals the pattern, ignoring upper and lower
/// case. A leading `*` on an extension keyword is sugar, so `*.mov` works
/// like `.mov`. Extension keywords never match folders, and empty patterns
/// never exclude.
fn is_excluded(name: &str, is_dir: bool, excludes: &[String]) -> bool {
    excludes.iter().any(|pattern| {
        let pattern = pattern.trim();
        if pattern.is_empty() {
            return false;
        }

        let ext = pattern
            .strip_prefix('.')
            .or_else(|| pattern.strip_prefix("*."));
        if let Some(ext) = ext {
            if is_dir {
                return false;
            }
            let ext = ext.trim();
            if ext.is_empty() {
                return false;
            }
            let file_ext = Path::new(name)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");
            file_ext.eq_ignore_ascii_case(ext)
        } else {
            name == pattern
        }
    })
}
