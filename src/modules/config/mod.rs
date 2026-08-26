//! Config: SQLite-backed settings store for Silo.
//!
//! The implementation lives in the [`config_silo`] submodule, which owns the
//! SQLite schema under `~/.local/share/silo/`. This file re-exports the public
//! API so callers can use `modules::config::init()` directly.

pub mod config_silo;

pub use config_silo::{
    ConfigError, add_data_path, init, load, remove_data_path, replace_excludes,
    set_rsync_dest_path, update_exclude,
};
