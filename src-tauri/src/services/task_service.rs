use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::database::{DatabaseError, DbResult};
use crate::models::{NewTask, Task, TaskImportance, TaskSource, TaskUpdate};

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

// Parsing helpers will be used by command/input mapping in later tasks.

fn row_to_task(row: &Row<'_>) -> rusqlite::Result<Task> {
    let importance_raw: String = row.get("importance")?;
    let source_raw: String = row.get("source")?;
    let importance = TaskImportance::parse(&importance_raw).unwrap_or(TaskImportance::Mid);
    let source = TaskSource::parse(&source_raw).unwrap_or(TaskSource::Personal);
    let is_focused_int: i64 = row.get("is_focused")?;
    Ok(Task {
        id: row.get("id")?,
        title: row.get("title")?,
        description: row.get("description")?,
        created_at: row.get("created_at")?,
        deadline_at: row.get("deadline_at")?,
        importance,
        source,
        external_source: row.get("external_source")?,
        external_id: row.get("external_id")?,
        source_key: row.get("source_key")?,
        source_hash: row.get("source_hash")?,
        completed_at: row.get("completed_at")?,
        is_focused: is_focused_int != 0,
        focus_order: row.get("focus_order")?,
        updated_at: row.get("updated_at")?,
        last_seen_at: row.get("last_seen_at")?,
    })
}

pub fn create_task(conn: &Connection, input: NewTask) -> DbResult<Task> {
    let title = input.title.trim();
    if title.is_empty() {
        return Err(DatabaseError::InvalidInput("title cannot be empty".into()));
    }

    let now = now_rfc3339();
    conn.execute(
        "INSERT INTO tasks (
            title, description, created_at, deadline_at, importance, source,
            external_source, external_id, source_key, source_hash,
            completed_at, is_focused, focus_order, created_source, updated_at, last_seen_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, NULL, 0, NULL, 'local', ?11, ?12)",
        params![
            title,
            input.description,
            now,
            input.deadline_at,
            input.importance.as_str(),
            input.source.as_str(),
            input.external_source,
            input.external_id,
            input.source_key,
            input.source_hash,
            now,
            input.last_seen_at,
        ],
    )?;
    let id = conn.last_insert_rowid();
    get_task(conn, id)?.ok_or_else(|| DatabaseError::NotFound(format!("task {id} not found after insert")))
}

pub fn get_task(conn: &Connection, id: i64) -> DbResult<Option<Task>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, description, created_at, deadline_at, importance, source,
                external_source, external_id, source_key, source_hash,
                completed_at, is_focused, focus_order, updated_at, last_seen_at
         FROM tasks WHERE id = ?1",
    )?;
    let task = stmt.query_row(params![id], row_to_task).optional()?;
    Ok(task)
}

pub fn list_tasks(conn: &Connection) -> DbResult<Vec<Task>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, description, created_at, deadline_at, importance, source,
                external_source, external_id, source_key, source_hash,
                completed_at, is_focused, focus_order, updated_at, last_seen_at
         FROM tasks
         ORDER BY created_at DESC, id DESC",
    )?;
    let rows = stmt.query_map([], row_to_task)?;
    let mut tasks = Vec::new();
    for row in rows {
        tasks.push(row?);
    }
    Ok(tasks)
}

pub fn update_task(conn: &Connection, id: i64, patch: TaskUpdate) -> DbResult<Task> {
    let existing = get_task(conn, id)?.ok_or_else(|| DatabaseError::NotFound(format!("task {id} not found")))?;
    let title = patch.title.unwrap_or(existing.title);
    if title.trim().is_empty() {
        return Err(DatabaseError::InvalidInput("title cannot be empty".into()));
    }
    let description = match patch.description {
        Some(v) => v,
        None => existing.description,
    };
    let deadline_at = match patch.deadline_at {
        Some(v) => v,
        None => existing.deadline_at,
    };
    let importance = patch.importance.unwrap_or(existing.importance);
    let source = patch.source.unwrap_or(existing.source);
    let external_source = match patch.external_source { Some(v) => v, None => existing.external_source };
    let external_id = match patch.external_id { Some(v) => v, None => existing.external_id };
    let source_key = match patch.source_key { Some(v) => v, None => existing.source_key };
    let source_hash = match patch.source_hash { Some(v) => v, None => existing.source_hash };
    let last_seen_at = match patch.last_seen_at { Some(v) => v, None => existing.last_seen_at };
    let now = now_rfc3339();

    conn.execute(
        "UPDATE tasks SET
            title = ?1, description = ?2, deadline_at = ?3, importance = ?4, source = ?5,
            external_source = ?6, external_id = ?7, source_key = ?8, source_hash = ?9,
            last_seen_at = ?10, updated_at = ?11
         WHERE id = ?12",
        params![
            title.trim(),
            description,
            deadline_at,
            importance.as_str(),
            source.as_str(),
            external_source,
            external_id,
            source_key,
            source_hash,
            last_seen_at,
            now,
            id
        ],
    )?;
    get_task(conn, id)?.ok_or_else(|| DatabaseError::NotFound(format!("task {id} not found after update")))
}

