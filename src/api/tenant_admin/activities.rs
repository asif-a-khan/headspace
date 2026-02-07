use axum::extract::{Extension, Multipart, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use validator::Validate;

use crate::auth::bouncer::{bouncer, validate_payload};
use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::models::activity::{Activity, ActivityRow};
use crate::models::company::Company;
use crate::models::tenant_admin::TenantUser;

use super::contacts::view_permission_filter;

#[derive(Deserialize, Validate)]
pub struct ActivityPayload {
    pub title: Option<String>,
    #[serde(rename = "type")]
    #[validate(length(min = 1, message = "Activity type is required."))]
    pub activity_type: String,
    pub comment: Option<String>,
    pub additional: Option<serde_json::Value>,
    pub location: Option<String>,
    pub schedule_from: Option<String>,
    pub schedule_to: Option<String>,
    pub is_done: Option<bool>,
    pub lead_ids: Option<Vec<i64>>,
    pub person_ids: Option<Vec<i64>>,
    pub participant_user_ids: Option<Vec<i64>>,
    pub participant_person_ids: Option<Vec<i64>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct ParticipantRow {
    user_id: Option<i64>,
    user_name: Option<String>,
    person_id: Option<i64>,
    person_name: Option<String>,
}

async fn save_participants(
    guard: &mut TenantGuard,
    activity_id: i64,
    user_ids: &[i64],
    person_ids: &[i64],
) {
    // Clear existing
    let _ = guard
        .execute(
            sqlx::query("DELETE FROM activity_participants WHERE activity_id = $1")
                .bind(activity_id),
        )
        .await;
    // Insert user participants
    for uid in user_ids {
        let _ = guard
            .execute(
                sqlx::query(
                    "INSERT INTO activity_participants (activity_id, user_id) VALUES ($1, $2)",
                )
                .bind(activity_id)
                .bind(uid),
            )
            .await;
    }
    // Insert person participants
    for pid in person_ids {
        let _ = guard
            .execute(
                sqlx::query(
                    "INSERT INTO activity_participants (activity_id, person_id) VALUES ($1, $2)",
                )
                .bind(activity_id)
                .bind(pid),
            )
            .await;
    }
}

async fn fetch_participants(guard: &mut TenantGuard, activity_id: i64) -> Vec<ParticipantRow> {
    guard
        .fetch_all(
            sqlx::query_as::<_, ParticipantRow>(
                "SELECT ap.user_id,
                        CASE WHEN ap.user_id IS NOT NULL THEN CONCAT(u.first_name, ' ', u.last_name) END AS user_name,
                        ap.person_id,
                        p.name AS person_name
                 FROM activity_participants ap
                 LEFT JOIN users u ON u.id = ap.user_id
                 LEFT JOIN persons p ON p.id = ap.person_id
                 WHERE ap.activity_id = $1
                 ORDER BY ap.id",
            )
            .bind(activity_id),
        )
        .await
        .unwrap_or_default()
}

pub async fn list(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    if let Err(resp) = bouncer(&user, "activities") { return resp; }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let vp = view_permission_filter(user.id, &user.view_permission).replace("t.user_id", "a.user_id");
    let activities = guard
        .fetch_all(sqlx::query_as::<_, ActivityRow>(&format!(
            "SELECT a.id, a.title, a.type, a.comment, a.location,
                    a.schedule_from, a.schedule_to, a.is_done, a.user_id,
                    a.created_at, a.updated_at,
                    CONCAT(u.first_name, ' ', u.last_name) AS user_name
             FROM activities a
             LEFT JOIN users u ON u.id = a.user_id
             WHERE true{vp}
             ORDER BY a.id DESC"
        )))
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
    if let Err(resp) = bouncer(&user, "activities.create") { return resp; }
    if let Err(resp) = validate_payload(&payload) { return resp; }

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
            // Save participants
            save_participants(
                &mut guard,
                activity.id,
                payload.participant_user_ids.as_deref().unwrap_or(&[]),
                payload.participant_person_ids.as_deref().unwrap_or(&[]),
            )
            .await;
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
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "activities.edit") { return resp; }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let vp = view_permission_filter(user.id, &user.view_permission).replace("t.user_id", "user_id");
    let activity = guard
        .fetch_optional(
            sqlx::query_as::<_, Activity>(&format!("SELECT * FROM activities WHERE id = $1{vp}")).bind(id),
        )
        .await;

    match &activity {
        Ok(Some(a)) => {
            let participants = fetch_participants(&mut guard, a.id).await;
            let files = guard
                .fetch_all(
                    sqlx::query_as::<_, ActivityFile>(
                        "SELECT * FROM activity_files WHERE activity_id = $1 ORDER BY id",
                    )
                    .bind(a.id),
                )
                .await
                .unwrap_or_default();
            let _ = guard.release().await;
            return Json(serde_json::json!({ "data": a, "participants": participants, "files": files })).into_response();
        }
        _ => {}
    }

    let _ = guard.release().await;

    match activity {
        Ok(Some(_)) => unreachable!(),
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
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
    Json(payload): Json<ActivityPayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "activities.edit") { return resp; }
    if let Err(resp) = validate_payload(&payload) { return resp; }

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

    let vp = view_permission_filter(user.id, &user.view_permission).replace("t.user_id", "user_id");
    let result = guard
        .fetch_optional(
            sqlx::query_as::<_, Activity>(&format!(
                "UPDATE activities
                 SET title = $1, type = $2, comment = $3, additional = $4, location = $5,
                     schedule_from = $6, schedule_to = $7, is_done = $8, updated_at = NOW()
                 WHERE id = $9{vp} RETURNING *"
            ))
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

    if let Ok(Some(ref a)) = result {
        save_participants(
            &mut guard,
            a.id,
            payload.participant_user_ids.as_deref().unwrap_or(&[]),
            payload.participant_person_ids.as_deref().unwrap_or(&[]),
        )
        .await;
    }

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
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "activities.delete") { return resp; }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let vp = view_permission_filter(user.id, &user.view_permission).replace("t.user_id", "user_id");
    let result = guard
        .execute(sqlx::query(&format!("DELETE FROM activities WHERE id = $1{vp}")).bind(id))
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

// --- Mass Delete ---

#[derive(Deserialize)]
pub struct MassDeletePayload {
    pub ids: Vec<i64>,
}

pub async fn mass_delete(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Json(payload): Json<MassDeletePayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "activities.delete") { return resp; }
    if payload.ids.is_empty() {
        return Json(serde_json::json!({ "message": "No activities selected.", "deleted_count": 0 })).into_response();
    }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let vp = view_permission_filter(user.id, &user.view_permission).replace("t.user_id", "user_id");
    let result = guard
        .execute(sqlx::query(&format!("DELETE FROM activities WHERE id = ANY($1::bigint[]){vp}")).bind(&payload.ids))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(r) => {
            let count = r.rows_affected();
            Json(serde_json::json!({ "message": format!("{count} activity(ies) deleted."), "deleted_count": count })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to mass delete activities: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Failed to delete activities." }))).into_response()
        }
    }
}

// --- Activity Files ---

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct ActivityFile {
    pub id: i64,
    pub activity_id: i64,
    pub file_name: String,
    pub file_path: String,
    pub file_type: Option<String>,
    pub file_size: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_files(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(activity_id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "activities") { return resp; }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let files = guard
        .fetch_all(
            sqlx::query_as::<_, ActivityFile>(
                "SELECT * FROM activity_files WHERE activity_id = $1 ORDER BY id",
            )
            .bind(activity_id),
        )
        .await;

    let _ = guard.release().await;

    match files {
        Ok(f) => Json(serde_json::json!({ "data": f })).into_response(),
        Err(e) => {
            tracing::error!("Failed to list activity files: {e}");
            internal_error()
        }
    }
}

pub async fn upload_file(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(activity_id): Path<i64>,
    mut multipart: Multipart,
) -> Response {
    if let Err(resp) = bouncer(&user, "activities.edit") { return resp; }

    let upload_dir = format!("uploads/{}", company.schema_name);
    if let Err(e) = tokio::fs::create_dir_all(&upload_dir).await {
        tracing::error!("Failed to create upload directory: {e}");
        return internal_error();
    }

    let mut saved_files = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let original_name = field.file_name().unwrap_or("unnamed").to_string();
        let content_type = field.content_type().map(|s| s.to_string());

        let bytes = match field.bytes().await {
            Ok(b) => b,
            Err(e) => {
                tracing::error!("Failed to read upload: {e}");
                continue;
            }
        };

        let file_size = bytes.len() as i64;
        let unique_name = format!("{}_{}", uuid::Uuid::new_v4(), original_name);
        let disk_path = format!("{upload_dir}/{unique_name}");

        if let Err(e) = tokio::fs::write(&disk_path, &bytes).await {
            tracing::error!("Failed to write file to disk: {e}");
            continue;
        }

        saved_files.push((original_name, disk_path, content_type, file_size));
    }

    if saved_files.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "No files uploaded." })),
        )
            .into_response();
    }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let mut result_files = Vec::new();
    for (name, path, content_type, size) in &saved_files {
        match guard
            .fetch_one(
                sqlx::query_as::<_, ActivityFile>(
                    "INSERT INTO activity_files (activity_id, file_name, file_path, file_type, file_size)
                     VALUES ($1, $2, $3, $4, $5) RETURNING *",
                )
                .bind(activity_id)
                .bind(name)
                .bind(path)
                .bind(content_type.as_deref())
                .bind(size),
            )
            .await
        {
            Ok(f) => result_files.push(f),
            Err(e) => tracing::error!("Failed to insert activity file record: {e}"),
        }
    }

    let _ = guard.release().await;

    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "data": result_files, "message": "Files uploaded successfully." })),
    )
        .into_response()
}

