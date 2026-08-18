//! Silo binary crate root.
//!
//! Launches the Iced main window. The subsystem modules (config, sync_engine,
//! silo_analysis, ui) are declared here so they are compiled and reachable.

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

    modules::ui::mainwindow::run()
}
