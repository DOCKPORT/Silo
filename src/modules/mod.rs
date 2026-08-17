//! Silo module tree.
//!
//! This module declares the core subsystems of Silo:
//! - config: SQLite-backed settings store (sources, excludes, destination, timestamps)
//! - sync_engine: rsync subprocess wrapper (bundled static binary)
//! - silo_analysis: filesystem analysis of the silo data folder
//! - ui: Iced graphical interface, with the main window under ui::mainwindow
//!
//! The module is declared but not yet wired into main.rs, which is intentionally
//! kept empty for now.

pub mod config;
pub mod silo_analysis;
pub mod sync_engine;
pub mod ui;