pub fn delete_task(conn: &Connection, id: i64) -> DbResult<()> {
    let changed = conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])?;
    if changed == 0 {
        return Err(DatabaseError::NotFound(format!("task {id} not found")));
    }
    Ok(())
}

pub fn complete_task(conn: &Connection, id: i64, completed_at: Option<String>) -> DbResult<Task> {
    let mut task = get_task(conn, id)?.ok_or_else(|| DatabaseError::NotFound(format!("task {id} not found")))?;
    let ts = completed_at.unwrap_or_else(now_rfc3339);
    let completion_date = completion_date_from_timestamp(&ts);

    let tx = conn.unchecked_transaction()?;
    if task.completed_at.is_none() {
        tx.execute(
            "UPDATE tasks SET completed_at = ?1, updated_at = ?2 WHERE id = ?3",
            params![ts, now_rfc3339(), id],
        )?;
        task.completed_at = Some(ts.clone());
    }
    tx.execute(
        "INSERT INTO completions (task_id, completed_at, completion_date, source) VALUES (?1, ?2, ?3, 'local')",
        params![id, ts, completion_date],
    )?;
    tx.commit()?;

    get_task(conn, id)?.ok_or_else(|| DatabaseError::NotFound(format!("task {id} not found after complete")))
}

pub fn set_task_focus(conn: &Connection, id: i64, focused: bool) -> DbResult<Task> {
    let _task = get_task(conn, id)?.ok_or_else(|| DatabaseError::NotFound(format!("task {id} not found")))?;
    let now = now_rfc3339();
    let tx = conn.unchecked_transaction()?;

    if focused {
        let focused_count: i64 = tx.query_row(
            "SELECT COUNT(*) FROM tasks WHERE is_focused = 1 AND id <> ?1",
            params![id],
            |row| row.get(0),
        )?;
        if focused_count >= 3 {
            return Err(DatabaseError::Conflict("maximum of 3 focused tasks allowed".into()));
        }
        let next_order: i64 = tx.query_row(
            "SELECT COALESCE(MAX(focus_order), 0) + 1 FROM tasks WHERE is_focused = 1 AND id <> ?1",
            params![id],
            |row| row.get(0),
        )?;
        tx.execute(
            "UPDATE tasks SET is_focused = 1, focus_order = ?1, updated_at = ?2 WHERE id = ?3",
            params![next_order, now, id],
        )?;
        tx.execute(
            "INSERT INTO task_focus_state (task_id, is_focused, focus_order, updated_at)
             VALUES (?1, 1, ?2, ?3)
             ON CONFLICT(task_id) DO UPDATE SET
               is_focused = excluded.is_focused,
               focus_order = excluded.focus_order,
               updated_at = excluded.updated_at",
            params![id, next_order, now],
        )?;
    } else {
        tx.execute(
            "UPDATE tasks SET is_focused = 0, focus_order = NULL, updated_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        tx.execute(
            "INSERT INTO task_focus_state (task_id, is_focused, focus_order, updated_at)
             VALUES (?1, 0, NULL, ?2)
             ON CONFLICT(task_id) DO UPDATE SET
               is_focused = excluded.is_focused,
               focus_order = excluded.focus_order,
               updated_at = excluded.updated_at",
            params![id, now],
        )?;
        // compact focus order for remaining focused tasks
        compact_focus_order_tx(&tx, &now)?;
    }

    tx.commit()?;
    get_task(conn, id)?.ok_or_else(|| DatabaseError::NotFound(format!("task {id} not found after focus update")))
}

fn compact_focus_order_tx(tx: &rusqlite::Transaction<'_>, now: &str) -> DbResult<()> {
    let mut stmt = tx.prepare("SELECT id FROM tasks WHERE is_focused = 1 ORDER BY focus_order ASC, id ASC")?;
    let ids = stmt
        .query_map([], |row| row.get::<_, i64>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for (idx, task_id) in ids.iter().enumerate() {
        let order = (idx as i64) + 1;
        tx.execute(
            "UPDATE tasks SET focus_order = ?1, updated_at = ?2 WHERE id = ?3",
            params![order, now, task_id],
        )?;
        tx.execute(
            "INSERT INTO task_focus_state (task_id, is_focused, focus_order, updated_at)
             VALUES (?1, 1, ?2, ?3)
             ON CONFLICT(task_id) DO UPDATE SET
               is_focused = excluded.is_focused,
               focus_order = excluded.focus_order,
               updated_at = excluded.updated_at",
            params![task_id, order, now],
        )?;
    }
    Ok(())
}

pub fn list_focused_tasks(conn: &Connection) -> DbResult<Vec<Task>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, description, created_at, deadline_at, importance, source,
                external_source, external_id, source_key, source_hash,
                completed_at, is_focused, focus_order, updated_at, last_seen_at
         FROM tasks
         WHERE is_focused = 1
         ORDER BY focus_order ASC, id ASC",
    )?;
    let rows = stmt.query_map([], row_to_task)?;
    let mut tasks = Vec::new();
    for row in rows {
        tasks.push(row?);
    }
    Ok(tasks)
}

