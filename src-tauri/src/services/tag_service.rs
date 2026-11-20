use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::database::{DatabaseError, DbResult};
use crate::models::{NewTag, Tag, TagUpdate};

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

fn row_to_tag(row: &Row<'_>) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get("id")?,
        name: row.get("name")?,
        color: row.get("color")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn create_tag(conn: &Connection, input: NewTag) -> DbResult<Tag> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(DatabaseError::InvalidInput("tag name cannot be empty".into()));
    }
    let now = now_rfc3339();
    conn.execute(
        "INSERT INTO tags (name, color, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        params![name, input.color, now, now],
    )?;
    let id = conn.last_insert_rowid();
    get_tag(conn, id)?.ok_or_else(|| DatabaseError::NotFound(format!("tag {id} not found after insert")))
}

pub fn get_tag(conn: &Connection, id: i64) -> DbResult<Option<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, color, created_at, updated_at FROM tags WHERE id = ?1",
    )?;
    let tag = stmt.query_row(params![id], row_to_tag).optional()?;
    Ok(tag)
}

pub fn list_tags(conn: &Connection) -> DbResult<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, color, created_at, updated_at FROM tags ORDER BY name ASC, id ASC",
    )?;
    let rows = stmt.query_map([], row_to_tag)?;
    let mut tags = Vec::new();
    for row in rows {
        tags.push(row?);
    }
    Ok(tags)
}

pub fn update_tag(conn: &Connection, id: i64, patch: TagUpdate) -> DbResult<Tag> {
    let existing = get_tag(conn, id)?.ok_or_else(|| DatabaseError::NotFound(format!("tag {id} not found")))?;
    let name = patch.name.unwrap_or(existing.name);
    if name.trim().is_empty() {
        return Err(DatabaseError::InvalidInput("tag name cannot be empty".into()));
    }
    let color = match patch.color {
        Some(v) => v,
        None => existing.color,
    };
    let now = now_rfc3339();
    conn.execute(
        "UPDATE tags SET name = ?1, color = ?2, updated_at = ?3 WHERE id = ?4",
        params![name.trim(), color, now, id],
    )?;
    get_tag(conn, id)?.ok_or_else(|| DatabaseError::NotFound(format!("tag {id} not found after update")))
}

pub fn delete_tag(conn: &Connection, id: i64) -> DbResult<()> {
    let changed = conn.execute("DELETE FROM tags WHERE id = ?1", params![id])?;
    if changed == 0 {
        return Err(DatabaseError::NotFound(format!("tag {id} not found")));
    }
    Ok(())
}

pub fn attach_tag_to_task(conn: &Connection, task_id: i64, tag_id: i64) -> DbResult<()> {
    ensure_task_exists(conn, task_id)?;
    ensure_tag_exists(conn, tag_id)?;
    let now = now_rfc3339();
    conn.execute(
        "INSERT OR IGNORE INTO task_tags (task_id, tag_id, created_at) VALUES (?1, ?2, ?3)",
        params![task_id, tag_id, now],
    )?;
    Ok(())
}

pub fn detach_tag_from_task(conn: &Connection, task_id: i64, tag_id: i64) -> DbResult<()> {
    conn.execute(
        "DELETE FROM task_tags WHERE task_id = ?1 AND tag_id = ?2",
        params![task_id, tag_id],
    )?;
    Ok(())
}

pub fn list_tags_for_task(conn: &Connection, task_id: i64) -> DbResult<Vec<Tag>> {
    ensure_task_exists(conn, task_id)?;
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color, t.created_at, t.updated_at
         FROM tags t
         INNER JOIN task_tags tt ON tt.tag_id = t.id
         WHERE tt.task_id = ?1
         ORDER BY t.name ASC, t.id ASC",
    )?;
    let rows = stmt.query_map(params![task_id], row_to_tag)?;
    let mut tags = Vec::new();
    for row in rows {
        tags.push(row?);
    }
    Ok(tags)
}

fn ensure_task_exists(conn: &Connection, task_id: i64) -> DbResult<()> {
    let exists: i64 = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM tasks WHERE id = ?1)",
        params![task_id],
        |row| row.get(0),
    )?;
    if exists == 0 {
        return Err(DatabaseError::NotFound(format!("task {task_id} not found")));
    }
    Ok(())
}

fn ensure_tag_exists(conn: &Connection, tag_id: i64) -> DbResult<()> {
    let exists: i64 = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM tags WHERE id = ?1)",
        params![tag_id],
        |row| row.get(0),
    )?;
    if exists == 0 {
        return Err(DatabaseError::NotFound(format!("tag {tag_id} not found")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::init_test_db;
    use crate::models::{NewTag, NewTask, TagUpdate, TaskImportance, TaskSource};
    use crate::services::task_service;

    fn sample_task(title: &str) -> NewTask {
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
    fn tag_crud_works() {
        let conn = init_test_db().expect("test db");
        let tag = create_tag(&conn, NewTag { name: "School".into(), color: Some("#00ff00".into()) }).unwrap();
        assert_eq!(tag.name, "School");

        let fetched = get_tag(&conn, tag.id).unwrap().expect("tag exists");
        assert_eq!(fetched.id, tag.id);

        let updated = update_tag(
            &conn,
            tag.id,
            TagUpdate { name: Some("Study".into()), color: Some(None) },
        )
        .unwrap();
        assert_eq!(updated.name, "Study");
        assert_eq!(updated.color, None);

        let tags = list_tags(&conn).unwrap();
        assert_eq!(tags.len(), 1);

        delete_tag(&conn, tag.id).unwrap();
        assert!(get_tag(&conn, tag.id).unwrap().is_none());
    }

    #[test]
    fn task_can_have_multiple_tags_and_detach() {
        let conn = init_test_db().expect("test db");
        let task = task_service::create_task(&conn, sample_task("Task A")).unwrap();
        let t1 = create_tag(&conn, NewTag { name: "A".into(), color: None }).unwrap();
        let t2 = create_tag(&conn, NewTag { name: "B".into(), color: None }).unwrap();

        attach_tag_to_task(&conn, task.id, t1.id).unwrap();
        attach_tag_to_task(&conn, task.id, t2.id).unwrap();
        attach_tag_to_task(&conn, task.id, t2.id).unwrap(); // idempotent

        let tags = list_tags_for_task(&conn, task.id).unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].name, "A");
        assert_eq!(tags[1].name, "B");

        detach_tag_from_task(&conn, task.id, t1.id).unwrap();
        let tags_after = list_tags_for_task(&conn, task.id).unwrap();
        assert_eq!(tags_after.len(), 1);
        assert_eq!(tags_after[0].name, "B");
    }
}
