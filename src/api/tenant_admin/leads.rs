use axum::extract::{Extension, Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::models::company::Company;
use crate::models::lead::{Lead, LeadKanbanCard, LeadRow};
use crate::models::tenant_admin::TenantUser;

use super::contacts::view_permission_filter;

#[derive(Deserialize)]
pub struct LeadPayload {
    pub title: String,
    pub description: Option<String>,
    pub lead_value: Option<Decimal>,
    pub expected_close_date: Option<String>,
    pub person_id: Option<i64>,
    pub lead_source_id: Option<i64>,
    pub lead_type_id: Option<i64>,
    pub lead_pipeline_id: Option<i64>,
    pub lead_pipeline_stage_id: Option<i64>,
    pub user_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct StagePayload {
    pub lead_pipeline_stage_id: i64,
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub pipeline_id: Option<i64>,
}

pub async fn list(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Query(query): Query<ListQuery>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let vp = view_permission_filter(user.id, &user.view_permission);
    let pipeline_filter = query
        .pipeline_id
        .map(|pid| format!(" AND l.lead_pipeline_id = {pid}"))
        .unwrap_or_default();

    let sql = format!(
        "SELECT l.*,
                p.name AS person_name,
                CONCAT(u.first_name, ' ', u.last_name) AS user_name,
                ls.name AS source_name,
                lt.name AS type_name,
                stg.name AS stage_name,
                pip.name AS pipeline_name
         FROM leads l
         LEFT JOIN persons p ON p.id = l.person_id
         LEFT JOIN users u ON u.id = l.user_id
         LEFT JOIN lead_sources ls ON ls.id = l.lead_source_id
         LEFT JOIN lead_types lt ON lt.id = l.lead_type_id
         LEFT JOIN lead_pipeline_stages lps ON lps.id = l.lead_pipeline_stage_id
         LEFT JOIN lead_stages stg ON stg.id = lps.lead_stage_id
         LEFT JOIN lead_pipelines pip ON pip.id = l.lead_pipeline_id
         WHERE true{vp}{pipeline_filter}
         ORDER BY l.id DESC",
        vp = vp.replace("t.user_id", "l.user_id"),
        pipeline_filter = pipeline_filter,
    );

    let leads = guard.fetch_all(sqlx::query_as::<_, LeadRow>(&sql)).await;
    let _ = guard.release().await;

    match leads {
        Ok(l) => Json(serde_json::json!({ "data": l })).into_response(),
        Err(e) => {
            tracing::error!("Failed to list leads: {e}");
            internal_error()
        }
    }
}

pub async fn kanban(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Query(query): Query<ListQuery>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let vp = view_permission_filter(user.id, &user.view_permission);
    let pipeline_filter = query
        .pipeline_id
        .map(|pid| format!(" AND l.lead_pipeline_id = {pid}"))
        .unwrap_or_default();

    let sql = format!(
        "SELECT l.id, l.title, l.lead_value, l.lead_pipeline_stage_id,
                p.name AS person_name, l.created_at
         FROM leads l
         LEFT JOIN persons p ON p.id = l.person_id
         WHERE l.status IS NULL{vp}{pipeline_filter}
         ORDER BY l.id DESC",
        vp = vp.replace("t.user_id", "l.user_id"),
        pipeline_filter = pipeline_filter,
    );

    let cards = guard
        .fetch_all(sqlx::query_as::<_, LeadKanbanCard>(&sql))
        .await;
    let _ = guard.release().await;

    match cards {
        Ok(c) => Json(serde_json::json!({ "data": c })).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch kanban data: {e}");
            internal_error()
        }
    }
}

pub async fn store(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Json(payload): Json<LeadPayload>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let assigned_user = payload.user_id.unwrap_or(user.id);
    let expected_close: Option<chrono::NaiveDate> = payload
        .expected_close_date
        .as_deref()
        .and_then(|d| d.parse().ok());

    let result = guard
        .fetch_one(
            sqlx::query_as::<_, Lead>(
                "INSERT INTO leads (title, description, lead_value, expected_close_date,
                    person_id, lead_source_id, lead_type_id, lead_pipeline_id,
                    lead_pipeline_stage_id, user_id)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING *",
            )
            .bind(&payload.title)
            .bind(&payload.description)
            .bind(&payload.lead_value)
            .bind(expected_close)
            .bind(&payload.person_id)
            .bind(&payload.lead_source_id)
            .bind(&payload.lead_type_id)
            .bind(&payload.lead_pipeline_id)
            .bind(&payload.lead_pipeline_stage_id)
            .bind(assigned_user),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(l) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "data": l, "message": "Lead created successfully." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to create lead: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to create lead." })),
            )
                .into_response()
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

    let lead = guard
        .fetch_optional(sqlx::query_as::<_, Lead>("SELECT * FROM leads WHERE id = $1").bind(id))
        .await;

    let _ = guard.release().await;

    match lead {
        Ok(Some(l)) => Json(serde_json::json!({ "data": l })).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Lead not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch lead: {e}");
            internal_error()
        }
    }
}

pub async fn update(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Path(id): Path<i64>,
    Json(payload): Json<LeadPayload>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let expected_close: Option<chrono::NaiveDate> = payload
        .expected_close_date
        .as_deref()
        .and_then(|d| d.parse().ok());

    let result = guard
        .fetch_optional(
            sqlx::query_as::<_, Lead>(
                "UPDATE leads
                 SET title = $1, description = $2, lead_value = $3, expected_close_date = $4,
                     person_id = $5, lead_source_id = $6, lead_type_id = $7,
                     lead_pipeline_id = $8, lead_pipeline_stage_id = $9, user_id = $10,
                     updated_at = NOW()
                 WHERE id = $11 RETURNING *",
            )
            .bind(&payload.title)
            .bind(&payload.description)
            .bind(&payload.lead_value)
            .bind(expected_close)
            .bind(&payload.person_id)
            .bind(&payload.lead_source_id)
            .bind(&payload.lead_type_id)
            .bind(&payload.lead_pipeline_id)
            .bind(&payload.lead_pipeline_stage_id)
            .bind(&payload.user_id)
            .bind(id),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(Some(l)) => {
            Json(serde_json::json!({ "data": l, "message": "Lead updated successfully." }))
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Lead not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to update lead: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to update lead." })),
            )
                .into_response()
        }
    }
}

pub async fn update_stage(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Path(id): Path<i64>,
    Json(payload): Json<StagePayload>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard
        .fetch_optional(
            sqlx::query_as::<_, Lead>(
                "UPDATE leads SET lead_pipeline_stage_id = $1, updated_at = NOW()
                 WHERE id = $2 RETURNING *",
            )
            .bind(payload.lead_pipeline_stage_id)
            .bind(id),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(Some(l)) => {
            Json(serde_json::json!({ "data": l, "message": "Stage updated." })).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Lead not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to update lead stage: {e}");
            internal_error()
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

    let result = guard
        .execute(sqlx::query("DELETE FROM leads WHERE id = $1").bind(id))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            Json(serde_json::json!({ "message": "Lead deleted successfully." })).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Lead not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to delete lead: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to delete lead." })),
            )
                .into_response()
        }
    }
}

fn internal_error() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": "An internal error occurred." })),
    )
        .into_response()
}
