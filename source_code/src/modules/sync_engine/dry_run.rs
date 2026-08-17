//! Dry run: preview a sync without changing any files.
//!
//! This module runs rsync with `--dry-run`, `--itemize-changes`, and `--stats`.
//! rsync then reports what would change and the totals, but writes nothing.
//!
//! Validation is relaxed compared to a real sync: the binary and the sources
//! must exist, but the destination does not. This lets the user preview a
//! first-time sync to a destination that does not exist yet.

use std::path::Path;

use super::command;
use super::error::SyncError;
use super::SyncPlan;

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
    validate_sources(plan)?;

    let output = command::build_dry_run(plan)
        .output()
        .map_err(SyncError::Process)?;

    Ok(DryRunOutcome {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

/// Validate the binary and the sources. The destination is deliberately not
/// checked, so a first-time sync can be previewed.
fn validate_sources(plan: &SyncPlan) -> Result<(), SyncError> {
    if !find_binary(&plan.binary) {
        return Err(SyncError::RsyncNotFound);
    }

    if plan.sources.is_empty() {
        return Err(SyncError::NoSources);
    }

    for src in &plan.sources {
        if !src.exists() {
            return Err(SyncError::SourceDoesNotExist(src.clone()));
        }
    }

    Ok(())
}

/// True if the binary path resolves to a file.
///
/// When the binary is a bare name such as `rsync`, this searches PATH.
/// When it is an absolute path, this checks the file exists.
fn find_binary(binary: &Path) -> bool {
    if binary.components().count() == 1 {
        if let Some(path_var) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&path_var) {
                if dir.join(binary).is_file() {
                    return true;
                }
            }
        }
        false
    } else {
        binary.is_file()
    }
}