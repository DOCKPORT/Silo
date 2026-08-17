//! Typed errors for the sync engine.
//!
//! These are the things that can go wrong at the sync engine boundary:
//! pre-flight validation failures and process-level failures (spawn/wait).
//!
//! Runtime failures reported by rsync itself (non-zero exit codes) are NOT
//! represented here; they are returned as a `Failure` outcome in `SyncOutcome`.

use std::fmt;
use std::io;
use std::path::PathBuf;

/// Errors produced by the sync engine.
#[derive(Debug)]
pub enum SyncError {
    /// The binary used for syncing (system `rsync` today, bundled later) could not be found.
    RsyncNotFound,

    /// The source list was empty. At least one source folder is required.
    NoSources,

    /// A source path does not exist on disk.
    SourceDoesNotExist(PathBuf),

    /// The destination path does not exist.
    DestinationDoesNotExist(PathBuf),

    /// The destination exists but is not a directory.
    DestinationNotADirectory(PathBuf),

    /// An I/O or process-level failure (for example, spawn or wait failed).
    Process(io::Error),
}

impl fmt::Display for SyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyncError::RsyncNotFound => write!(f, "rsync binary could not be found"),
            SyncError::NoSources => {
                write!(f, "source list is empty; at least one source is required")
            }
            SyncError::SourceDoesNotExist(path) => {
                write!(f, "source does not exist: {}", path.display())
            }
            SyncError::DestinationDoesNotExist(path) => {
                write!(f, "destination does not exist: {}", path.display())
            }
            SyncError::DestinationNotADirectory(path) => {
                write!(f, "destination is not a directory: {}", path.display())
            }
            SyncError::Process(err) => write!(f, "process error: {err}"),
        }
    }
}

impl std::error::Error for SyncError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SyncError::Process(err) => Some(err),
            _ => None,
        }
    }
}
