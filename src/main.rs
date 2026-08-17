//! Silo binary crate root.
//!
//! Launches the Iced main window. The subsystem modules (config, sync_engine,
//! silo_analysis, ui) are declared here so they are compiled and reachable.

mod modules;

fn main() -> iced::Result {
    modules::ui::mainwindow::run()
}
