pub mod migrations;

use rusqlite::Connection;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("migration error: {0}")]
    Migration(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("conflict: {0}")]
    Conflict(String),
}

pub type DbResult<T> = std::result::Result<T, DatabaseError>;

#[derive(Clone, Debug)]
pub struct DbPath(pub PathBuf);

/// Open a SQLite connection to `db_path` with required pragmas.
/// Assumes migrations have already been applied for the app database.
pub fn open_connection<P: AsRef<Path>>(db_path: P) -> DbResult<Connection> {
    let conn = Connection::open(db_path.as_ref())?;
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;
         PRAGMA busy_timeout = 5000;",
    )?;
    Ok(conn)
}

/// Initialize/open the SQLite database at `db_path` and apply pending migrations.
pub fn init_db<P: AsRef<Path>>(db_path: P) -> DbResult<Connection> {
    let db_path = db_path.as_ref();
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut conn = open_connection(db_path)?;
    migrations::apply_pending_migrations(&mut conn)?;
    Ok(conn)
}

#[cfg(test)]
pub fn init_test_db() -> DbResult<Connection> {
    let mut conn = Connection::open_in_memory()?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    migrations::apply_pending_migrations(&mut conn)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn init_db_creates_database_and_applies_migrations() {
        let temp_root =
            std::env::temp_dir().join(format!("flows-db-test-{}", std::process::id()));
        let db_path = temp_root.join("nested").join("flows.sqlite");

        let _ = fs::remove_dir_all(&temp_root);
        let conn = init_db(&db_path).expect("db should initialize");

        assert!(db_path.exists(), "database file should be created");

        let user_version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("user_version should be readable");
        assert!(
            user_version >= 1,
            "user_version should be set after migration runner executes"
        );

        drop(conn);
        let _ = fs::remove_dir_all(&temp_root);
    }
}
