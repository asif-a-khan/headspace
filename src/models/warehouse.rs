use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Warehouse {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub contact_name: Option<String>,
    pub contact_emails: Option<serde_json::Value>,
    pub contact_numbers: Option<serde_json::Value>,
    pub contact_address: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
