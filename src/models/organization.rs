use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Organization {
    pub id: i64,
    pub name: String,
    pub address: Option<serde_json::Value>,
    pub user_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Organization with the assigned user's name included.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct OrganizationRow {
    pub id: i64,
    pub name: String,
    pub address: Option<serde_json::Value>,
    pub user_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_name: Option<String>,
}
