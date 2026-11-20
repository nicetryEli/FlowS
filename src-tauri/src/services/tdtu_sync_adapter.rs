use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::models::{CalendarEventSource, NewCalendarEvent, NewTask, TaskImportance, TaskSource};

/// Normalized TDTU crawl records (reference: tool-check-calender/docs/dataset-structure.md)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TdtuScheduleSession {
    pub student_id: Option<String>,
    pub subject_name: String,
    pub room: String,
    pub day_of_week: Option<String>,
    pub session_date: String, // YYYY-MM-DD
    pub start_period: i32,
    pub end_period: i32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TdtuExam {
    pub subject_name: String,
    pub exam_date: String, // YYYY-MM-DD
    pub start_time: String, // HH:MM or ""
    pub end_time: String,   // HH:MM or ""
    pub exam_room: String,
    pub exam_type: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TdtuDeadline {
    pub course_id: String,
    pub course_name: String,
    pub activity_name: String,
    pub activity_url: String,
    pub due_date: String, // ISO datetime with tz
    pub completion_status: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TdtuDataset {
    pub schedule: Vec<TdtuScheduleSession>,
    pub exams: Vec<TdtuExam>,
    pub deadlines: Vec<TdtuDeadline>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncMappingResult {
    pub calendar_events: Vec<NewCalendarEvent>,
    pub task_candidates: Vec<NewTask>,
}

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let out = hasher.finalize();
    format!("{:x}", out)
}

fn period_start_time(period: i32) -> Option<NaiveTime> {
    // Common TDTU period mapping (approximate; reference impl may differ). Used for mapping only.
    match period {
        1 => NaiveTime::from_hms_opt(7, 0, 0),
        2 => NaiveTime::from_hms_opt(7, 50, 0),
        3 => NaiveTime::from_hms_opt(9, 0, 0),
        4 => NaiveTime::from_hms_opt(9, 50, 0),
        5 => NaiveTime::from_hms_opt(10, 40, 0),
        6 => NaiveTime::from_hms_opt(13, 0, 0),
        7 => NaiveTime::from_hms_opt(13, 50, 0),
        8 => NaiveTime::from_hms_opt(15, 0, 0),
        9 => NaiveTime::from_hms_opt(15, 50, 0),
        10 => NaiveTime::from_hms_opt(16, 40, 0),
        11 => NaiveTime::from_hms_opt(17, 30, 0),
        12 => NaiveTime::from_hms_opt(18, 20, 0),
        13 => NaiveTime::from_hms_opt(19, 10, 0),
        14 => NaiveTime::from_hms_opt(20, 0, 0),
        _ => None,
    }
}

fn period_end_time(period: i32) -> Option<NaiveTime> {
    period_start_time(period).map(|t| t + Duration::minutes(50))
}

fn local_dt_to_rfc3339(date: NaiveDate, time: NaiveTime) -> String {
    // Store as +07:00 for TDTU context by default (mapping-level assumption)
    let dt = NaiveDateTime::new(date, time);
    // Represent with fixed offset +07:00
    format!("{}+07:00", dt.format("%Y-%m-%dT%H:%M:%S"))
}

pub fn map_schedule_session(session: &TdtuScheduleSession) -> Option<NewCalendarEvent> {
    let date = NaiveDate::parse_from_str(&session.session_date, "%Y-%m-%d").ok()?;
    let start_t = period_start_time(session.start_period)?;
    let end_t = period_end_time(session.end_period)?;
    let start_at = local_dt_to_rfc3339(date, start_t);
    let end_at = local_dt_to_rfc3339(date, end_t);

    let source_key = format!(
        "tdtu:schedule:{}:{}:{}:{}:{}",
        session.session_date,
        session.start_period,
        session.end_period,
        session.subject_name.trim().to_lowercase(),
        session.status.trim().to_lowercase()
    );
    let hash_input = format!(
        "{}|{}|{}|{}|{}|{}|{}",
        source_key,
        session.subject_name,
        session.room,
        session.start_period,
        session.end_period,
        session.status,
        session.session_date
    );

    Some(NewCalendarEvent {
        title: session.subject_name.clone(),
        description: Some(format!("Room: {} | Status: {}", session.room, session.status)),
        start_at: Some(start_at),
        end_at: Some(end_at),
        all_day: false,
        timezone: Some("Asia/Ho_Chi_Minh".into()),
        source: CalendarEventSource::School,
        event_kind: Some("class_session".into()),
        external_source: Some("tdtu".into()),
        external_id: None,
        source_key: Some(source_key),
        source_hash: Some(sha256_hex(&hash_input)),
        related_task_id: None,
        status: Some(session.status.clone()),
        metadata_json: serde_json::to_string(session).ok(),
        last_seen_at: Some(Utc::now().to_rfc3339()),
    })
}

pub fn map_exam(exam: &TdtuExam) -> Option<NewCalendarEvent> {
    let date = NaiveDate::parse_from_str(&exam.exam_date, "%Y-%m-%d").ok()?;
    let start_at = if exam.start_time.trim().is_empty() {
        Some(format!("{}T00:00:00+07:00", exam.exam_date))
    } else {
        NaiveTime::parse_from_str(exam.start_time.trim(), "%H:%M")
            .ok()
            .map(|t| local_dt_to_rfc3339(date, t))
    }?;
    let end_at = if exam.end_time.trim().is_empty() {
        None
    } else {
        NaiveTime::parse_from_str(exam.end_time.trim(), "%H:%M")
            .ok()
            .map(|t| local_dt_to_rfc3339(date, t))
    };

    let exam_type = exam.exam_type.clone().unwrap_or_default();
    let source_key = format!(
        "tdtu:exam:{}:{}:{}",
        exam.exam_date,
        exam.subject_name.trim().to_lowercase(),
        exam_type.trim().to_lowercase()
    );
    let hash_input = format!(
        "{}|{}|{}|{}|{}|{}",
        source_key, exam.subject_name, exam.exam_date, exam.start_time, exam.end_time, exam.exam_room
    );

    Some(NewCalendarEvent {
        title: format!("Exam: {}", exam.subject_name),
        description: Some(format!(
            "Room: {}{}",
            exam.exam_room,
            exam.notes.as_ref().map(|n| format!(" | {}", n)).unwrap_or_default()
        )),
        start_at: Some(start_at),
        end_at,
        all_day: exam.start_time.trim().is_empty(),
        timezone: Some("Asia/Ho_Chi_Minh".into()),
        source: CalendarEventSource::School,
        event_kind: Some("exam".into()),
        external_source: Some("tdtu".into()),
        external_id: None,
        source_key: Some(source_key),
        source_hash: Some(sha256_hex(&hash_input)),
        related_task_id: None,
        status: exam.exam_type.clone(),
        metadata_json: serde_json::to_string(exam).ok(),
        last_seen_at: Some(Utc::now().to_rfc3339()),
    })
}

pub fn map_deadline(deadline: &TdtuDeadline) -> Option<(NewCalendarEvent, NewTask)> {
    if deadline.completion_status.trim().to_lowercase() != "incomplete" {
        return None;
    }
    let due = DateTime::parse_from_rfc3339(&deadline.due_date)
        .ok()
        .map(|dt| dt.with_timezone(&Utc).to_rfc3339())
        .or_else(|| Some(deadline.due_date.clone()))?;

    let source_key = if !deadline.activity_url.trim().is_empty() {
        format!("tdtu:deadline:url:{}", deadline.activity_url.trim())
    } else {
        format!(
            "tdtu:deadline:{}:{}:{}",
            deadline.course_id.trim().to_lowercase(),
            deadline.activity_name.trim().to_lowercase(),
            deadline.due_date.trim()
        )
    };
    let hash_input = format!(
        "{}|{}|{}|{}|{}",
        source_key, deadline.course_name, deadline.activity_name, deadline.activity_url, deadline.due_date
    );
    let source_hash = sha256_hex(&hash_input);
    let title = if deadline.activity_name.trim().is_empty() {
        deadline.course_name.clone()
    } else {
        format!("{} - {}", deadline.course_name, deadline.activity_name)
    };

    let event = NewCalendarEvent {
        title: title.clone(),
        description: Some(format!("Deadline ({})", deadline.completion_status)),
        start_at: Some(due.clone()),
        end_at: Some(due.clone()),
        all_day: false,
        timezone: Some("Asia/Ho_Chi_Minh".into()),
        source: CalendarEventSource::Deadline,
        event_kind: Some("deadline".into()),
        external_source: Some("tdtu".into()),
        external_id: if deadline.activity_url.is_empty() { None } else { Some(deadline.activity_url.clone()) },
        source_key: Some(source_key.clone()),
        source_hash: Some(source_hash.clone()),
        related_task_id: None,
        status: Some(deadline.completion_status.clone()),
        metadata_json: serde_json::to_string(deadline).ok(),
        last_seen_at: Some(Utc::now().to_rfc3339()),
    };

    let task = NewTask {
        title,
        description: Some("Synced from TDTU eLearning deadline".into()),
        deadline_at: Some(due),
        importance: TaskImportance::High,
        source: TaskSource::School,
        external_source: Some("tdtu".into()),
        external_id: if deadline.activity_url.is_empty() { None } else { Some(deadline.activity_url.clone()) },
        source_key: Some(source_key),
        source_hash: Some(source_hash),
        last_seen_at: Some(Utc::now().to_rfc3339()),
    };

    Some((event, task))
}

pub fn map_dataset(dataset: &TdtuDataset) -> SyncMappingResult {
    let mut result = SyncMappingResult::default();
    for session in &dataset.schedule {
        if let Some(event) = map_schedule_session(session) {
            result.calendar_events.push(event);
        }
    }
    for exam in &dataset.exams {
        if let Some(event) = map_exam(exam) {
            result.calendar_events.push(event);
        }
    }
    for deadline in &dataset.deadlines {
        if let Some((event, task)) = map_deadline(deadline) {
            result.calendar_events.push(event);
            result.task_candidates.push(task);
        }
    }
    result
}

/// Adapter boundary for fetching/mapping TDTU data. Concrete fetch implementations can be
/// Rust-native, WebView-assisted, or a local bridge to reference tooling.
pub trait TdtuSyncAdapter {
    fn fetch_dataset(&self, window_start: &str, window_end: &str) -> Result<TdtuDataset, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_schedule_exam_deadline_with_stable_keys() {
        let dataset = TdtuDataset {
            schedule: vec![TdtuScheduleSession {
                student_id: Some("123".into()),
                subject_name: "Software Engineering".into(),
                room: "D0205-B".into(),
                day_of_week: Some("Thursday".into()),
                session_date: "2026-11-05".into(),
                start_period: 7,
                end_period: 9,
                status: "scheduled".into(),
            }],
            exams: vec![TdtuExam {
                subject_name: "Software Engineering".into(),
                exam_date: "2026-09-30".into(),
                start_time: "07:30".into(),
                end_time: "".into(),
                exam_room: "A507".into(),
                exam_type: Some("Giữa kỳ".into()),
                notes: Some("note".into()),
            }],
            deadlines: vec![TdtuDeadline {
                course_id: "123".into(),
                course_name: "CNPM".into(),
                activity_name: "BT1".into(),
                activity_url: "https://elearning.tdtu.edu.vn/mod/assign/view.php?id=1".into(),
                due_date: "2026-11-12T23:59:00+07:00".into(),
                completion_status: "incomplete".into(),
            }],
        };

        let mapped = map_dataset(&dataset);
        assert_eq!(mapped.calendar_events.len(), 3);
        assert_eq!(mapped.task_candidates.len(), 1);
        assert!(mapped.calendar_events.iter().all(|e| e.source_key.is_some() && e.source_hash.is_some()));
        assert!(mapped.task_candidates[0].source_key.as_ref().unwrap().contains("tdtu:deadline"));

        let mapped_again = map_dataset(&dataset);
        assert_eq!(mapped.calendar_events[0].source_key, mapped_again.calendar_events[0].source_key);
        assert_eq!(mapped.calendar_events[0].source_hash, mapped_again.calendar_events[0].source_hash);
    }
}
