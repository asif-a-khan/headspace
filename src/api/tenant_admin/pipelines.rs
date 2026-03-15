use axum::Json;
use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use validator::Validate;

use crate::auth::bouncer::{bouncer, validate_payload};
use crate::db::Database;
use crate::db::guard::TenantGuard;
use crate::models::company::Company;
use crate::models::pipeline::{LeadPipeline, LeadStage, PipelineStageDetail, PipelineWithStages};
use crate::models::tenant_admin::TenantUser;

#[derive(Deserialize, Validate)]
pub struct PipelinePayload {
    #[validate(length(min = 1, message = "Name is required."))]
    pub name: String,
    pub is_default: Option<bool>,
    pub rotten_days: Option<i32>,
    pub stages: Option<Vec<StagePayload>>,
}

#[derive(Deserialize)]
pub struct StagePayload {
    pub name: String,
    pub probability: Option<i32>,
    pub sort_order: Option<i32>,
}

pub async fn list(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.pipelines") {
        return resp;
    }

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
    Extension(user): Extension<TenantUser>,
    Json(payload): Json<PipelinePayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.pipelines.create") {
        return resp;
    }
    if let Err(resp) = validate_payload(&payload) {
        return resp;
    }

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
        .fetch_one(
            sqlx::query_as::<_, LeadPipeline>(
                "INSERT INTO lead_pipelines (name, is_default, rotten_days)
             VALUES ($1, $2, $3) RETURNING *",
            )
            .bind(&payload.name)
            .bind(payload.is_default.unwrap_or(false))
            .bind(payload.rotten_days.unwrap_or(30)),
        )
        .await;

    match result {
        Ok(pipeline) => {
            // Create pipeline stages
            let mut next_sort: i32 = 0;
            let mut has_won = false;
            let mut has_lost = false;
            if let Some(stages) = &payload.stages {
                for (i, s) in stages.iter().enumerate() {
                    let name_lower = s.name.to_lowercase();
                    if name_lower == "won" {
                        has_won = true;
                    }
                    if name_lower == "lost" {
                        has_lost = true;
                    }
                    let stage_id = match find_or_create_stage(&mut guard, &s.name).await {
                        Ok(id) => id,
                        Err(e) => {
                            tracing::error!("Failed to find/create stage '{}': {e}", s.name);
                            continue;
                        }
                    };
                    let _ = guard
                        .execute(
                            sqlx::query(
                                "INSERT INTO lead_pipeline_stages (lead_pipeline_id, lead_stage_id, probability, sort_order)
                                 VALUES ($1, $2, $3, $4)",
                            )
                            .bind(pipeline.id)
                            .bind(stage_id)
                            .bind(s.probability.unwrap_or(100))
                            .bind(s.sort_order.unwrap_or(i as i32)),
                        )
                        .await;
                    next_sort = s.sort_order.unwrap_or(i as i32) + 1;
                }
            }

            // Auto-add Won and Lost stages if not provided (mirrors Krayin)
            if !has_won && let Ok(won_id) = find_or_create_stage(&mut guard, "Won").await {
                let _ = guard
                        .execute(sqlx::query(
                            "INSERT INTO lead_pipeline_stages (lead_pipeline_id, lead_stage_id, probability, sort_order)
                             VALUES ($1, $2, 100, $3)",
                        ).bind(pipeline.id).bind(won_id).bind(next_sort))
                        .await;
                next_sort += 1;
            }
            if !has_lost && let Ok(lost_id) = find_or_create_stage(&mut guard, "Lost").await {
                let _ = guard
                        .execute(sqlx::query(
                            "INSERT INTO lead_pipeline_stages (lead_pipeline_id, lead_stage_id, probability, sort_order)
                             VALUES ($1, $2, 0, $3)",
                        ).bind(pipeline.id).bind(lost_id).bind(next_sort))
                        .await;
            }

            let _ = guard.release().await;
            (StatusCode::CREATED, Json(serde_json::json!({ "data": pipeline, "message": "Pipeline created successfully." }))).into_response()
        }
        Err(e) => {
            let _ = guard.release().await;
            tracing::error!("Failed to create pipeline: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to create pipeline." })),
            )
                .into_response()
        }
    }
}

