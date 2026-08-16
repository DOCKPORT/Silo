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

    for ex in &plan.excludes {
        cmd.arg(format!("--exclude={ex}"));
    }

    // Sources are passed as-is so the folder itself is copied into the destination.
    for src in &plan.sources {
        cmd.arg(strip_trailing_slash(src));
    }

    cmd.arg(with_trailing_slash(&plan.destination));

    cmd
}

/// Remove a trailing slash from a path.
///
/// This makes the source an exact folder reference so rsync copies the folder
/// itself (with its name) into the destination.
fn strip_trailing_slash(path: &Path) -> String {
    let s = path.to_string_lossy();
    s.strip_suffix('/').unwrap_or(&s).to_string()
}

/// Append a trailing slash to a path if it does not already have one.
///
/// This marks the destination as the mirror root that holds each source folder.
fn with_trailing_slash(path: &Path) -> String {
    let s = path.to_string_lossy();
    if s.ends_with('/') {
        s.into_owned()
    } else {
        format!("{s}/")
    }
}
