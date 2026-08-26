//! Silo module tree.
//!
//! This module declares the core subsystems of Silo:
//! - config: SQLite-backed settings store (sources, excludes, destination, timestamps)
//! - sync_engine: rsync subprocess wrapper (bundled static binary)
//! - silo_analysis: filesystem analysis, size computation, and sync deltas
//! - ui: Iced graphical interface, with the main window under ui::mainwindow
//!
//! main.rs calls `config::init()` at startup to create the settings database.
//! The remaining subsystems are compiled but not yet wired into the UI.

pub mod config;
pub mod silo_analysis;
pub use silo_analysis::silo_size;
pub mod sync_engine;
pub mod ui;
