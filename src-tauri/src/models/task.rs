use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskImportance {
    Extreme,
    High,
    Mid,
    Low,
    Simple,
}

impl TaskImportance {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Extreme => "extreme",
            Self::High => "high",
            Self::Mid => "mid",
            Self::Low => "low",
            Self::Simple => "simple",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "extreme" => Some(Self::Extreme),
            "high" => Some(Self::High),
            "mid" => Some(Self::Mid),
            "low" => Some(Self::Low),
            "simple" => Some(Self::Simple),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskSource {
    Personal,
    School,
}

impl TaskSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Personal => "personal",
            Self::School => "school",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "personal" => Some(Self::Personal),
            "school" => Some(Self::School),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub created_at: String,
    pub deadline_at: Option<String>,
    pub importance: TaskImportance,
    pub source: TaskSource,
    pub external_source: Option<String>,
    pub external_id: Option<String>,
    pub source_key: Option<String>,
    pub source_hash: Option<String>,
    pub completed_at: Option<String>,
    pub is_focused: bool,
    pub focus_order: Option<i64>,
    pub updated_at: String,
    pub last_seen_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewTask {
    pub title: String,
    pub description: Option<String>,
    pub deadline_at: Option<String>,
    pub importance: TaskImportance,
    pub source: TaskSource,
    pub external_source: Option<String>,
    pub external_id: Option<String>,
    pub source_key: Option<String>,
    pub source_hash: Option<String>,
    pub last_seen_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TaskUpdate {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub deadline_at: Option<Option<String>>,
    pub importance: Option<TaskImportance>,
    pub source: Option<TaskSource>,
    pub external_source: Option<Option<String>>,
    pub external_id: Option<Option<String>>,
    pub source_key: Option<Option<String>>,
    pub source_hash: Option<Option<String>>,
    pub last_seen_at: Option<Option<String>>,
}

pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

pub fn parse_rfc3339(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}
