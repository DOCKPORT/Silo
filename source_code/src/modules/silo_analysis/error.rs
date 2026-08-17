//! Typed errors for the silo analysis engine.
//!
//! These are the things that can go wrong at the analysis boundary:
//! validation failures for the silo path and root-level walk failures.
//! Individual unreadable sub-entries inside the silo are NOT returned as
//! errors; they are recorded in `AnalysisReport::scan_errors` instead.

use std::fmt;
use std::io;
use std::path::PathBuf;

/// Errors produced by the silo analysis engine.
#[derive(Debug)]
pub enum AnalysisError {
    /// The path to analyze does not exist on disk.
    PathDoesNotExist(PathBuf),

    /// The path to analyze exists but is not a directory.
    PathNotADirectory(PathBuf),

    /// The root directory could not be read, or the walk failed at the
    /// root level. Nested per-entry failures are recorded separately in
    /// `AnalysisReport::scan_errors`.
    WalkRoot(io::Error),
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnalysisError::PathDoesNotExist(path) => {
                write!(f, "path to analyze does not exist: {}", path.display())
            }
            AnalysisError::PathNotADirectory(path) => {
                write!(f, "path to analyze is not a directory: {}", path.display())
            }
            AnalysisError::WalkRoot(err) => write!(f, "failed to walk the silo root: {err}"),
        }
    }
}

impl std::error::Error for AnalysisError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AnalysisError::WalkRoot(err) => Some(err),
            _ => None,
        }
    }
}