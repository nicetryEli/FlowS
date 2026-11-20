use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::database::{DatabaseError, DbResult};
use crate::models::{CalendarEvent, CalendarEventSource, CalendarEventUpdate, NewCalendarEvent};

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

fn row_to_event(row: &Row<'_>) -> rusqlite::Result<CalendarEvent> {
    let source_raw: String = row.get("source")?;
    let source = CalendarEventSource::parse(&source_raw).unwrap_or(CalendarEventSource::Personal);
    let all_day_int: i64 = row.get("all_day")?;
    Ok(CalendarEvent {
        id: row.get("id")?,
        title: row.get("title")?,
        description: row.get("description")?,
        start_at: row.get("start_at")?,
        end_at: row.get("end_at")?,
        all_day: all_day_int != 0,
        timezone: row.get("timezone")?,
        source,
        event_kind: row.get("event_kind")?,
        external_source: row.get("external_source")?,
        external_id: row.get("external_id")?,
        source_key: row.get("source_key")?,
        source_hash: row.get("source_hash")?,
        related_task_id: row.get("related_task_id")?,
        status: row.get("status")?,
        metadata_json: row.get("metadata_json")?,
        last_seen_at: row.get("last_seen_at")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn create_event(conn: &Connection, input: NewCalendarEvent) -> DbResult<CalendarEvent> {
    let title = input.title.trim();
    if title.is_empty() {
        return Err(DatabaseError::InvalidInput("event title cannot be empty".into()));
    }
    let now = now_rfc3339();
    conn.execute(
        "INSERT INTO calendar_events (
            title, description, start_at, end_at, all_day, timezone, source, event_kind,
            external_source, external_id, source_key, source_hash, related_task_id, status,
            metadata_json, last_seen_at, created_at, updated_at
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
        params![
            title,
            input.description,
            input.start_at,
            input.end_at,
            if input.all_day { 1 } else { 0 },
            input.timezone,
            input.source.as_str(),
            input.event_kind,
            input.external_source,
            input.external_id,
            input.source_key,
            input.source_hash,
            input.related_task_id,
            input.status,
            input.metadata_json,
            input.last_seen_at,
            now,
            now,
        ],
    )?;
    let id = conn.last_insert_rowid();
    get_event(conn, id)?.ok_or_else(|| DatabaseError::NotFound(format!("event {id} not found after insert")))
}

pub fn get_event(conn: &Connection, id: i64) -> DbResult<Option<CalendarEvent>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, description, start_at, end_at, all_day, timezone, source, event_kind,
                external_source, external_id, source_key, source_hash, related_task_id, status,
                metadata_json, last_seen_at, created_at, updated_at
         FROM calendar_events WHERE id = ?1",
    )?;
    Ok(stmt.query_row(params![id], row_to_event).optional()?)
}

pub fn list_events_in_range(
    conn: &Connection,
    start_at: &str,
    end_at: &str,
) -> DbResult<Vec<CalendarEvent>> {
    // Include events that overlap [start_at, end_at] (string compare works for ISO8601/RFC3339 in same format)
    let mut stmt = conn.prepare(
        "SELECT id, title, description, start_at, end_at, all_day, timezone, source, event_kind,
                external_source, external_id, source_key, source_hash, related_task_id, status,
                metadata_json, last_seen_at, created_at, updated_at
         FROM calendar_events
         WHERE (start_at IS NULL OR start_at <= ?2)
           AND (end_at IS NULL OR end_at >= ?1 OR (end_at IS NULL AND start_at >= ?1))
         ORDER BY COALESCE(start_at, end_at) ASC, id ASC",
    )?;
    let rows = stmt.query_map(params![start_at, end_at], row_to_event)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

pub fn list_events(conn: &Connection, source: Option<CalendarEventSource>) -> DbResult<Vec<CalendarEvent>> {
    let mut out = Vec::new();
    if let Some(source) = source {
        let mut stmt = conn.prepare(
            "SELECT id, title, description, start_at, end_at, all_day, timezone, source, event_kind,
                    external_source, external_id, source_key, source_hash, related_task_id, status,
                    metadata_json, last_seen_at, created_at, updated_at
             FROM calendar_events WHERE source = ?1
             ORDER BY COALESCE(start_at, end_at) ASC, id ASC",
        )?;
        let rows = stmt.query_map(params![source.as_str()], row_to_event)?;
        for row in rows { out.push(row?); }
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, title, description, start_at, end_at, all_day, timezone, source, event_kind,
                    external_source, external_id, source_key, source_hash, related_task_id, status,
                    metadata_json, last_seen_at, created_at, updated_at
             FROM calendar_events
             ORDER BY COALESCE(start_at, end_at) ASC, id ASC",
        )?;
        let rows = stmt.query_map([], row_to_event)?;
        for row in rows { out.push(row?); }
    }
    Ok(out)
}

pub fn update_event(conn: &Connection, id: i64, patch: CalendarEventUpdate) -> DbResult<CalendarEvent> {
    let existing = get_event(conn, id)?.ok_or_else(|| DatabaseError::NotFound(format!("event {id} not found")))?;
    let title = patch.title.unwrap_or(existing.title);
    if title.trim().is_empty() {
        return Err(DatabaseError::InvalidInput("event title cannot be empty".into()));
    }
    let description = match patch.description { Some(v) => v, None => existing.description };
    let start_at = match patch.start_at { Some(v) => v, None => existing.start_at };
    let end_at = match patch.end_at { Some(v) => v, None => existing.end_at };
    let all_day = patch.all_day.unwrap_or(existing.all_day);
    let timezone = match patch.timezone { Some(v) => v, None => existing.timezone };
    let source = patch.source.unwrap_or(existing.source);
    let event_kind = match patch.event_kind { Some(v) => v, None => existing.event_kind };
    let external_source = match patch.external_source { Some(v) => v, None => existing.external_source };
    let external_id = match patch.external_id { Some(v) => v, None => existing.external_id };
    let source_key = match patch.source_key { Some(v) => v, None => existing.source_key };
    let source_hash = match patch.source_hash { Some(v) => v, None => existing.source_hash };
    let related_task_id = match patch.related_task_id { Some(v) => v, None => existing.related_task_id };
    let status = match patch.status { Some(v) => v, None => existing.status };
    let metadata_json = match patch.metadata_json { Some(v) => v, None => existing.metadata_json };
    let last_seen_at = match patch.last_seen_at { Some(v) => v, None => existing.last_seen_at };
    let now = now_rfc3339();

    conn.execute(
        "UPDATE calendar_events SET
            title=?1, description=?2, start_at=?3, end_at=?4, all_day=?5, timezone=?6, source=?7, event_kind=?8,
            external_source=?9, external_id=?10, source_key=?11, source_hash=?12, related_task_id=?13, status=?14,
            metadata_json=?15, last_seen_at=?16, updated_at=?17
         WHERE id=?18",
        params![
            title.trim(), description, start_at, end_at, if all_day {1} else {0}, timezone, source.as_str(), event_kind,
            external_source, external_id, source_key, source_hash, related_task_id, status,
            metadata_json, last_seen_at, now, id
        ],
    )?;
    get_event(conn, id)?.ok_or_else(|| DatabaseError::NotFound(format!("event {id} not found after update")))
}

pub fn delete_event(conn: &Connection, id: i64) -> DbResult<()> {
    let changed = conn.execute("DELETE FROM calendar_events WHERE id = ?1", params![id])?;
    if changed == 0 {
        return Err(DatabaseError::NotFound(format!("event {id} not found")));
    }
    Ok(())
}

pub fn upsert_event_by_source_key(conn: &Connection, input: NewCalendarEvent) -> DbResult<CalendarEvent> {
    let source_key = input.source_key.clone().ok_or_else(|| {
        DatabaseError::InvalidInput("source_key required for upsert_event_by_source_key".into())
    })?;
    let existing_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM calendar_events WHERE source = ?1 AND source_key = ?2",
            params![input.source.as_str(), source_key],
            |row| row.get(0),
        )
        .optional()?;

    if let Some(id) = existing_id {
        update_event(
            conn,
            id,
            CalendarEventUpdate {
                title: Some(input.title),
                description: Some(input.description),
                start_at: Some(input.start_at),
                end_at: Some(input.end_at),
                all_day: Some(input.all_day),
                timezone: Some(input.timezone),
                source: Some(input.source),
                event_kind: Some(input.event_kind),
                external_source: Some(input.external_source),
                external_id: Some(input.external_id),
                source_key: Some(Some(source_key)),
                source_hash: Some(input.source_hash),
                related_task_id: Some(input.related_task_id),
                status: Some(input.status),
                metadata_json: Some(input.metadata_json),
                last_seen_at: Some(input.last_seen_at),
            },
        )
    } else {
        create_event(conn, input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::init_test_db;

    fn personal(title: &str, start: &str) -> NewCalendarEvent {
        NewCalendarEvent {
            title: title.into(),
            description: None,
            start_at: Some(start.into()),
            end_at: Some(start.into()),
            all_day: false,
            timezone: Some("Asia/Ho_Chi_Minh".into()),
            source: CalendarEventSource::Personal,
            event_kind: Some("personal".into()),
            external_source: None,
            external_id: None,
            source_key: None,
            source_hash: None,
            related_task_id: None,
            status: None,
            metadata_json: None,
            last_seen_at: None,
        }
    }

    #[test]
    fn create_list_mixed_sources_and_range_query() {
        let conn = init_test_db().unwrap();
        create_event(&conn, personal("Personal", "2026-11-20T10:00:00+07:00")).unwrap();
        create_event(&conn, NewCalendarEvent {
            source: CalendarEventSource::School,
            event_kind: Some("class_session".into()),
            source_key: Some("tdtu:class:1".into()),
            external_source: Some("tdtu".into()),
            ..personal("School", "2026-11-21T08:00:00+07:00")
        }).unwrap();
        create_event(&conn, NewCalendarEvent {
            source: CalendarEventSource::Deadline,
            event_kind: Some("deadline".into()),
            source_key: Some("tdtu:deadline:1".into()),
            external_source: Some("tdtu".into()),
            ..personal("Deadline", "2026-11-22T23:59:00+07:00")
        }).unwrap();

        let all = list_events(&conn, None).unwrap();
        assert_eq!(all.len(), 3);
        let school = list_events(&conn, Some(CalendarEventSource::School)).unwrap();
        assert_eq!(school.len(), 1);
        assert_eq!(school[0].source, CalendarEventSource::School);

        let range = list_events_in_range(&conn, "2026-11-20T00:00:00+07:00", "2026-11-21T23:59:59+07:00").unwrap();
        assert_eq!(range.len(), 2);
    }

    #[test]
    fn upsert_by_source_key_is_idempotent() {
        let conn = init_test_db().unwrap();
        let e1 = upsert_event_by_source_key(&conn, NewCalendarEvent {
            source: CalendarEventSource::School,
            source_key: Some("tdtu:exam:1".into()),
            source_hash: Some("h1".into()),
            ..personal("Exam", "2026-11-25T09:00:00+07:00")
        }).unwrap();
        let e2 = upsert_event_by_source_key(&conn, NewCalendarEvent {
            title: "Exam updated".into(),
            source: CalendarEventSource::School,
            source_key: Some("tdtu:exam:1".into()),
            source_hash: Some("h2".into()),
            ..personal("Exam updated", "2026-11-25T10:00:00+07:00")
        }).unwrap();
        assert_eq!(e1.id, e2.id);
        assert_eq!(e2.title, "Exam updated");
        assert_eq!(e2.source_hash.as_deref(), Some("h2"));

        let all = list_events(&conn, None).unwrap();
        assert_eq!(all.len(), 1, "repeated upsert must not duplicate");
    }
}
