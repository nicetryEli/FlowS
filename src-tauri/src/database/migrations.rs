use rusqlite::Connection;

use crate::database::{DatabaseError, DbResult};

struct Migration {
    version: i32,
    name: &'static str,
    sql: &'static str,
}

fn migrations() -> &'static [Migration] {
    // Task 3 will add the initial domain schema migration.
    &[]
}

pub fn apply_pending_migrations(conn: &mut Connection) -> DbResult<()> {
    let current_version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

    for migration in migrations() {
        if migration.version <= current_version {
            continue;
        }
        if migration.version != current_version + 1 {
            return Err(DatabaseError::Migration(format!(
                "non-sequential migration: expected {}, found {} ({})",
                current_version + 1,
                migration.version,
                migration.name
            )));
        }

        let tx = conn.transaction()?;
        tx.execute_batch(migration.sql).map_err(|e| {
            DatabaseError::Migration(format!("failed migration {} ({}): {e}", migration.version, migration.name))
        })?;
        tx.pragma_update(None, "user_version", migration.version)?;
        tx.commit()?;
    }

    Ok(())
}
