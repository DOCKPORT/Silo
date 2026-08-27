//! Silo module tree.
//!
//! This module declares the core subsystems of Silo:
//! - config: SQLite-backed settings store (sources, excludes, destination, timestamps)
//! - desktop_entry: AppImage desktop integration (installs a .desktop entry on launch)
//! - sync_engine: rsync subprocess wrapper (bundled fallback binary)
//! - silo_analysis: filesystem analysis, size computation, and sync deltas
//! - ui: Iced graphical interface, with the main window under ui::mainwindow
//!
//! main.rs calls `config::init()` at startup to create the settings database.
//! The remaining subsystems are compiled but not yet wired into the UI.

pub mod config;
pub mod desktop_entry;
pub mod silo_analysis;
pub use silo_analysis::silo_size;
pub mod sync_engine;
pub mod ui;
