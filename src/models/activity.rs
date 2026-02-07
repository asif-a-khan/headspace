use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Activity {
    pub id: i64,
    pub title: Option<String>,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub activity_type: String,
    pub comment: Option<String>,
    pub additional: Option<serde_json::Value>,
    pub location: Option<String>,
    pub schedule_from: Option<DateTime<Utc>>,
    pub schedule_to: Option<DateTime<Utc>>,
    pub is_done: bool,
    pub user_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Activity with joined user name for list display.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct ActivityRow {
    pub id: i64,
    pub title: Option<String>,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub activity_type: String,
    pub comment: Option<String>,
    pub location: Option<String>,
    pub schedule_from: Option<DateTime<Utc>>,
    pub schedule_to: Option<DateTime<Utc>>,
    pub is_done: bool,
    pub user_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_name: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct ActivityParticipant {
    pub id: i64,
    pub activity_id: i64,
    pub user_id: Option<i64>,
    pub person_id: Option<i64>,
}
