//! Pure rsync command builder.
//!
//! This module turns a [`SyncPlan`] into a [`std::process::Command`].
//! It performs no I/O and never runs rsync, so it is easy to reason about.
//!
//! The built command is, roughly:
//! `rsync -a --delete --exclude=<each> <each source> <destination>/`
//!
//! Sources are passed as-is (no trailing slash), so rsync copies the folder
//! itself into the destination. The destination keeps a trailing slash, which
//! makes it the mirror root that holds each source as its own named folder.
//! All arguments are passed without a shell (`Command::arg`), so paths with
//! spaces or special characters are handled safely.

use std::ffi::OsString;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::Path;
use std::process::Command;

use super::SyncPlan;

/// Build a rsync `Command` from a sync plan.
///
/// The command is not executed. The caller decides how to run and wait on it.
pub(crate) fn build(plan: &SyncPlan) -> Command {
    let mut cmd = Command::new(&plan.binary);

    // Archive mode: preserve permissions, timestamps, symlinks, and more.
    cmd.arg("-a");

    // Mirror semantics: the destination matches the source exactly.
    cmd.arg("--delete");

    // Empty patterns never exclude anything; skip them so rsync never
    // receives an empty `--exclude=` argument.
    for ex in &plan.excludes {
        if !ex.trim().is_empty() {
            cmd.arg(format!("--exclude={ex}"));
        }
    }

    // Sources are passed as-is so the folder itself is copied into the destination.
    for src in &plan.sources {
        cmd.arg(strip_trailing_slash(src));
    }

    cmd.arg(with_trailing_slash(&plan.destination));

    cmd
}

/// Build a rsync `Command` for a real sync with live progress reporting.
///
/// Identical to [`build`], plus `--info=progress2`. rsync then writes one
/// progress line to standard error per update: the bytes transferred, the
/// percentage complete, the transfer rate, and an ETA. Progress stops when
/// the transfer ends.
pub(crate) fn build_progress(plan: &SyncPlan) -> Command {
    let mut cmd = build(plan);
    cmd.arg("--info=progress2");
    cmd
}

/// Build a rsync `Command` for a dry run simulation.
///
/// The command is identical to a real sync, with three extra flags:
/// - `--dry-run`: simulate the transfer without writing any files
/// - `--itemize-changes`: list every change with a detail prefix
/// - `--stats`: report totals such as file count and byte count
///
/// Sizes are left as raw byte numbers. rsync's `-h` flag would print decimal
/// units (for example `7.42G`); the UI re-formats the raw counts in IEC units
/// so every size label in the app reads the same way.
///
/// The command is not executed. The caller decides how to run and wait on it.
pub(crate) fn build_dry_run(plan: &SyncPlan) -> Command {
    let mut cmd = build(plan);
    cmd.arg("--dry-run");
    cmd.arg("--itemize-changes");
    cmd.arg("--stats");
    cmd
}

/// Remove a trailing slash from a path.
///
/// This makes the source an exact folder reference so rsync copies the folder
/// itself (with its name) into the destination. The path is handled as raw
/// bytes, so a non-UTF-8 name survives intact.
fn strip_trailing_slash(path: &Path) -> OsString {
    let bytes = path.as_os_str().as_bytes();
    let trimmed = bytes.strip_suffix(b"/").unwrap_or(bytes);
    OsString::from_vec(trimmed.to_vec())
}

/// Append a trailing slash to a path if it does not already have one.
///
/// This marks the destination as the mirror root that holds each source
/// folder. The path is handled as raw bytes, so a non-UTF-8 name survives
/// intact.
fn with_trailing_slash(path: &Path) -> OsString {
    let bytes = path.as_os_str().as_bytes();
    if bytes.ends_with(b"/") {
        path.as_os_str().to_os_string()
    } else {
        let mut s = path.as_os_str().to_os_string();
        s.push("/");
        s
    }
}
