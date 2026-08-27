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

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

pub use dry_run::{DryRunOutcome, dry_run};
pub use error::SyncError;

/// The rsync executable name, resolved through PATH when used as a bare name.
const RSYNC: &str = "rsync";

/// The inputs for a sync operation.
#[derive(Debug, Clone)]
pub struct SyncPlan {
    /// Source folders to mirror. Must be non-empty.
    pub sources: Vec<PathBuf>,
    /// rsync exclude patterns (for example `node_modules`, `*.log`).
    pub excludes: Vec<String>,
    /// Destination folder. Must exist and be a directory.
    pub destination: PathBuf,
    /// The rsync binary to run. [`SyncPlan::new`] resolves it with
    /// [`default_rsync_binary`]: the system `rsync` on PATH when present,
    /// otherwise the copy bundled inside the AppImage.
    pub binary: PathBuf,
}

impl SyncPlan {
    /// Create a plan with the resolved default rsync binary.
    pub fn new(sources: Vec<PathBuf>, excludes: Vec<String>, destination: PathBuf) -> Self {
        Self {
            sources,
            excludes,
            destination,
            binary: default_rsync_binary(),
        }
    }
}

/// Resolve the rsync binary to use for a sync.
///
/// Preference order:
/// 1. A system `rsync` found on PATH, so a host with its own rsync keeps
///    using it (that copy may be newer than the bundled one).
/// 2. The rsync bundled inside the AppImage at `$APPDIR/usr/bin/rsync`, which
///    makes syncs work on hosts without rsync installed.
/// 3. The same bundled location next to the running executable, covering an
///    extracted AppDir where `APPDIR` is not set.
/// 4. The bare name `rsync`, so the pre-flight check still reports
///    "rsync binary could not be found" when neither copy exists.
fn default_rsync_binary() -> PathBuf {
    let name = Path::new(RSYNC);
    if runner::find_binary(name) {
        return name.to_path_buf();
    }
    if let Some(appdir) = std::env::var_os("APPDIR") {
        let bundled = Path::new(&appdir).join("usr/bin/rsync");
        if bundled.is_file() {
            return bundled;
        }
    }
    if let Some(exe) = std::env::current_exe().ok() {
        if let Some(dir) = exe.parent() {
            let sibling = dir.join(RSYNC);
            if sibling.is_file() {
                return sibling;
            }
        }
    }
    name.to_path_buf()
}

/// The result of a sync run.
#[derive(Debug)]
pub enum SyncOutcome {
    /// rsync exited with code 0.
    Success {
        /// Captured standard error.
        stderr: String,
    },
    /// rsync exited with a non-zero code, or could not be started.
    Failure {
        /// The exit code, or `None` if rsync could not be started at all.
        exit_code: Option<i32>,
        /// Captured standard error.
        stderr: String,
    },
    /// The sync was aborted by the user.
    Aborted,
}

/// Run a sync while streaming rsync's output line by line.
///
/// `on_line` receives every line rsync writes, in order, while the process
/// runs. Returns the [`SyncOutcome`] carrying the captured standard error.
/// When `abort` becomes true, rsync is killed and the outcome is
/// [`SyncOutcome::Aborted`].
pub fn sync_streaming(
    plan: &SyncPlan,
    abort: &AtomicBool,
    on_line: impl FnMut(&str),
) -> Result<SyncOutcome, SyncError> {
    runner::sync_streaming(plan, abort, on_line)
}
