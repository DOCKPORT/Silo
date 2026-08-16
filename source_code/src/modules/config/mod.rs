//! Config: SQLite-backed settings store for Silo.
//!
//! This module is responsible for persisting silo settings under the user's
//! ~/.local directory. It stores only settings — selected source folder paths,
//! exclude patterns, the destination path, and timestamps (e.g. last sync).
//! It never stores the data files or folders themselves; the real data is read
//! live from disk by rsync at sync time.
//!
//! TODO: Implement the schema and read/write logic in a later step.
