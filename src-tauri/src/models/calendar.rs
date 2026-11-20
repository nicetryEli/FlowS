use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CalendarEventSource {
    Personal,
    School,
    Deadline,
}

impl CalendarEventSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Personal => "personal",
            Self::School => "school",
            Self::Deadline => "deadline",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "personal" => Some(Self::Personal),
            "school" => Some(Self::School),
            "deadline" => Some(Self::Deadline),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CalendarEvent {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub all_day: bool,
    pub timezone: Option<String>,
    pub source: CalendarEventSource,
    pub event_kind: Option<String>,
    pub external_source: Option<String>,
    pub external_id: Option<String>,
    pub source_key: Option<String>,
    pub source_hash: Option<String>,
    pub related_task_id: Option<i64>,
    pub status: Option<String>,
    pub metadata_json: Option<String>,
    pub last_seen_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewCalendarEvent {
    pub title: String,
    pub description: Option<String>,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub all_day: bool,
    pub timezone: Option<String>,
    pub source: CalendarEventSource,
    pub event_kind: Option<String>,
    pub external_source: Option<String>,
    pub external_id: Option<String>,
    pub source_key: Option<String>,
    pub source_hash: Option<String>,
    pub related_task_id: Option<i64>,
    pub status: Option<String>,
    pub metadata_json: Option<String>,
    pub last_seen_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CalendarEventUpdate {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub start_at: Option<Option<String>>,
    pub end_at: Option<Option<String>>,
    pub all_day: Option<bool>,
    pub timezone: Option<Option<String>>,
    pub source: Option<CalendarEventSource>,
    pub event_kind: Option<Option<String>>,
    pub external_source: Option<Option<String>>,
    pub external_id: Option<Option<String>>,
    pub source_key: Option<Option<String>>,
    pub source_hash: Option<Option<String>>,
    pub related_task_id: Option<Option<i64>>,
    pub status: Option<Option<String>>,
    pub metadata_json: Option<Option<String>>,
    pub last_seen_at: Option<Option<String>>,
}
