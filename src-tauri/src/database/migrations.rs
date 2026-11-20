use rusqlite::Connection;

use crate::database::{DatabaseError, DbResult};

struct Migration {
    version: i32,
    name: &'static str,
    sql: &'static str,
}

fn migrations() -> &'static [Migration] {
    &[Migration {
        version: 1,
        name: "0001_initial_schema",
        sql: include_str!("../../migrations/0001_initial_schema.sql"),
    }]
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
            DatabaseError::Migration(format!(
                "failed migration {} ({}): {e}",
                migration.version, migration.name
            ))
        })?;
        tx.pragma_update(None, "user_version", migration.version)?;
        tx.commit()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::apply_pending_migrations;

    #[test]
    fn applies_initial_schema_migration() {
        let mut conn = Connection::open_in_memory().expect("in-memory db should open");
        apply_pending_migrations(&mut conn).expect("migrations should apply");

        let user_version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("user_version should be readable");
        assert_eq!(user_version, 1);

        for table in [
            "tasks",
            "tags",
            "task_tags",
            "completions",
            "task_focus_state",
            "calendar_events",
            "sync_state",
            "sync_runs",
        ] {
            let exists: i32 = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name = ?1)",
                    [table],
                    |row| row.get(0),
                )
                .unwrap_or_else(|_| panic!("failed to check table {table}"));
            assert_eq!(exists, 1, "expected table {table} to exist");
        }
    }
}

