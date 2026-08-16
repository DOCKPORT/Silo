//! Silo binary crate root.
//!
//! Module declarations live here so the subsystem modules (config, sync_engine,
//! ui) are compiled and reachable during development. The application entry
//! point is intentionally minimal for now.

mod modules;

fn main() {}
