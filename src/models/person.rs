use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Person {
    pub id: i64,
    pub name: String,
    pub emails: serde_json::Value,
    pub contact_numbers: serde_json::Value,
    pub job_title: Option<String>,
    pub organization_id: Option<i64>,
    pub user_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Person with joined organization and user names.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct PersonRow {
    pub id: i64,
    pub name: String,
    pub emails: serde_json::Value,
    pub contact_numbers: serde_json::Value,
    pub job_title: Option<String>,
    pub organization_id: Option<i64>,
    pub user_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub organization_name: Option<String>,
    pub user_name: Option<String>,
}