pub async fn show(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.pipelines.edit") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let pipeline = guard
        .fetch_optional(
            sqlx::query_as::<_, LeadPipeline>("SELECT * FROM lead_pipelines WHERE id = $1")
                .bind(id),
        )
        .await;

    let stages = guard
        .fetch_all(
            sqlx::query_as::<_, PipelineStageDetail>(
                "SELECT ps.*, ls.code AS stage_code, ls.name AS stage_name
             FROM lead_pipeline_stages ps
             JOIN lead_stages ls ON ls.id = ps.lead_stage_id
             WHERE ps.lead_pipeline_id = $1
             ORDER BY ps.sort_order",
            )
            .bind(id),
        )
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    match pipeline {
        Ok(Some(p)) => {
            let result = PipelineWithStages {
                pipeline: p,
                stages,
            };
            Json(serde_json::json!({ "data": result })).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Pipeline not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch pipeline: {e}");
            internal_error()
        }
    }
}

pub async fn update(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
    Json(payload): Json<PipelinePayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.pipelines.edit") {
        return resp;
    }
    if let Err(resp) = validate_payload(&payload) {
        return resp;
    }

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
        .fetch_optional(
            sqlx::query_as::<_, LeadPipeline>(
                "UPDATE lead_pipelines
             SET name = $1, is_default = $2, rotten_days = $3, updated_at = NOW()
             WHERE id = $4 RETURNING *",
            )
            .bind(&payload.name)
            .bind(payload.is_default.unwrap_or(false))
            .bind(payload.rotten_days.unwrap_or(30))
            .bind(id),
        )
        .await;

    match result {
        Ok(Some(pipeline)) => {
            // Sync pipeline stages (preserving IDs for leads)
            if let Some(stages) = &payload.stages {
                // Resolve all stage names to lead_stage IDs
                let mut resolved: Vec<(i64, i32, i32)> = Vec::new(); // (lead_stage_id, probability, sort_order)
                for (i, s) in stages.iter().enumerate() {
                    match find_or_create_stage(&mut guard, &s.name).await {
                        Ok(stage_id) => {
                            resolved.push((
                                stage_id,
                                s.probability.unwrap_or(100),
                                s.sort_order.unwrap_or(i as i32),
                            ));
                        }
                        Err(e) => {
                            tracing::error!("Failed to find/create stage '{}': {e}", s.name);
                        }
                    }
                }

                // Get existing pipeline_stages for this pipeline
                let existing = guard
                    .fetch_all(sqlx::query_as::<_, (i64, i64)>(
                        "SELECT id, lead_stage_id FROM lead_pipeline_stages WHERE lead_pipeline_id = $1",
                    ).bind(pipeline.id))
                    .await
                    .unwrap_or_default();

                let new_stage_ids: std::collections::HashSet<i64> =
                    resolved.iter().map(|(sid, _, _)| *sid).collect();

                // Update existing stages that are kept, insert new ones
                for (stage_id, probability, sort_order) in &resolved {
                    let existing_ps = existing.iter().find(|(_, lsid)| lsid == stage_id);
                    if let Some((ps_id, _)) = existing_ps {
                        let _ = guard
                            .execute(
                                sqlx::query(
                                    "UPDATE lead_pipeline_stages SET probability = $1, sort_order = $2 WHERE id = $3",
                                )
                                .bind(probability)
                                .bind(sort_order)
                                .bind(ps_id),
                            )
                            .await;
                    } else {
                        let _ = guard
                            .execute(
                                sqlx::query(
                                    "INSERT INTO lead_pipeline_stages (lead_pipeline_id, lead_stage_id, probability, sort_order)
                                     VALUES ($1, $2, $3, $4)",
                                )
                                .bind(pipeline.id)
                                .bind(stage_id)
                                .bind(probability)
                                .bind(sort_order),
                            )
                            .await;
                    }
                }

                // Remove stages no longer in payload, migrating leads first
                let removed: Vec<i64> = existing
                    .iter()
                    .filter(|(_, lsid)| !new_stage_ids.contains(lsid))
                    .map(|(ps_id, _)| *ps_id)
                    .collect();

                if !removed.is_empty() {
                    let first_remaining = guard
                        .fetch_optional(
                            sqlx::query_as::<_, (i64,)>(
                                "SELECT id FROM lead_pipeline_stages
                             WHERE lead_pipeline_id = $1 AND id != ALL($2)
                             ORDER BY sort_order LIMIT 1",
                            )
                            .bind(pipeline.id)
                            .bind(&removed),
                        )
                        .await
                        .ok()
                        .flatten()
                        .map(|(id,)| id);

                    for ps_id in &removed {
                        if let Some(target_id) = first_remaining {
                            let _ = guard
                                .execute(
                                    sqlx::query(
                                        "UPDATE leads SET lead_pipeline_stage_id = $1 WHERE lead_pipeline_stage_id = $2",
                                    )
                                    .bind(target_id)
                                    .bind(ps_id),
                                )
                                .await;
                        }
                        let _ = guard
                            .execute(
                                sqlx::query("DELETE FROM lead_pipeline_stages WHERE id = $1")
                                    .bind(ps_id),
                            )
                            .await;
                    }
                }
            }

            let _ = guard.release().await;
            Json(serde_json::json!({ "data": pipeline, "message": "Pipeline updated successfully." })).into_response()
        }
        Ok(None) => {
            let _ = guard.release().await;
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Pipeline not found." })),
            )
                .into_response()
        }
        Err(e) => {
            let _ = guard.release().await;
            tracing::error!("Failed to update pipeline: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to update pipeline." })),
            )
                .into_response()
        }
    }
}

