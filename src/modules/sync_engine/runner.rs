//! Subprocess runner for the sync engine.
//!
//! This is the only module in the sync engine that touches the filesystem and
//! the process. It performs pre-flight validation first, then builds and runs
//! the rsync command, captures its output, and maps the exit code to a
//! [`SyncOutcome`].

use std::io::{self, BufReader, Read};
use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use super::command;
use super::error::SyncError;
use super::{SyncOutcome, SyncPlan};

/// Run a sync while streaming rsync's output line by line.
///
/// `on_line` is called for every line rsync writes, in order, as the process
/// runs. rsync writes its progress lines (`--info=progress2`) to standard
/// output and its warnings to standard error. Only the standard error
/// output is kept in the final [`SyncOutcome`], so the STATUS box shows only
/// the warnings.
///
/// When `abort` becomes true, rsync is killed and the outcome is
/// [`SyncOutcome::Aborted`]. The flag is checked between reads, so the abort
/// takes effect as soon as the next output chunk arrives.
pub(crate) fn sync_streaming(
    plan: &SyncPlan,
    abort: &AtomicBool,
    mut on_line: impl FnMut(&str),
) -> Result<SyncOutcome, SyncError> {
    validate(plan)?;

    let mut cmd = command::build_progress(plan);

    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(SyncError::Process)?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| SyncError::Process(io::Error::other("stdout pipe unavailable")))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| SyncError::Process(io::Error::other("stderr pipe unavailable")))?;

    // A helper thread drains standard error so rsync never blocks on it. The
    // main thread reads standard output live, so progress reaches the UI as
    // it happens.
    let stderr_lines: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let stderr_buffer = Arc::clone(&stderr_lines);
    let stderr_thread = std::thread::spawn(move || {
        read_lines(stderr, None, &mut |line| {
            stderr_buffer.lock().unwrap().push(line);
        });
    });

    // Progress lines stream live from stdout; the outcome keeps only stderr.
    read_lines(stdout, Some(abort), &mut |line| {
        on_line(&line);
    });

    // The user pressed ABORT SYNC: kill rsync so the pipes close and the
    // child can be reaped. Killing the child also ends the stderr thread.
    if abort.load(Ordering::Relaxed) {
        let _ = child.kill();
    }

    // Now the warnings that were buffered from stderr.
    let _ = stderr_thread.join();
    let mut clean_stderr = String::new();
    let stderr_lines = stderr_lines.lock().unwrap().clone();
    for line in stderr_lines {
        on_line(&line);
        if !is_progress_line(&line) {
            clean_stderr.push_str(&line);
            clean_stderr.push('\n');
        }
    }

    let status = child.wait().map_err(SyncError::Process)?;

    if abort.load(Ordering::Relaxed) {
        return Ok(SyncOutcome::Aborted);
    }

    Ok(match status.code() {
        Some(0) => SyncOutcome::Success {
            stderr: clean_stderr,
        },
        code => SyncOutcome::Failure {
            exit_code: code,
            stderr: clean_stderr,
        },
    })
}

/// Reads a pipe, splits it on `\r` and `\n`, and calls `emit` per line.
///
/// rsync separates progress updates with `\r` and only the final line ends
/// with `\n`, so the reader must split on both. When `abort` is set and
/// becomes true, the reader stops early so the caller can kill the child.
fn read_lines<R: Read>(reader: R, abort: Option<&AtomicBool>, emit: &mut dyn FnMut(String)) {
    let mut reader = BufReader::new(reader);
    let mut buffer: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 4096];

    loop {
        if abort.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
            return;
        }
        let n = match reader.read(&mut chunk) {
            Ok(n) => n,
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        };
        if n == 0 {
            break;
        }
        for &byte in &chunk[..n] {
            if byte == b'\r' || byte == b'\n' {
                if !buffer.is_empty() {
                    emit(String::from_utf8_lossy(&buffer).into_owned());
                    buffer.clear();
                }
            } else {
                buffer.push(byte);
            }
        }
    }

    // A trailing partial line without a terminator.
    if !buffer.is_empty() {
        emit(String::from_utf8_lossy(&buffer).into_owned());
    }
}

/// True if a line is a progress update from `--info=progress2`.
///
/// Such a line starts with a byte count followed by a percentage, for example
/// `1,234,567  12%  123.45MB/s  0:00:05 (xfr#3, to-chk=10/20)`.
fn is_progress_line(line: &str) -> bool {
    let mut tokens = line.split_whitespace();
    let bytes = tokens.next().is_some_and(|t| parse_bytes(t).is_some());
    let pct = tokens
        .next()
        .is_some_and(|t| t.ends_with('%') && t.trim_end_matches('%').parse::<f64>().is_ok());
    bytes && pct
}

/// Parses a byte count that may include thousands separators.
fn parse_bytes(raw: &str) -> Option<u64> {
    raw.replace(',', "").parse().ok()
}

/// Validate the plan before running rsync. Fails fast on any problem.
///
/// Shared by the real sync and the dry run, so both reject the same invalid
/// plans: a missing binary, an empty source list, a missing source, or a
/// destination that does not exist or is not a directory.
pub(crate) fn validate(plan: &SyncPlan) -> Result<(), SyncError> {
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