pub async fn download_file(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path((_activity_id, file_id)): Path<(i64, i64)>,
) -> Response {
    if let Err(resp) = bouncer(&user, "activities") { return resp; }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let file = guard
        .fetch_optional(
            sqlx::query_as::<_, ActivityFile>("SELECT * FROM activity_files WHERE id = $1")
                .bind(file_id),
        )
        .await;

    let _ = guard.release().await;

    let file = match file {
        Ok(Some(f)) => f,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "File not found." })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to fetch file record: {e}");
            return internal_error();
        }
    };

    let bytes = match tokio::fs::read(&file.file_path).await {
        Ok(b) => b,
        Err(e) => {
            tracing::error!("Failed to read file from disk: {e}");
            return internal_error();
        }
    };

    let content_type = file.file_type.as_deref().unwrap_or("application/octet-stream");
    let disposition = format!("attachment; filename=\"{}\"", file.file_name);

    (
        [
            (axum::http::header::CONTENT_TYPE, content_type.to_string()),
            (axum::http::header::CONTENT_DISPOSITION, disposition),
        ],
        bytes,
    )
        .into_response()
}

pub async fn delete_file(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path((_activity_id, file_id)): Path<(i64, i64)>,
) -> Response {
    if let Err(resp) = bouncer(&user, "activities.edit") { return resp; }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let file = guard
        .fetch_optional(
            sqlx::query_as::<_, ActivityFile>("SELECT * FROM activity_files WHERE id = $1")
                .bind(file_id),
        )
        .await;

    if let Ok(Some(ref f)) = file {
        // Delete from disk
        let _ = tokio::fs::remove_file(&f.file_path).await;
        // Delete from DB
        let _ = guard
            .execute(sqlx::query("DELETE FROM activity_files WHERE id = $1").bind(file_id))
            .await;
    }

    let _ = guard.release().await;

    match file {
        Ok(Some(_)) => {
            Json(serde_json::json!({ "message": "File deleted successfully." })).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "File not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to delete file: {e}");
            internal_error()
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
