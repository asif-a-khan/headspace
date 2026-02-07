use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct LeadSource {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct LeadType {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct LeadPipeline {
    pub id: i64,
    pub name: String,
    pub is_default: bool,
    pub rotten_days: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct LeadStage {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub is_user_defined: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct LeadPipelineStage {
    pub id: i64,
    pub probability: i32,
    pub sort_order: i32,
    pub lead_stage_id: i64,
    pub lead_pipeline_id: i64,
}

/// Pipeline with its associated stages (for API responses).
#[derive(Debug, Clone, Serialize)]
pub struct PipelineWithStages {
    #[serde(flatten)]
    pub pipeline: LeadPipeline,
    pub stages: Vec<PipelineStageDetail>,
}

/// A pipeline stage with the stage name included.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct PipelineStageDetail {
    pub id: i64,
    pub probability: i32,
    pub sort_order: i32,
    pub lead_stage_id: i64,
    pub lead_pipeline_id: i64,
    pub stage_code: String,
    pub stage_name: String,
}
