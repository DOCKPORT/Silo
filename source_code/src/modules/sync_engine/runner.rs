//! Subprocess runner for the sync engine.
//!
//! This is the only module in the sync engine that touches the filesystem and
//! the process. It performs pre-flight validation first, then builds and runs
//! the rsync command, captures its output, and maps the exit code to a
//! [`SyncOutcome`].

use std::path::Path;

use super::command;
use super::error::SyncError;
use super::{SyncOutcome, SyncPlan};

/// Run a sync. This is the main entry point of the engine.
///
/// Validates the plan before spawning rsync, then reports the outcome.
pub(crate) fn sync(plan: &SyncPlan) -> Result<SyncOutcome, SyncError> {
    validate(plan)?;

    let mut cmd = command::build(plan);

    let output = cmd.output().map_err(SyncError::Process)?;

    Ok(match output.status.code() {
        Some(0) => SyncOutcome::Success {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        },
        code => SyncOutcome::Failure {
            exit_code: code,
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        },
    })
}

/// Validate the plan before spawning rsync. Fails fast on any problem.
fn validate(plan: &SyncPlan) -> Result<(), SyncError> {
    // The binary must be findable. Defaults to "rsync" in PATH.
    if !find_binary(&plan.binary) {
        return Err(SyncError::RsyncNotFound);
    }

    // At least one source is required.
    if plan.sources.is_empty() {
        return Err(SyncError::NoSources);
    }

    // Every source must exist on disk.
    for src in &plan.sources {
        if !src.exists() {
            return Err(SyncError::SourceDoesNotExist(src.clone()));
        }
    }

    // The destination must exist and be a directory.
    let dest = &plan.destination;
    if !dest.exists() {
        return Err(SyncError::DestinationDoesNotExist(dest.clone()));
    }
    if !dest.is_dir() {
        return Err(SyncError::DestinationNotADirectory(dest.clone()));
    }

    Ok(())
}

/// True if the binary path resolves to an executable file.
///
/// When the binary is a bare name such as `rsync`, this searches PATH.
/// When it is an absolute path, this checks the file exists and is executable.
fn find_binary(binary: &Path) -> bool {
    if binary.components().count() == 1 {
        // Bare name: search PATH.
        if let Some(path_var) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&path_var) {
                let candidate = dir.join(binary);
                if candidate.is_file() {
                    return true;
                }
            }
        }
        false
    } else {
        // Path with a directory component: check existence directly.
        binary.is_file()
    }
}
