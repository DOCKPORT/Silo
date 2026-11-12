//! FileExplorer: opens a folder in the desktop's native file manager.
//!
//! The app must not trust the desktop default handler for `inode/directory`.
//! That default can point to a different application, for example an editor,
//! so a folder click would show the wrong app. This module starts the
//! desktop's own file manager directly, so the click shows the real folder.
//!
//! The file manager is selected in two steps:
//! - the desktop named by `XDG_CURRENT_DESKTOP` gets its matching file manager
//! - the remaining known file managers are tried in a fixed order
//!
//! The first process that starts wins. When no known file manager starts, the
//! module falls back to `gio open` and then `xdg-open`, which honor the
//! desktop default. A failed launch is logged and never stops the app.
//!
//! An AppImage puts its bundled libraries first on `LD_LIBRARY_PATH`, and the
//! child process inherits that path. A system file manager then loads the
//! bundled libraries and can fail on a missing symbol. Before each launch,
//! this module removes the variables that point inside the AppImage, so the
//! child uses the system libraries that it was built for. Entries that point
//! outside the AppImage stay, so a user library path is kept.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The known file managers, in the order to try when the desktop is unknown.
const FILE_MANAGERS: [&str; 6] = ["thunar", "nautilus", "nemo", "caja", "dolphin", "pcmanfm"];

/// The desktop launcher that follows the GLib handler settings.
const GIO: &str = "gio";

/// The XDG launcher that follows the desktop default handler.
const XDG_OPEN: &str = "xdg-open";

/// Opens `path` in the desktop's native file manager.
///
/// The desktop's own file manager is tried first, then the remaining known
/// file managers, then the two launchers. The call returns after the first
/// process starts. A total failure is logged to standard error.
pub fn open(path: &Path) {
    let preferred = std::env::var("XDG_CURRENT_DESKTOP")
        .ok()
        .and_then(|desktop| desktop_file_manager(&desktop));

    // The desktop's own file manager comes first, then the rest in fixed
    // order. Filtering removes the duplicate when the desktop is known.
    let candidates = preferred.into_iter().chain(
        FILE_MANAGERS
            .into_iter()
            .filter(|name| Some(*name) != preferred),
    );
    for binary in candidates {
        if spawn_and_reap(binary, &[path.as_os_str()]) {
            return;
        }
    }

    // Fall back to the desktop default through the two standard launchers.
    if spawn_and_reap(GIO, &["open".as_ref(), path.as_os_str()]) {
        return;
    }
    if !spawn_and_reap(XDG_OPEN, &[path.as_os_str()]) {
        eprintln!("silo: could not open the folder {path:?}: no file manager found");
    }
}

/// The file manager that belongs to `desktop`, or `None` for an unknown one.
///
/// `desktop` is the value of `XDG_CURRENT_DESKTOP`, for example `XFCE` or
/// `ubuntu:GNOME`. The comparison ignores upper and lower case.
fn desktop_file_manager(desktop: &str) -> Option<&'static str> {
    let desktop = desktop.to_ascii_uppercase();
    if desktop.contains("XFCE") {
        Some("thunar")
    } else if desktop.contains("CINNAMON") {
        Some("nemo")
    } else if desktop.contains("MATE") {
        Some("caja")
    } else if desktop.contains("KDE") {
        Some("dolphin")
    } else if desktop.contains("GNOME") || desktop.contains("UNITY") {
        Some("nautilus")
    } else if desktop.contains("LXDE") || desktop.contains("LXQT") {
        Some("pcmanfm")
    } else {
        None
    }
}

/// Starts `binary` with `args` and reaps the child on a detached thread.
///
/// Returns `true` when the process started. The detached thread waits on the
/// child, so the app never leaves a zombie process behind.
fn spawn_and_reap(binary: &str, args: &[&std::ffi::OsStr]) -> bool {
    let mut cmd = Command::new(binary);
    cmd.args(args);
    strip_appimage_env(&mut cmd);
    let Ok(mut child) = cmd.spawn() else {
        return false;
    };
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    true
}

/// The AppImage root of the running process, or `None` for a plain build.
fn appimage_root() -> Option<PathBuf> {
    if let Some(appdir) = std::env::var_os("APPDIR") {
        return Some(PathBuf::from(appdir));
    }
    let appimage = std::env::var_os("APPIMAGE")?;
    Path::new(&appimage).parent().map(Path::to_path_buf)
}

/// Remove the entries of a path-list variable that point inside `root`.
fn remove_appimage_entries(cmd: &mut Command, name: &str, root: &Path) {
    let Some(value) = std::env::var_os(name) else {
        return;
    };
    let kept: Vec<PathBuf> = std::env::split_paths(&value)
        .filter(|entry| !entry.starts_with(root))
        .collect();
    if kept.is_empty() {
        cmd.env_remove(name);
        return;
    }
    match std::env::join_paths(&kept) {
        Ok(joined) => {
            cmd.env(name, joined);
        }
        Err(_) => {
            cmd.env_remove(name);
        }
    }
}

/// Remove a single-path variable when it points inside `root`.
fn remove_appimage_path(cmd: &mut Command, name: &str, root: &Path) {
    let Some(value) = std::env::var_os(name) else {
        return;
    };
    if Path::new(&value).starts_with(root) {
        cmd.env_remove(name);
    }
}

/// Drop the AppImage paths from the child environment.
///
/// The variables that carry the bundled libraries, modules, and schemas are
/// filtered, so the child process uses the system copies. The AppImage markers
/// are removed too. A process that runs outside an AppImage starts with an
/// unchanged environment.
fn strip_appimage_env(cmd: &mut Command) {
    let Some(root) = appimage_root() else {
        return;
    };
    for name in ["LD_LIBRARY_PATH", "LD_PRELOAD"] {
        remove_appimage_entries(cmd, name, &root);
    }
    for name in [
        "GDK_PIXBUF_MODULE_FILE",
        "GSETTINGS_SCHEMA_DIR",
        "PANGO_MODULE_FILE",
    ] {
        remove_appimage_path(cmd, name, &root);
    }
    cmd.env_remove("APPDIR");
    cmd.env_remove("APPIMAGE");
}
