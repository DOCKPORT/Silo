//! SyncProgress: the live sync progress model and rsync progress parser.
//!
//! Turns the progress lines that rsync prints with `--info=progress2` into a
//! [`SyncProgress`] value: the bytes transferred so far, the total transfer
//! size, and the remaining time estimate. The progress bar reads this value
//! each frame, so the ETA, size, and percent labels stay live during a sync.
//!
//! A progress2 line looks like:
//! `1,234,567  12%  123.45MB/s  0:00:05 (xfr#3, to-chk=100/200)`
//! The total transfer size is computed once at sync start from the source
//! folders, so it stays fixed for the whole run. The ETA comes from the
//! remaining bytes and rsync's reported rate, so it counts down.

use crate::modules::silo_size;

/// The live state of a running sync.
#[derive(Debug, Clone, Copy)]
pub(super) struct SyncProgress {
    /// Bytes transferred so far.
    pub(super) transferred: u64,
    /// Fixed total transfer size in bytes.
    pub(super) total: u64,
    /// Estimated remaining time, in seconds.
    pub(super) eta_secs: u64,
}

impl SyncProgress {
    /// The fraction of the bar that is filled, clamped to `0.0..=1.0`.
    pub(super) fn fraction(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        (self.transferred as f32 / self.total as f32).clamp(0.0, 1.0)
    }

    /// The percent complete, for example `36.36`.
    pub(super) fn percent(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        self.transferred as f64 / self.total as f64 * 100.0
    }

    /// The ETA label, for example `ETA: 00:12:05`.
    pub(super) fn eta_text(&self) -> String {
        format!("ETA: {}", format_eta(self.eta_secs))
    }

    /// The sizes label, for example `2.0 GiB / 5.5 GiB`.
    pub(super) fn sizes_text(&self) -> String {
        format!(
            "{} / {}",
            silo_size::human_size(self.transferred),
            silo_size::human_size(self.total)
        )
    }

    /// The percent label, for example `36.36%`.
    pub(super) fn percent_text(&self) -> String {
        // The delta estimate can slightly exceed the bytes rsync actually
        // sends, so never display past 100%.
        format!("{:.2}%", self.percent().min(100.0))
    }
}

/// Parses one rsync progress2 line into a progress value.
///
/// Returns `None` when the line is not a progress line. `total` is the fixed
/// transfer size computed at sync start; when it is zero (the size walk
/// failed), `prev` carries a fallback total derived from rsync's percentage.
/// The ETA is computed from the remaining bytes and rsync's rate, so it
/// counts down to completion.
pub(super) fn parse_line(
    line: &str,
    prev: Option<&SyncProgress>,
    total: u64,
) -> Option<SyncProgress> {
    let mut tokens = line.split_whitespace();

    let transferred = super::status_format::parse_bytes(tokens.next()?)?;
    let pct = tokens.next()?.strip_suffix('%')?.parse::<f64>().ok()?;
    let rate = parse_rate(tokens.next()?);

    // The fixed total comes from the silo-size walk. When that failed, fall
    // back to the old percent-derived estimate.
    let total = if total > 0 {
        total
    } else if pct > 0.0 {
        (transferred as f64 / pct * 100.0) as u64
    } else {
        prev.map_or(0, |p| p.total)
    };

    let eta_secs = remaining_secs(transferred, total, rate);

    Some(SyncProgress {
        transferred,
        total,
        eta_secs,
    })
}

/// The estimated remaining seconds, from the rate and the remaining bytes.
fn remaining_secs(transferred: u64, total: u64, rate: u64) -> u64 {
    if rate == 0 || total <= transferred {
        return 0;
    }
    ((total - transferred) as f64 / rate as f64) as u64
}

/// Parses a transfer rate such as `123.45MB/s` into bytes per second.
fn parse_rate(raw: &str) -> u64 {
    let value_unit = raw.trim_end_matches("/s");
    let split = value_unit
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(value_unit.len());
    let (value, unit) = value_unit.split_at(split);

    let Ok(value) = value.parse::<f64>() else {
        return 0;
    };

    let multiplier = match unit {
        "K" | "KB" => 1024.0,
        "M" | "MB" => 1024.0 * 1024.0,
        "G" | "GB" => 1024.0 * 1024.0 * 1024.0,
        "T" | "TB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => 1.0,
    };

    (value * multiplier) as u64
}

/// Formats seconds as `HH:MM:SS`, for example `00:12:05`.
fn format_eta(secs: u64) -> String {
    format!(
        "{:02}:{:02}:{:02}",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}
