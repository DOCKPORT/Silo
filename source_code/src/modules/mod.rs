//! Silo module tree.
//!
//! This module declares the core subsystems of Silo:
//! - config: SQLite-backed settings store (sources, excludes, destination, timestamps)
//! - sync_engine: rsync subprocess wrapper (bundled static binary)
//! - ui: Iced graphical interface, with the main window under ui::mainwindow
//!
//! The module is declared but not yet wired into main.rs, which is intentionally
//! kept empty for now.

pub mod config;
pub mod sync_engine;
// NOTE: UI module is disabled for now. The Iced skeleton it contains uses APIs
// that are not present in the installed iced version. UI is out of scope for the
// current sync-engine task and will be rebuilt against the real Iced API later.
// pub mod ui;