pub async fn destroy(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.pipelines.delete") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    // Check if this is the default pipeline
    let is_default = guard
        .fetch_optional(
            sqlx::query_as::<_, (bool,)>("SELECT is_default FROM lead_pipelines WHERE id = $1")
                .bind(id),
        )
        .await;

    if let Ok(Some((true,))) = is_default {
        let _ = guard.release().await;
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "Cannot delete the default pipeline." })),
        )
            .into_response();
    }

    // Check if any leads use this pipeline
    let lead_count = guard
        .fetch_one(
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM leads WHERE lead_pipeline_id = $1")
                .bind(id),
        )
        .await;

    if let Ok((c,)) = lead_count
        && c > 0
    {
        let _ = guard.release().await;
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(
                serde_json::json!({ "error": "Cannot delete a pipeline that has leads assigned." }),
            ),
        )
            .into_response();
    }

    let result = guard
        .execute(sqlx::query("DELETE FROM lead_pipelines WHERE id = $1").bind(id))
        .await;
    let _ = guard.release().await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            Json(serde_json::json!({ "message": "Pipeline deleted successfully." })).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Pipeline not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to delete pipeline: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to delete pipeline." })),
            )
                .into_response()
        }
    }
}

/// Find or create a lead_stage by name, returning its id.
async fn find_or_create_stage(guard: &mut TenantGuard, name: &str) -> Result<i64, sqlx::Error> {
    let code = name
        .to_lowercase()
        .replace(' ', "-")
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "");
    let row = guard
        .fetch_one(
            sqlx::query_as::<_, (i64,)>(
                "INSERT INTO lead_stages (code, name, is_user_defined)
             VALUES ($1, $2, true)
             ON CONFLICT (code) DO UPDATE SET name = EXCLUDED.name
             RETURNING id",
            )
            .bind(&code)
            .bind(name),
        )
        .await?;
    Ok(row.0)
}

fn internal_error() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": "An internal error occurred." })),
    )
        .into_response()
}
