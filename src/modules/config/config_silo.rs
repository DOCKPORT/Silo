//! Config: SQLite-backed settings store for Silo.
//!
//! This module persists the silo settings under `~/.local/share/silo/` in a
//! SQLite database. It stores settings only: selected source folder paths,
//! exclude patterns, and the rsync destination path. It never stores the data
//! files or folders themselves; rsync reads the real data live from disk at
//! sync time.
//!
//! The database uses one table per setting:
//! - `silo_data_paths`: one row per selected source folder
//! - `exclude`: one row per exclude pattern
//! - `rsync_dest_path`: a single row holding the rsync destination path
//!
//! Call [`init`] once at startup. It creates the directory, the database file,
//! the three tables, and the singleton row. It is idempotent, so calling it on
//! every launch creates the store when it is missing and never deletes existing
//! settings.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension};

/// File name of the settings database.
const DB_FILE_NAME: &str = "silo.db";

/// Location of the settings directory relative to the user's data home.
///
/// Used only when `XDG_DATA_HOME` is unset, in which case the XDG default
/// `$HOME/.local/share` applies.
const DB_DIR_NAME: &str = ".local/share/silo";

/// The silo settings persisted in the database.
///
/// The shapes match what the sync engine consumes ([`crate::modules::sync_engine::SyncPlan`]),
/// so the UI and the engine can pass the settings through directly.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SiloSettings {
    /// Selected source folders. Stored one row per folder in `silo_data_paths`.
    pub silo_data_paths: Vec<PathBuf>,
    /// Exclude patterns. Stored one row per pattern in `exclude`.
    pub excludes: Vec<String>,
    /// Destination folder for rsync, or `None` until the user picks one.
    pub rsync_dest_path: Option<PathBuf>,
}

/// Errors produced by the config store.
#[derive(Debug)]
pub enum ConfigError {
    /// The `HOME` environment variable is not set.
    HomeDirNotFound,
    /// The settings directory could not be created.
    CreateDir(io::Error),
    /// A database read, write, or schema operation failed.
    Sql(rusqlite::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::HomeDirNotFound => {
                write!(f, "HOME environment variable is not set")
            }
            ConfigError::CreateDir(err) => {
                write!(f, "failed to create the settings directory: {err}")
            }
            ConfigError::Sql(err) => write!(f, "settings database error: {err}"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::CreateDir(err) => Some(err),
            ConfigError::Sql(err) => Some(err),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for ConfigError {
    fn from(err: rusqlite::Error) -> Self {
        ConfigError::Sql(err)
    }
}

/// The settings directory, following the XDG data home convention.
///
/// Uses `$XDG_DATA_HOME/silo` when `XDG_DATA_HOME` is set, and falls back to
/// `$HOME/.local/share/silo` otherwise.
fn default_db_dir() -> Result<PathBuf, ConfigError> {
    if let Some(xdg) = std::env::var_os("XDG_DATA_HOME").filter(|value| !value.is_empty()) {
        return Ok(PathBuf::from(xdg).join("silo"));
    }
    let home = std::env::var_os("HOME").ok_or(ConfigError::HomeDirNotFound)?;
    Ok(PathBuf::from(home).join(DB_DIR_NAME))
}

/// The settings database path inside the settings directory.
fn default_db_path() -> Result<PathBuf, ConfigError> {
    Ok(default_db_dir()?.join(DB_FILE_NAME))
}

/// Ensure that the settings directory, database file, tables, and singleton
/// rows exist.
///
/// Call this once at startup. It is idempotent and never deletes existing
/// settings.
pub fn init() -> Result<(), ConfigError> {
    init_at(&default_db_dir()?)?;
    Ok(())
}

/// The same as [`init`], but creates the store under `dir`.
///
/// Returns the path of the created database file. This variant exists so tests
/// can run against a temporary directory without touching the real `$HOME`.
fn init_at(dir: &Path) -> Result<PathBuf, ConfigError> {
    fs::create_dir_all(dir).map_err(ConfigError::CreateDir)?;
    let db_path = dir.join(DB_FILE_NAME);
    let conn = Connection::open(&db_path)?;
    create_schema(&conn)?;
    Ok(db_path)
}

/// Create the three settings tables and seed the singleton row.
///
/// `CREATE TABLE IF NOT EXISTS` makes this safe to run on every launch.
fn create_schema(conn: &Connection) -> Result<(), ConfigError> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS silo_data_paths (
            path TEXT NOT NULL
        );

        CREATE UNIQUE INDEX IF NOT EXISTS uq_silo_data_paths_path
            ON silo_data_paths (path);

        CREATE TABLE IF NOT EXISTS exclude (
            pattern TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS rsync_dest_path (
            path TEXT
        );

        INSERT INTO rsync_dest_path (path)
            SELECT NULL WHERE NOT EXISTS (SELECT 1 FROM rsync_dest_path);
        "#,
    )?;
    Ok(())
}

/// Load the silo settings from the default database.
///
/// The store must be initialized with [`init`] first.
pub fn load() -> Result<SiloSettings, ConfigError> {
    load_from(&default_db_path()?)
}

/// The same as [`load`], but reads from `db`.
fn load_from(db: &Path) -> Result<SiloSettings, ConfigError> {
    let conn = Connection::open(db)?;

    let silo_data_paths = load_text_column(&conn, "silo_data_paths", "path")?
        .into_iter()
        .map(PathBuf::from)
        .collect();

    let excludes = load_text_column(&conn, "exclude", "pattern")?;

    let rsync_dest_path = conn
        .query_row("SELECT path FROM rsync_dest_path", [], |row| {
            let value: Option<String> = row.get(0)?;
            Ok(value)
        })
        .optional()?
        .flatten()
        .map(PathBuf::from);

    Ok(SiloSettings {
        silo_data_paths,
        excludes,
        rsync_dest_path,
    })
}

/// Read every text value from one table column, in `id` order.
///
/// `table` and `column` are internal constants, never user input.
fn load_text_column(
    conn: &Connection,
    table: &str,
    column: &str,
) -> Result<Vec<String>, ConfigError> {
    // `rowid` preserves insertion order whether or not the table declares an
    // explicit `id` column.
    let sql = format!("SELECT {column} FROM {table} ORDER BY rowid");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        let value: String = row.get(0)?;
        Ok(value)
    })?;
    let values = rows.collect::<Result<Vec<_>, _>>()?;
    Ok(values)
}

