//! Dry run: preview a sync without changing any files.
//!
//! This module runs rsync with `--dry-run`, `--itemize-changes`, and `--stats`.
//! rsync then reports what would change and the totals, but writes nothing.
//!
//! Validation matches a real sync: the binary, the sources, and the
//! destination must all be valid before rsync runs.

use super::SyncPlan;
use super::command;
use super::error::SyncError;

/// The result of a dry run.
#[derive(Debug)]
pub struct DryRunOutcome {
    /// Captured standard output of rsync.
    pub stdout: String,
    /// Captured standard error of rsync.
    pub stderr: String,
}

/// Run a dry run and report the captured output.
///
/// Returns [`SyncError`] for pre-flight validation failures and process-level
/// errors. rsync's own exit code is ignored because `--dry-run` always exits 0
/// in normal operation.
pub fn dry_run(plan: &SyncPlan) -> Result<DryRunOutcome, SyncError> {
    super::runner::validate(plan)?;

    let output = command::build_dry_run(plan)
        .output()
        .map_err(SyncError::Process)?;

    Ok(DryRunOutcome {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}
