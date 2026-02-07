use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Lead {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub lead_value: Option<Decimal>,
    pub status: Option<bool>,
    pub lost_reason: Option<String>,
    pub closed_at: Option<DateTime<Utc>>,
    pub expected_close_date: Option<NaiveDate>,
    pub user_id: Option<i64>,
    pub person_id: Option<i64>,
    pub lead_source_id: Option<i64>,
    pub lead_type_id: Option<i64>,
    pub lead_pipeline_id: Option<i64>,
    pub lead_pipeline_stage_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lead with joined names for list display.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct LeadRow {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub lead_value: Option<Decimal>,
    pub status: Option<bool>,
    pub lost_reason: Option<String>,
    pub closed_at: Option<DateTime<Utc>>,
    pub expected_close_date: Option<NaiveDate>,
    pub user_id: Option<i64>,
    pub person_id: Option<i64>,
    pub lead_source_id: Option<i64>,
    pub lead_type_id: Option<i64>,
    pub lead_pipeline_id: Option<i64>,
    pub lead_pipeline_stage_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub person_name: Option<String>,
    pub user_name: Option<String>,
    pub source_name: Option<String>,
    pub type_name: Option<String>,
    pub stage_name: Option<String>,
    pub pipeline_name: Option<String>,
}

/// Minimal lead info for kanban cards.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct LeadKanbanCard {
    pub id: i64,
    pub title: String,
    pub lead_value: Option<Decimal>,
    pub lead_pipeline_stage_id: Option<i64>,
    pub person_name: Option<String>,
    pub created_at: DateTime<Utc>,
}
