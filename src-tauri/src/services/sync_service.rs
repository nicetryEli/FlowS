use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::database::DbResult;
use crate::models::NewTask;
use crate::services::tdtu_sync_adapter::{map_dataset, TdtuDataset};
use crate::services::{calendar_service, task_service};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncWindow {
    pub start_at: String,
    pub end_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncRunResult {
    pub source: String,
    pub status: String,
    pub records_seen: i64,
    pub events_upserted: i64,
    pub tasks_upserted: i64,
    pub window_start_at: String,
    pub window_end_at: String,
    pub error_message: Option<String>,
}

pub fn compute_sync_window(now: DateTime<Utc>, weeks_ahead: i64) -> SyncWindow {
    let start_at = now.to_rfc3339();
    let end_at = (now + Duration::weeks(weeks_ahead)).to_rfc3339();
    SyncWindow { start_at, end_at }
}

pub fn ensure_sync_state_row(conn: &Connection, source: &str) -> DbResult<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR IGNORE INTO sync_state (source, enabled, updated_at) VALUES (?1, 1, ?2)",
        params![source, now],
    )?;
    Ok(())
}

pub fn record_sync_attempt(conn: &Connection, source: &str, window: &SyncWindow) -> DbResult<i64> {
    ensure_sync_state_row(conn, source)?;
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO sync_runs (source, status, started_at, window_start_at, window_end_at)
         VALUES (?1, 'running', ?2, ?3, ?4)",
        params![source, now, window.start_at, window.end_at],
    )?;
    let run_id = conn.last_insert_rowid();
    conn.execute(
        "UPDATE sync_state SET last_attempt_at = ?1, window_start_at = ?2, window_end_at = ?3, updated_at = ?4 WHERE source = ?5",
        params![now, window.start_at, window.end_at, now, source],
    )?;
    Ok(run_id)
}

pub fn finalize_sync_run(
    conn: &Connection,
    run_id: i64,
    source: &str,
    status: &str,
    records_seen: i64,
    records_upserted: i64,
    error_message: Option<String>,
    next_run_at: Option<String>,
) -> DbResult<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE sync_runs SET status = ?1, finished_at = ?2, records_seen = ?3, records_upserted = ?4, error_message = ?5
         WHERE id = ?6",
        params![status, now, records_seen, records_upserted, error_message, run_id],
    )?;

    if status == "success" || status == "partial" {
        let last_error: Option<String> = if status == "partial" {
            error_message.clone().or_else(|| Some("partial success".to_string()))
        } else {
            None
        };
        conn.execute(
            "UPDATE sync_state SET last_success_at = ?1, last_error = ?2, next_run_at = COALESCE(?3, next_run_at), updated_at = ?4 WHERE source = ?5",
            params![now, last_error, next_run_at, now, source],
        )?;
    } else {
        conn.execute(
            "UPDATE sync_state SET last_error = ?1, next_run_at = COALESCE(?2, next_run_at), updated_at = ?3 WHERE source = ?4",
            params![error_message.unwrap_or_else(|| "sync failed".into()), next_run_at, now, source],
        )?;
    }
    Ok(())
}

fn upsert_school_task(conn: &Connection, input: NewTask) -> DbResult<i64> {
    let source = input.source.as_str();
    let source_key = input.source_key.clone().ok_or_else(|| {
        crate::database::DatabaseError::InvalidInput("school task source_key required for upsert".into())
    })?;
    let existing_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM tasks WHERE source = ?1 AND source_key = ?2",
            params![source, source_key],
            |row| row.get(0),
        )
        .optional()?;

    if let Some(id) = existing_id {
        // update mutable fields; preserve completion/focus state
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE tasks SET
                title = ?1, description = ?2, deadline_at = ?3, importance = ?4,
                external_source = ?5, external_id = ?6, source_hash = ?7, last_seen_at = ?8, updated_at = ?9
             WHERE id = ?10",
            params![
                input.title,
                input.description,
                input.deadline_at,
                input.importance.as_str(),
                input.external_source,
                input.external_id,
                input.source_hash,
                input.last_seen_at,
                now,
                id
            ],
        )?;
        Ok(id)
    } else {
        let task = task_service::create_task(conn, input)?;
        Ok(task.id)
    }
}

pub fn apply_tdtu_dataset(conn: &Connection, dataset: &TdtuDataset) -> DbResult<(i64, i64)> {
    let mapped = map_dataset(dataset);
    let mut events_upserted = 0i64;
    let mut tasks_upserted = 0i64;

    // Upsert events (idempotent by source+source_key)
    for event in &mapped.calendar_events {
        if event.source_key.is_some() {
            calendar_service::upsert_event_by_source_key(conn, event.clone())?;
        } else {
            calendar_service::create_event(conn, event.clone())?;
        }
        events_upserted += 1;
    }

    // Upsert school task candidates (deadlines) without deleting existing data on partial sync
    for task in &mapped.task_candidates {
        upsert_school_task(conn, task.clone())?;
        tasks_upserted += 1;
    }

    Ok((events_upserted, tasks_upserted))
}

