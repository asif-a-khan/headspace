use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::models::activity::{Activity, ActivityRow};
use crate::models::company::Company;
use crate::models::tenant_admin::TenantUser;

#[derive(Deserialize)]
pub struct ActivityPayload {
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub comment: Option<String>,
    pub additional: Option<serde_json::Value>,
    pub location: Option<String>,
    pub schedule_from: Option<String>,
    pub schedule_to: Option<String>,
    pub is_done: Option<bool>,
    pub lead_ids: Option<Vec<i64>>,
    pub person_ids: Option<Vec<i64>>,
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

    let activities = guard
        .fetch_all(sqlx::query_as::<_, ActivityRow>(
            "SELECT a.id, a.title, a.type, a.comment, a.location,
                    a.schedule_from, a.schedule_to, a.is_done, a.user_id,
                    a.created_at, a.updated_at,
                    CONCAT(u.first_name, ' ', u.last_name) AS user_name
             FROM activities a
             LEFT JOIN users u ON u.id = a.user_id
             ORDER BY a.id DESC",
        ))
        .await;

    let _ = guard.release().await;

    match activities {
        Ok(a) => Json(serde_json::json!({ "data": a })).into_response(),
        Err(e) => {
            tracing::error!("Failed to list activities: {e}");
            internal_error()
        }
    }
}

pub async fn store(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Json(payload): Json<ActivityPayload>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let schedule_from: Option<chrono::DateTime<chrono::Utc>> = payload
        .schedule_from
        .as_deref()
        .and_then(|s| s.parse().ok());
    let schedule_to: Option<chrono::DateTime<chrono::Utc>> = payload
        .schedule_to
        .as_deref()
        .and_then(|s| s.parse().ok());

    let result = guard
        .fetch_one(
            sqlx::query_as::<_, Activity>(
                "INSERT INTO activities (title, type, comment, additional, location, schedule_from, schedule_to, is_done, user_id)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *",
            )
            .bind(&payload.title)
            .bind(&payload.activity_type)
            .bind(&payload.comment)
            .bind(&payload.additional)
            .bind(&payload.location)
            .bind(schedule_from)
            .bind(schedule_to)
            .bind(payload.is_done.unwrap_or(false))
            .bind(user.id),
        )
        .await;

    match &result {
        Ok(activity) => {
            // Link to leads
            if let Some(lead_ids) = &payload.lead_ids {
                for lid in lead_ids {
                    let _ = guard
                        .execute(
                            sqlx::query("INSERT INTO lead_activities (lead_id, activity_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                                .bind(lid)
                                .bind(activity.id),
                        )
                        .await;
                }
            }
            // Link to persons
            if let Some(person_ids) = &payload.person_ids {
                for pid in person_ids {
                    let _ = guard
                        .execute(
                            sqlx::query("INSERT INTO person_activities (person_id, activity_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                                .bind(pid)
                                .bind(activity.id),
                        )
                        .await;
                }
            }
        }
        Err(_) => {}
    }

    let _ = guard.release().await;

    match result {
        Ok(a) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "data": a, "message": "Activity created successfully." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to create activity: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to create activity." })),
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

    let activity = guard
        .fetch_optional(
            sqlx::query_as::<_, Activity>("SELECT * FROM activities WHERE id = $1").bind(id),
        )
        .await;

    let _ = guard.release().await;

    match activity {
        Ok(Some(a)) => Json(serde_json::json!({ "data": a })).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Activity not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch activity: {e}");
            internal_error()
        }
    }
}

pub async fn update(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Path(id): Path<i64>,
    Json(payload): Json<ActivityPayload>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let schedule_from: Option<chrono::DateTime<chrono::Utc>> = payload
        .schedule_from
        .as_deref()
        .and_then(|s| s.parse().ok());
    let schedule_to: Option<chrono::DateTime<chrono::Utc>> = payload
        .schedule_to
        .as_deref()
        .and_then(|s| s.parse().ok());

    let result = guard
        .fetch_optional(
            sqlx::query_as::<_, Activity>(
                "UPDATE activities
                 SET title = $1, type = $2, comment = $3, additional = $4, location = $5,
                     schedule_from = $6, schedule_to = $7, is_done = $8, updated_at = NOW()
                 WHERE id = $9 RETURNING *",
            )
            .bind(&payload.title)
            .bind(&payload.activity_type)
            .bind(&payload.comment)
            .bind(&payload.additional)
            .bind(&payload.location)
            .bind(schedule_from)
            .bind(schedule_to)
            .bind(payload.is_done.unwrap_or(false))
            .bind(id),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(Some(a)) => {
            Json(serde_json::json!({ "data": a, "message": "Activity updated successfully." }))
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Activity not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to update activity: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to update activity." })),
            )
                .into_response()
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
        .execute(sqlx::query("DELETE FROM activities WHERE id = $1").bind(id))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            Json(serde_json::json!({ "message": "Activity deleted successfully." })).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Activity not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to delete activity: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to delete activity." })),
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
