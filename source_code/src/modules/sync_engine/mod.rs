//! SyncEngine: rsync subprocess wrapper for Silo.
//!
//! This module builds the rsync command line from the silo settings
//! (source folders, exclude patterns, destination) and runs the rsync binary
//! as a subprocess, capturing its output. Sync uses `--delete` so the
//! destination is a 100% mirror of the source.
//!
//! Design: the engine is split into four internal pieces:
//! - [`command`]: pure command builder (no I/O)
//! - [`dry_run`]: simulation without writing files
//! - [`runner`]: validation + subprocess execution
//! - [`error`]: typed errors

mod command;
mod dry_run;
mod error;
mod runner;

use std::path::PathBuf;

pub use dry_run::{DryRunOutcome, dry_run};
pub use error::SyncError;

/// The inputs for a sync operation.
#[derive(Debug, Clone)]
pub struct SyncPlan {
    /// Source folders to mirror. Must be non-empty.
    pub sources: Vec<PathBuf>,
    /// rsync exclude patterns (for example `node_modules`, `*.log`).
    pub excludes: Vec<String>,
    /// Destination folder. Must exist and be a directory.
    pub destination: PathBuf,
    /// The rsync binary to run. Defaults to `rsync` (found via PATH).
    pub binary: PathBuf,
}

impl SyncPlan {
    /// Create a plan with the default `rsync` binary.
    pub fn new(sources: Vec<PathBuf>, excludes: Vec<String>, destination: PathBuf) -> Self {
        Self {
            sources,
            excludes,
            destination,
            binary: PathBuf::from("rsync"),
        }
    }

    /// Set a custom rsync binary path. Used later for the bundled binary.
    pub fn with_binary(mut self, binary: PathBuf) -> Self {
        self.binary = binary;
        self
    }
}

/// The result of a sync run.
#[derive(Debug)]
pub enum SyncOutcome {
    /// rsync exited with code 0.
    Success {
        /// Captured standard output.
        stdout: String,
        /// Captured standard error.
        stderr: String,
    },
    /// rsync exited with a non-zero code, or could not be started.
    Failure {
        /// The exit code, or `None` if rsync could not be started at all.
        exit_code: Option<i32>,
        /// Captured standard output.
        stdout: String,
        /// Captured standard error.
        stderr: String,
    },
}

/// Run a sync and report the outcome.
///
/// Validates the plan first, then runs rsync and maps its exit code to a
/// [`SyncOutcome`]. Returns [`SyncError`] for pre-flight validation failures
/// and process-level errors.
pub fn sync(plan: &SyncPlan) -> Result<SyncOutcome, SyncError> {
    runner::sync(plan)
}