pub fn run_tdtu_sync_with_dataset(
    conn: &Connection,
    source: &str,
    dataset: &TdtuDataset,
    now: DateTime<Utc>,
) -> DbResult<SyncRunResult> {
    let window = compute_sync_window(now, 4);
    let run_id = record_sync_attempt(conn, source, &window)?;

    let records_seen = (dataset.schedule.len() + dataset.exams.len() + dataset.deadlines.len()) as i64;
    match apply_tdtu_dataset(conn, dataset) {
        Ok((events_upserted, tasks_upserted)) => {
            let next_run = (now + Duration::hours(5)).to_rfc3339();
            let upserted_total = events_upserted + tasks_upserted;
            finalize_sync_run(conn, run_id, source, "success", records_seen, upserted_total, None, Some(next_run.clone()))?;
            Ok(SyncRunResult {
                source: source.to_string(),
                status: "success".into(),
                records_seen,
                events_upserted,
                tasks_upserted,
                window_start_at: window.start_at,
                window_end_at: window.end_at,
                error_message: None,
            })
        }
        Err(e) => {
            let next_run = (now + Duration::hours(5)).to_rfc3339();
            let msg = e.to_string();
            finalize_sync_run(conn, run_id, source, "failed", records_seen, 0, Some(msg.clone()), Some(next_run))?;
            Err(e)
        }
    }
}

pub fn get_sync_state(conn: &Connection, source: &str) -> DbResult<Option<serde_json::Value>> {
    ensure_sync_state_row(conn, source)?;
    let row = conn
        .query_row(
            "SELECT source, enabled, last_attempt_at, last_success_at, last_error, next_run_at, window_start_at, window_end_at, updated_at
             FROM sync_state WHERE source = ?1",
            params![source],
            |row| {
                Ok(serde_json::json!({
                    "source": row.get::<_, String>(0)?,
                    "enabled": row.get::<_, i64>(1)? != 0,
                    "last_attempt_at": row.get::<_, Option<String>>(2)?,
                    "last_success_at": row.get::<_, Option<String>>(3)?,
                    "last_error": row.get::<_, Option<String>>(4)?,
                    "next_run_at": row.get::<_, Option<String>>(5)?,
                    "window_start_at": row.get::<_, Option<String>>(6)?,
                    "window_end_at": row.get::<_, Option<String>>(7)?,
                    "updated_at": row.get::<_, String>(8)?,
                }))
            },
        )
        .optional()?;
    Ok(row)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::init_test_db;
    use crate::services::tdtu_sync_adapter::{TdtuDataset, TdtuDeadline, TdtuExam, TdtuScheduleSession};

    fn sample_dataset() -> TdtuDataset {
        TdtuDataset {
            schedule: vec![TdtuScheduleSession {
                student_id: Some("1".into()),
                subject_name: "Math".into(),
                room: "A1".into(),
                day_of_week: Some("Monday".into()),
                session_date: "2026-11-23".into(),
                start_period: 1,
                end_period: 2,
                status: "scheduled".into(),
            }],
            exams: vec![TdtuExam {
                subject_name: "Math".into(),
                exam_date: "2026-12-01".into(),
                start_time: "08:00".into(),
                end_time: "".into(),
                exam_room: "B2".into(),
                exam_type: Some("Final".into()),
                notes: None,
            }],
            deadlines: vec![TdtuDeadline {
                course_id: "c1".into(),
                course_name: "Math".into(),
                activity_name: "HW1".into(),
                activity_url: "https://elearning.tdtu.edu.vn/mod/assign/view.php?id=99".into(),
                due_date: "2026-11-30T23:59:00+07:00".into(),
                completion_status: "incomplete".into(),
            }],
        }
    }

    #[test]
    fn window_is_four_weeks_ahead() {
        let now = DateTime::parse_from_rfc3339("2026-11-20T10:00:00Z").unwrap().with_timezone(&Utc);
        let w = compute_sync_window(now, 4);
        assert_eq!(w.start_at, "2026-11-20T10:00:00+00:00");
        assert!(w.end_at.starts_with("2026-12-18T10:00:00"));
    }

    #[test]
    fn apply_dataset_is_idempotent_for_events_and_tasks() {
        let conn = init_test_db().unwrap();
        let ds = sample_dataset();
        let (e1, t1) = apply_tdtu_dataset(&conn, &ds).unwrap();
        let (e2, t2) = apply_tdtu_dataset(&conn, &ds).unwrap();
        assert_eq!((e1, t1), (e2, t2));

        let event_count: i64 = conn.query_row("SELECT COUNT(*) FROM calendar_events", [], |r| r.get(0)).unwrap();
        let task_count: i64 = conn.query_row("SELECT COUNT(*) FROM tasks WHERE source='school'", [], |r| r.get(0)).unwrap();
        assert_eq!(event_count, 3); // schedule + exam + deadline
        assert_eq!(task_count, 1);  // deadline task
    }

    #[test]
    fn run_records_sync_state_and_runs() {
        let conn = init_test_db().unwrap();
        let ds = sample_dataset();
        let now = DateTime::parse_from_rfc3339("2026-11-20T10:00:00Z").unwrap().with_timezone(&Utc);
        let res = run_tdtu_sync_with_dataset(&conn, "tdtu", &ds, now).unwrap();
        assert_eq!(res.status, "success");
        assert_eq!(res.records_seen, 3);

        let state = get_sync_state(&conn, "tdtu").unwrap().unwrap();
        assert_eq!(state["source"], "tdtu");
        assert!(state["last_success_at"].as_str().is_some());
        assert!(state["next_run_at"].as_str().is_some());

        let run_count: i64 = conn.query_row("SELECT COUNT(*) FROM sync_runs WHERE source='tdtu'", [], |r| r.get(0)).unwrap();
        assert_eq!(run_count, 1);
    }
}
