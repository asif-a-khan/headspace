use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use validator::Validate;

use crate::auth::bouncer::{bouncer, validate_payload};
use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::models::company::Company;
use crate::models::email_template::EmailTemplate;
use crate::models::tenant_admin::TenantUser;

#[derive(Deserialize, Validate)]
pub struct EmailTemplatePayload {
    #[validate(length(min = 1, message = "Name is required."))]
    pub name: String,
    #[validate(length(min = 1, message = "Subject is required."))]
    pub subject: String,
    pub content: Option<String>,
}

pub async fn list(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.email_templates") { return resp; }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let templates = guard
        .fetch_all(sqlx::query_as::<_, EmailTemplate>(
            "SELECT * FROM email_templates ORDER BY id",
        ))
        .await;

    let _ = guard.release().await;

    match templates {
        Ok(t) => Json(serde_json::json!({ "data": t })).into_response(),
        Err(e) => {
            tracing::error!("Failed to list email templates: {e}");
            internal_error()
        }
    }
}

pub async fn store(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Json(payload): Json<EmailTemplatePayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.email_templates.create") { return resp; }
    if let Err(resp) = validate_payload(&payload) { return resp; }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let content = payload.content.as_deref().unwrap_or("");

    let result = guard
        .fetch_one(sqlx::query_as::<_, EmailTemplate>(
            "INSERT INTO email_templates (name, subject, content) VALUES ($1, $2, $3) RETURNING *",
        ).bind(&payload.name).bind(&payload.subject).bind(content))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(t) => (StatusCode::CREATED, Json(serde_json::json!({ "data": t, "message": "Email template created successfully." }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to create email template: {e}");
            (StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "Failed to create email template." }))).into_response()
        }
    }
}

pub async fn show(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.email_templates.edit") { return resp; }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let template = guard
        .fetch_optional(sqlx::query_as::<_, EmailTemplate>("SELECT * FROM email_templates WHERE id = $1").bind(id))
        .await;

    let _ = guard.release().await;

    match template {
        Ok(Some(t)) => Json(serde_json::json!({ "data": t })).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Email template not found." }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch email template: {e}");
            internal_error()
        }
    }
}

pub async fn update(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
    Json(payload): Json<EmailTemplatePayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.email_templates.edit") { return resp; }
    if let Err(resp) = validate_payload(&payload) { return resp; }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let content = payload.content.as_deref().unwrap_or("");

    let result = guard
        .fetch_optional(sqlx::query_as::<_, EmailTemplate>(
            "UPDATE email_templates SET name = $1, subject = $2, content = $3, updated_at = NOW() WHERE id = $4 RETURNING *",
        ).bind(&payload.name).bind(&payload.subject).bind(content).bind(id))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(Some(t)) => Json(serde_json::json!({ "data": t, "message": "Email template updated successfully." })).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Email template not found." }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to update email template: {e}");
            (StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "Failed to update email template." }))).into_response()
        }
    }
}

pub async fn destroy(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.email_templates.delete") { return resp; }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard.execute(sqlx::query("DELETE FROM email_templates WHERE id = $1").bind(id)).await;
    let _ = guard.release().await;

    match result {
        Ok(r) if r.rows_affected() > 0 => Json(serde_json::json!({ "message": "Email template deleted successfully." })).into_response(),
        Ok(_) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Email template not found." }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to delete email template: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Failed to delete email template." }))).into_response()
        }
    }
}

fn internal_error() -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "An internal error occurred." }))).into_response()
}