fn completion_date_from_timestamp(ts: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(ts) {
        return dt.date_naive().to_string();
    }
    // fallback: take first 10 chars if ISO-like
    ts.chars().take(10).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::init_test_db;

    fn sample_new_task(title: &str) -> NewTask {
        NewTask {
            title: title.to_string(),
            description: None,
            deadline_at: None,
            importance: TaskImportance::Mid,
            source: TaskSource::Personal,
            external_source: None,
            external_id: None,
            source_key: None,
            source_hash: None,
            last_seen_at: None,
        }
    }

    #[test]
    fn create_and_list_tasks_allows_no_deadline_and_school_source() {
        let conn = init_test_db().expect("test db");
        let personal = create_task(&conn, sample_new_task("Personal task")).expect("create personal");
        assert!(personal.deadline_at.is_none(), "no-deadline task must be allowed");

        let school = create_task(
            &conn,
            NewTask {
                source: TaskSource::School,
                external_source: Some("tdtu".into()),
                external_id: Some("course-1".into()),
                source_key: Some("tdtu:course-1:2026-11-20".into()),
                ..sample_new_task("School task")
            },
        )
        .expect("create school task");
        assert_eq!(school.source, TaskSource::School);

        let tasks = list_tasks(&conn).expect("list tasks");
        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn complete_task_records_completion_history() {
        let conn = init_test_db().expect("test db");
        let task = create_task(&conn, sample_new_task("Do homework")).expect("create");
        let completed = complete_task(&conn, task.id, Some("2026-11-20T10:00:00Z".into())).expect("complete");
        assert!(completed.completed_at.is_some());

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM completions WHERE task_id = ?1",
                params![task.id],
                |row| row.get(0),
            )
            .expect("count completions");
        assert_eq!(count, 1, "completion history should be recorded");
    }

    #[test]
    fn focus_state_persists_and_enforces_max_three() {
        let conn = init_test_db().expect("test db");
        let t1 = create_task(&conn, sample_new_task("T1")).unwrap();
        let t2 = create_task(&conn, sample_new_task("T2")).unwrap();
        let t3 = create_task(&conn, sample_new_task("T3")).unwrap();
        let t4 = create_task(&conn, sample_new_task("T4")).unwrap();

        set_task_focus(&conn, t1.id, true).unwrap();
        set_task_focus(&conn, t2.id, true).unwrap();
        set_task_focus(&conn, t3.id, true).unwrap();
        let err = set_task_focus(&conn, t4.id, true).expect_err("4th focus should fail");
        match err {
            DatabaseError::Conflict(_) => {}
            other => panic!("expected conflict, got {other:?}"),
        }

        let focused = list_focused_tasks(&conn).unwrap();
        assert_eq!(focused.len(), 3);
        assert_eq!(focused[0].id, t1.id);

        set_task_focus(&conn, t2.id, false).unwrap();
        let focused_after = list_focused_tasks(&conn).unwrap();
        assert_eq!(focused_after.len(), 2);
        assert_eq!(focused_after[0].id, t1.id);
        assert_eq!(focused_after[1].id, t3.id);
    }

    #[test]
    fn focus_state_persists_across_reopened_database() {
        use crate::database::init_db;
        use std::fs;

        let temp_root = std::env::temp_dir().join(format!("flows-task-reopen-{}", std::process::id()));
        let db_path = temp_root.join("flows.sqlite");
        let _ = fs::remove_dir_all(&temp_root);

        let conn = init_db(&db_path).expect("init db");
        let t1 = create_task(&conn, sample_new_task("T1")).unwrap();
        let t2 = create_task(&conn, sample_new_task("T2")).unwrap();
        set_task_focus(&conn, t1.id, true).unwrap();
        set_task_focus(&conn, t2.id, true).unwrap();
        drop(conn);

        let reopened = crate::database::open_connection(&db_path).expect("reopen db");
        let focused = list_focused_tasks(&reopened).unwrap();
        assert_eq!(focused.len(), 2);
        assert_eq!(focused[0].id, t1.id);
        assert_eq!(focused[1].id, t2.id);

        drop(reopened);
        let _ = fs::remove_dir_all(&temp_root);
    }
}
