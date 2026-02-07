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
    /// Days past rotten threshold (>0 means rotten). NULL for won/lost leads.
    pub rotten_days: Option<i32>,
}

/// Lead product line item.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct LeadProduct {
    pub id: i64,
    pub lead_id: i64,
    pub product_id: i64,
    pub quantity: i32,
    pub price: Decimal,
    pub amount: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lead product with product name for display.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct LeadProductRow {
    pub id: i64,
    pub lead_id: i64,
    pub product_id: i64,
    pub quantity: i32,
    pub price: Decimal,
    pub amount: Decimal,
    pub product_name: String,
    pub product_sku: String,
}

/// Minimal lead info for kanban cards.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct LeadKanbanCard {
    pub id: i64,
    pub title: String,
    pub lead_value: Option<Decimal>,
    pub lead_pipeline_stage_id: Option<i64>,
    pub person_name: Option<String>,
    pub organization_name: Option<String>,
    pub user_name: Option<String>,
    pub source_name: Option<String>,
    pub type_name: Option<String>,
    pub created_at: DateTime<Utc>,
    /// Days past rotten threshold (>0 means rotten).
    pub rotten_days: Option<i32>,
    /// JSON array of {id, name, color} for tags attached to this lead.
    pub tags_json: Option<serde_json::Value>,
}
