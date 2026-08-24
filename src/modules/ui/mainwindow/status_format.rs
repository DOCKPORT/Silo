//! StatusFormat: turns rsync dry-run and sync outcomes into STATUS box lines.
//!
//! Pure text formatting with no UI state: the byte counts in rsync's stats
//! block are re-formatted to IEC units so they read in GiB like every other
//! size label in the UI.

use crate::modules::{silo_size, sync_engine};

/// The kind of a status line; the view maps it to a theme color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StatusKind {
    /// Neutral progress information, rendered in grey.
    Info,
    /// A successful outcome, rendered in teal.
    Success,
    /// A failed outcome, rendered in orange.
    Error,
}

/// One line of output in the Sync dialog STATUS box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StatusLine {
    /// How the line is categorized, which drives its color.
    pub(super) kind: StatusKind,
    /// The text of the line.
    pub(super) text: String,
}

/// Turns a finished dry run into the lines shown in the STATUS box.
///
/// A teal "Dry run complete" header, the rsync stats summary in grey, and any
/// rsync warnings in orange. Trailing blank space is trimmed from the output.
/// The stats byte counts are re-formatted to IEC units, so they read in GiB
/// like every other size label in the UI.
pub(super) fn dry_run_result_lines(outcome: sync_engine::DryRunOutcome) -> Vec<StatusLine> {
    let mut lines = vec![StatusLine {
        kind: StatusKind::Success,
        text: "Dry run complete".to_string(),
    }];

    let summary = reformat_stats_summary(&dry_run_summary(&outcome.stdout));
    if !summary.is_empty() {
        lines.push(StatusLine {
            kind: StatusKind::Info,
            text: summary,
        });
    }

    let stderr = outcome.stderr.trim();
    if !stderr.is_empty() {
        lines.push(StatusLine {
            kind: StatusKind::Error,
            text: stderr.to_string(),
        });
    }

    lines
}

/// Extracts the stats summary from a dry run's stdout.
///
/// The full output lists every file change; only the trailing stats block is
/// wanted for the STATUS box. The block starts at the `Number of files:` line
/// and runs to the end. Returns the whole trimmed output when the block is
/// missing.
fn dry_run_summary(stdout: &str) -> String {
    match stdout.find("Number of files:") {
        Some(index) => stdout[index..].trim().to_string(),
        None => stdout.trim().to_string(),
    }
}

/// Re-formats the byte counts in a dry-run stats summary to IEC units.
///
/// rsync prints raw byte numbers in its stats block. Every value that is a
/// byte count becomes a [`silo_size::human_size`] label, so the summary reads
/// in GiB like the rest of the UI. Lines without a byte count pass through
/// unchanged.
fn reformat_stats_summary(summary: &str) -> String {
    summary
        .lines()
        .map(reformat_stats_line)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Re-formats one stats line's byte counts to IEC units.
///
/// Handles the known byte-count shapes in the rsync stats block: `label: N
/// bytes`, `Total bytes sent/received: N`, the trailing `sent N bytes
/// received M bytes ...` line, and `total size is N speedup is ...`. Any
/// other line is returned unchanged. rsync prints thousands separators by
/// default, so the byte counts are normalized before parsing.
fn reformat_stats_line(line: &str) -> String {
    // "label: N bytes", for example "Total file size: 7,423,077,535 bytes".
    if let Some((label, raw)) = line.rsplit_once(": ") {
        if let Some(rest) = raw.strip_suffix(" bytes") {
            if let Some(bytes) = parse_bytes(rest) {
                return format!("{label}: {}", silo_size::human_size(bytes));
            }
        }
        // "Total bytes sent: 517,176" and "Total bytes received: 2,754".
        if label.starts_with("Total bytes") {
            if let Some(bytes) = parse_bytes(raw) {
                return format!("{label}: {}", silo_size::human_size(bytes));
            }
        }
    }

    // "sent 517,176 bytes  received 2,754 bytes  1,039,860.00 bytes/sec".
    if let Some(rest) = line.strip_prefix("sent ") {
        let tokens: Vec<&str> = rest.split_whitespace().collect();
        if tokens.len() >= 5
            && tokens[1] == "bytes"
            && tokens[2] == "received"
            && tokens[4] == "bytes"
        {
            if let (Some(sent), Some(received)) = (parse_bytes(tokens[0]), parse_bytes(tokens[3])) {
                return format!(
                    "sent {}  received {}  {}",
                    silo_size::human_size(sent),
                    silo_size::human_size(received),
                    tokens[5..].join(" ")
                );
            }
        }
    }

    // "total size is 7,423,077,535  speedup is 14,277.07 (DRY RUN)".
    if let Some(rest) = line.strip_prefix("total size is ") {
        if let Some((value, tail)) = rest.split_once(char::is_whitespace) {
            if let Some(bytes) = parse_bytes(value) {
                return format!(
                    "total size is {} {}",
                    silo_size::human_size(bytes),
                    tail.trim_start()
                );
            }
        }
    }

    line.to_string()
}

/// Parses a byte count that may include thousands separators.
///
/// rsync prints numbers such as `7,423,077,535` by default; the separators
/// are removed so the value parses as a plain integer.
fn parse_bytes(raw: &str) -> Option<u64> {
    raw.replace(',', "").parse().ok()
}

/// Turns a finished sync into the lines shown in the STATUS box.
///
/// Success shows a teal completion message; rsync and engine failures show
/// an orange reason. Any rsync stderr output is appended in orange, trimmed.
pub(super) fn sync_result_lines(
    result: Result<sync_engine::SyncOutcome, sync_engine::SyncError>,
) -> Vec<StatusLine> {
    match result {
        Ok(sync_engine::SyncOutcome::Success { stderr, .. }) => {
            let mut lines = vec![StatusLine {
                kind: StatusKind::Success,
                text: "Sync complete.".to_string(),
            }];
            append_sync_stderr(&mut lines, &stderr);
            lines
        }
        Ok(sync_engine::SyncOutcome::Aborted) => vec![StatusLine {
            kind: StatusKind::Error,
            text: "Sync aborted".to_string(),
        }],
        Ok(sync_engine::SyncOutcome::Failure {
            exit_code, stderr, ..
        }) => {
            let reason = match exit_code {
                Some(code) => format!("Sync failed: rsync exited with code {code}"),
                None => "Sync failed: rsync did not exit cleanly".to_string(),
            };
            let mut lines = vec![StatusLine {
                kind: StatusKind::Error,
                text: reason,
            }];
            append_sync_stderr(&mut lines, &stderr);
            lines
        }
        Err(err) => vec![StatusLine {
            kind: StatusKind::Error,
            text: format!("Sync failed: {err}"),
        }],
    }
}

/// Appends rsync's standard error to the status lines, if there is any.
///
/// rsync writes warnings and error details to stderr. They are shown in
/// orange so they stand out from the progress and completion lines.
fn append_sync_stderr(lines: &mut Vec<StatusLine>, stderr: &str) {
    let stderr = stderr.trim();
    if !stderr.is_empty() {
        lines.push(StatusLine {
            kind: StatusKind::Error,
            text: stderr.to_string(),
        });
    }
}