/// Persist the silo settings to the default database, replacing all previous
/// values.
///
/// The store must be initialized with [`init`] first.
pub fn save(settings: &SiloSettings) -> Result<(), ConfigError> {
    save_to(&default_db_path()?, settings)
}

/// The same as [`save`], but writes to `db`.
fn save_to(db: &Path, settings: &SiloSettings) -> Result<(), ConfigError> {
    let mut conn = Connection::open(db)?;
    let tx = conn.transaction()?;

    // Replace the multi-valued settings wholesale.
    tx.execute("DELETE FROM silo_data_paths", [])?;
    tx.execute("DELETE FROM exclude", [])?;

    {
        let mut stmt = tx.prepare("INSERT INTO silo_data_paths (path) VALUES (?1)")?;
        for path in &settings.silo_data_paths {
            stmt.execute([path.to_string_lossy().into_owned()])?;
        }
    }

    {
        let mut stmt = tx.prepare("INSERT INTO exclude (pattern) VALUES (?1)")?;
        for pattern in &settings.excludes {
            stmt.execute([pattern.as_str()])?;
        }
    }

    // Replace the singleton row. Deleting then inserting keeps exactly one
    // row in the table.
    let dest_path = settings
        .rsync_dest_path
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned());
    tx.execute("DELETE FROM rsync_dest_path", [])?;
    tx.execute(
        "INSERT INTO rsync_dest_path (path) VALUES (?1)",
        [dest_path],
    )?;

    tx.commit()?;
    Ok(())
}

/// Append one selected source folder to the settings database.
///
/// The folder is stored as its own row in `silo_data_paths`. Existing rows
/// are kept; nothing is replaced. A path that already has a row is ignored,
/// so the same folder cannot be stored twice.
///
/// The store must be initialized with [`init`] first.
pub fn add_data_path(path: &Path) -> Result<(), ConfigError> {
    add_data_path_to(&default_db_path()?, path)
}

/// The same as [`add_data_path`], but writes to `db`.
fn add_data_path_to(db: &Path, path: &Path) -> Result<(), ConfigError> {
    let conn = Connection::open(db)?;
    conn.execute(
        "INSERT INTO silo_data_paths (path) VALUES (?1)
         ON CONFLICT (path) DO NOTHING",
        [path.to_string_lossy().into_owned()],
    )?;
    Ok(())
}

/// Remove one selected source folder from the settings database.
///
/// Deletes the folder's row from `silo_data_paths`. Other rows are kept. A
/// path that has no row is a no-op.
///
/// The store must be initialized with [`init`] first.
pub fn remove_data_path(path: &Path) -> Result<(), ConfigError> {
    remove_data_path_from(&default_db_path()?, path)
}

/// The same as [`remove_data_path`], but writes to `db`.
fn remove_data_path_from(db: &Path, path: &Path) -> Result<(), ConfigError> {
    let conn = Connection::open(db)?;
    conn.execute(
        "DELETE FROM silo_data_paths WHERE path = ?1",
        [path.to_string_lossy().into_owned()],
    )?;
    Ok(())
}

/// Replace all exclude patterns in the settings database.
///
/// Deletes every row in `exclude`, then inserts one row per non-empty
/// pattern, in order. Empty patterns are skipped.
///
/// The store must be initialized with [`init`] first.
pub fn replace_excludes(excludes: &[String]) -> Result<(), ConfigError> {
    replace_excludes_from(&default_db_path()?, excludes)
}

/// The same as [`replace_excludes`], but writes to `db`.
fn replace_excludes_from(db: &Path, excludes: &[String]) -> Result<(), ConfigError> {
    let mut conn = Connection::open(db)?;
    let tx = conn.transaction()?;

    tx.execute("DELETE FROM exclude", [])?;
    {
        let mut stmt = tx.prepare("INSERT INTO exclude (pattern) VALUES (?1)")?;
        for pattern in excludes.iter().filter(|pattern| !pattern.is_empty()) {
            stmt.execute([pattern.as_str()])?;
        }
    }

    tx.commit()?;
    Ok(())
}

/// Set the rsync destination path in the settings database.
///
/// Deletes the existing row in `rsync_dest_path`, then inserts the new path,
/// so the table always holds exactly one row. `None` clears the destination.
///
/// The store must be initialized with [`init`] first.
pub fn set_rsync_dest_path(path: Option<&Path>) -> Result<(), ConfigError> {
    set_rsync_dest_path_to(&default_db_path()?, path)
}

/// The same as [`set_rsync_dest_path`], but writes to `db`.
fn set_rsync_dest_path_to(db: &Path, path: Option<&Path>) -> Result<(), ConfigError> {
    let mut conn = Connection::open(db)?;
    let tx = conn.transaction()?;

    tx.execute("DELETE FROM rsync_dest_path", [])?;
    let value = path.map(|path| path.to_string_lossy().into_owned());
    tx.execute("INSERT INTO rsync_dest_path (path) VALUES (?1)", [value])?;

    tx.commit()?;
    Ok(())
}
