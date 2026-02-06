use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Company {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
    pub domain: String,
    pub cname: Option<String>,
    pub description: Option<String>,
    pub is_active: bool,
    pub schema_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
