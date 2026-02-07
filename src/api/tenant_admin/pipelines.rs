use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::models::company::Company;
use crate::models::pipeline::{LeadPipeline, LeadStage, PipelineStageDetail, PipelineWithStages};

#[derive(Deserialize)]
pub struct PipelinePayload {
    pub name: String,
    pub is_default: Option<bool>,
    pub rotten_days: Option<i32>,
    pub stages: Option<Vec<StagePayload>>,
}

#[derive(Deserialize)]
pub struct StagePayload {
    pub lead_stage_id: i64,
    pub probability: Option<i32>,
    pub sort_order: Option<i32>,
}

pub async fn list(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let pipelines = guard
        .fetch_all(sqlx::query_as::<_, LeadPipeline>(
            "SELECT * FROM lead_pipelines ORDER BY id",
        ))
        .await
        .unwrap_or_default();

    let stages = guard
        .fetch_all(sqlx::query_as::<_, LeadStage>(
            "SELECT * FROM lead_stages ORDER BY id",
        ))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    Json(serde_json::json!({ "data": pipelines, "stages": stages })).into_response()
}

pub async fn store(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Json(payload): Json<PipelinePayload>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    // If this pipeline is default, unset other defaults
    if payload.is_default.unwrap_or(false) {
        let _ = guard
            .execute(sqlx::query("UPDATE lead_pipelines SET is_default = false"))
            .await;
    }

    let result = guard
        .fetch_one(sqlx::query_as::<_, LeadPipeline>(
            "INSERT INTO lead_pipelines (name, is_default, rotten_days)
             VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(&payload.name)
        .bind(payload.is_default.unwrap_or(false))
        .bind(payload.rotten_days.unwrap_or(30)))
        .await;

    match result {
        Ok(pipeline) => {
            // Create pipeline stages
            if let Some(stages) = &payload.stages {
                for (i, s) in stages.iter().enumerate() {
                    let _ = guard
                        .execute(
                            sqlx::query(
                                "INSERT INTO lead_pipeline_stages (lead_pipeline_id, lead_stage_id, probability, sort_order)
                                 VALUES ($1, $2, $3, $4)",
                            )
                            .bind(pipeline.id)
                            .bind(s.lead_stage_id)
                            .bind(s.probability.unwrap_or(100))
                            .bind(s.sort_order.unwrap_or(i as i32)),
                        )
                        .await;
                }
            }

            let _ = guard.release().await;
            (StatusCode::CREATED, Json(serde_json::json!({ "data": pipeline, "message": "Pipeline created successfully." }))).into_response()
        }
        Err(e) => {
            let _ = guard.release().await;
            tracing::error!("Failed to create pipeline: {e}");
            (StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "Failed to create pipeline." }))).into_response()
        }
    }
}

pub async fn show(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Path(id): Path<i64>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let pipeline = guard
        .fetch_optional(sqlx::query_as::<_, LeadPipeline>(
            "SELECT * FROM lead_pipelines WHERE id = $1",
        ).bind(id))
        .await;

    let stages = guard
        .fetch_all(sqlx::query_as::<_, PipelineStageDetail>(
            "SELECT ps.*, ls.code AS stage_code, ls.name AS stage_name
             FROM lead_pipeline_stages ps
             JOIN lead_stages ls ON ls.id = ps.lead_stage_id
             WHERE ps.lead_pipeline_id = $1
             ORDER BY ps.sort_order",
        ).bind(id))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    match pipeline {
        Ok(Some(p)) => {
            let result = PipelineWithStages { pipeline: p, stages };
            Json(serde_json::json!({ "data": result })).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Pipeline not found." }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch pipeline: {e}");
            internal_error()
        }
    }
}

pub async fn update(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Path(id): Path<i64>,
    Json(payload): Json<PipelinePayload>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    if payload.is_default.unwrap_or(false) {
        let _ = guard
            .execute(sqlx::query("UPDATE lead_pipelines SET is_default = false"))
            .await;
    }

    let result = guard
        .fetch_optional(sqlx::query_as::<_, LeadPipeline>(
            "UPDATE lead_pipelines
             SET name = $1, is_default = $2, rotten_days = $3, updated_at = NOW()
             WHERE id = $4 RETURNING *",
        )
        .bind(&payload.name)
        .bind(payload.is_default.unwrap_or(false))
        .bind(payload.rotten_days.unwrap_or(30))
        .bind(id))
        .await;

    match result {
        Ok(Some(pipeline)) => {
            // Replace pipeline stages
            if let Some(stages) = &payload.stages {
                let _ = guard
                    .execute(sqlx::query("DELETE FROM lead_pipeline_stages WHERE lead_pipeline_id = $1").bind(pipeline.id))
                    .await;

                for (i, s) in stages.iter().enumerate() {
                    let _ = guard
                        .execute(
                            sqlx::query(
                                "INSERT INTO lead_pipeline_stages (lead_pipeline_id, lead_stage_id, probability, sort_order)
                                 VALUES ($1, $2, $3, $4)",
                            )
                            .bind(pipeline.id)
                            .bind(s.lead_stage_id)
                            .bind(s.probability.unwrap_or(100))
                            .bind(s.sort_order.unwrap_or(i as i32)),
                        )
                        .await;
                }
            }

            let _ = guard.release().await;
            Json(serde_json::json!({ "data": pipeline, "message": "Pipeline updated successfully." })).into_response()
        }
        Ok(None) => {
            let _ = guard.release().await;
            (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Pipeline not found." }))).into_response()
        }
        Err(e) => {
            let _ = guard.release().await;
            tracing::error!("Failed to update pipeline: {e}");
            (StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "Failed to update pipeline." }))).into_response()
        }
    }
}

pub async fn destroy(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Path(id): Path<i64>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard.execute(sqlx::query("DELETE FROM lead_pipelines WHERE id = $1").bind(id)).await;
    let _ = guard.release().await;

    match result {
        Ok(r) if r.rows_affected() > 0 => Json(serde_json::json!({ "message": "Pipeline deleted successfully." })).into_response(),
        Ok(_) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Pipeline not found." }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to delete pipeline: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Failed to delete pipeline." }))).into_response()
        }
    }
}

fn internal_error() -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "An internal error occurred." }))).into_response()
}
