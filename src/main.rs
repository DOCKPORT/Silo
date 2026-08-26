//! Silo binary crate root.
//!
//! Launches the Iced main window. The subsystem modules (config, desktop_entry,
//! sync_engine, silo_analysis, ui) are declared here so they are compiled and
//! reachable.

mod modules;

fn main() -> iced::Result {
    // Resolve the scale factor before any other startup work so the UI
    // renders correctly from the very first frame.
    modules::ui::scaling::Scaling::init();

    // Ensure the settings database exists under ~/.local/share/silo before the
    // UI starts. A missing or broken config store is fatal.
    if let Err(err) = modules::config::init() {
        eprintln!("silo: could not initialize the settings database: {err}");
        std::process::exit(1);
    }

    // When running as an AppImage, install or refresh the desktop entry so the
    // app appears in the system launcher. This is a no-op for plain builds.
    modules::desktop_entry::ensure();

    modules::ui::mainwindow::run()
}
