use std::env;
use std::fs;
use std::path::PathBuf;

/// The app icon SVG, embedded in the binary at compile time so the desktop
/// entry is self-contained and does not depend on any path on disk.
const ICON_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/logo/silo_logo_icon.svg"
));

/// The desktop file name in the standard user applications location.
const DESKTOP_FILE: &str = "silo.desktop";

/// The desktop entry heading in a `.desktop` file.
const EXEC_PREFIX: &str = "Exec=";

/// Path to the `.desktop` file at the standard user applications location.
pub fn desktop_file_path() -> Option<PathBuf> {
    let data = dirs::data_dir()?;
    Some(data.join("applications").join(DESKTOP_FILE))
}

/// Path to the app icon copied to the standard user icons location.
pub fn icon_path() -> Option<PathBuf> {
    let data = dirs::data_dir()?;
    Some(data.join("icons").join("silo.svg"))
}

/// Read the `Exec=` value from an existing `.desktop` file.
///
/// Returns `None` when the file does not exist, cannot be read, or has no
/// `Exec=` line. A malformed entry is treated as absent so it gets rewritten.
fn parse_exec_from_desktop(path: &PathBuf) -> Option<String> {
    let text = fs::read_to_string(path).ok()?;
    for line in text.lines() {
        let line = line.trim();
        if let Some(exec) = line.strip_prefix(EXEC_PREFIX) {
            return Some(exec.to_string());
        }
    }
    None
}

/// Automatically integrates the AppImage into the system (desktop entry).
///
/// This function is a no-op when the application is *not* running as an
/// AppImage (i.e. the `APPIMAGE` environment variable is not set).
///
/// On every AppImage launch it compares the existing `.desktop` file's
/// `Exec=` value against the current `APPIMAGE` path. When the path changed
/// (the AppImage was moved or renamed) or the file is missing, it rewrites
/// the entry and copies the SVG icon to the standard icons location. When the
/// paths match, it leaves the file untouched.
///
/// Returns `()` always; any failure is logged as a warning so a missing entry
/// never stops the application from starting.
pub fn ensure() {
    #[cfg(target_os = "linux")]
    {
        // The AppImage runtime always sets this environment variable to the
        // full path of the running AppImage file.
        let Some(appimage) = env::var("APPIMAGE").ok() else {
            return; // Not running as an AppImage.
        };

        let Some(entry_path) = desktop_file_path() else {
            eprintln!("[silo] warning: could not determine home for desktop entries");
            return;
        };
        let Some(target_icon) = icon_path() else {
            eprintln!("[silo] warning: could not determine home for app icons");
            return;
        };

        // Read the current Exec= from the existing entry, if any.
        let current_exec = parse_exec_from_desktop(&entry_path);
        if current_exec.as_deref() == Some(appimage.as_str()) {
            // Paths match — nothing to update.
            return;
        }

        // Create the parent directories for the icon and the entry.
        for dir in [target_icon.parent(), entry_path.parent()]
            .into_iter()
            .flatten()
        {
            if let Err(e) = fs::create_dir_all(dir) {
                eprintln!("[silo] warning: could not create directory {dir:?}: {e}");
                return;
            }
        }

        // Copy the embedded icon unconditionally — cheap, and repairs a missing
        // or stale icon alongside the entry.
        if let Err(e) = fs::write(&target_icon, ICON_SVG) {
            eprintln!("[silo] warning: could not write app icon {target_icon:?}: {e}");
            return;
        }

        let desktop_content = format!(
            "[Desktop Entry]\n\
             Name=Silo\n\
             Exec={appimage}\n\
             Icon={}\n\
             Type=Application\n\
             Categories=Utility;\n\
             Terminal=false\n\
             Comment=Silo and Sync data\n",
            target_icon.display()
        );

        if let Err(e) = fs::write(&entry_path, desktop_content) {
            eprintln!("[silo] warning: could not write desktop entry {entry_path:?}: {e}");
            return;
        }

        let action = if current_exec.is_some() {
            "Updated"
        } else {
            "Created"
        };
        eprintln!("[silo] desktop entry {action}: {}", entry_path.display());
    }
}
