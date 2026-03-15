use axum::Json;
use axum::extract::{Extension, Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use validator::Validate;

use crate::auth::bouncer::{bouncer, validate_payload};
use crate::db::Database;
use crate::db::guard::TenantGuard;
use crate::models::company::Company;
use crate::models::person::{Person, PersonRow};
use crate::models::tenant_admin::TenantUser;

use super::contacts::view_permission_filter;

#[derive(Deserialize, Validate)]
pub struct PersonPayload {
    #[validate(length(min = 1, message = "Name is required."))]
    pub name: String,
    pub emails: Option<serde_json::Value>,
    pub contact_numbers: Option<serde_json::Value>,
    pub job_title: Option<String>,
    pub organization_id: Option<i64>,
    pub user_id: Option<i64>,
}

pub async fn list(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    if let Err(resp) = bouncer(&user, "contacts.persons") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let vp = view_permission_filter(user.id, &user.view_permission);
    let sql = format!(
        "SELECT t.*, o.name AS organization_name,
                CONCAT(u.first_name, ' ', u.last_name) AS user_name
         FROM persons t
         LEFT JOIN organizations o ON o.id = t.organization_id
         LEFT JOIN users u ON u.id = t.user_id
         WHERE true{vp}
         ORDER BY t.id DESC"
    );

    let persons = guard.fetch_all(sqlx::query_as::<_, PersonRow>(&sql)).await;
    let _ = guard.release().await;

    match persons {
        Ok(p) => Json(serde_json::json!({ "data": p })).into_response(),
        Err(e) => {
            tracing::error!("Failed to list persons: {e}");
            internal_error()
        }
    }
}

pub async fn store(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Json(payload): Json<PersonPayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "contacts.persons.create") {
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

    let assigned_user = payload.user_id.unwrap_or(user.id);
    let emails = payload.emails.unwrap_or(serde_json::json!([]));
    let contact_numbers = payload.contact_numbers.unwrap_or(serde_json::json!([]));

    let result = guard
        .fetch_one(sqlx::query_as::<_, Person>(
            "INSERT INTO persons (name, emails, contact_numbers, job_title, organization_id, user_id)
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
        )
        .bind(&payload.name)
        .bind(&emails)
        .bind(&contact_numbers)
        .bind(&payload.job_title)
        .bind(payload.organization_id)
        .bind(assigned_user))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(p) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "data": p, "message": "Person created successfully." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to create person: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to create person." })),
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
    if let Err(resp) = bouncer(&user, "contacts.persons.edit") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let vp = view_permission_filter(user.id, &user.view_permission).replace("t.user_id", "user_id");
    let person = guard
        .fetch_optional(
            sqlx::query_as::<_, Person>(&format!("SELECT * FROM persons WHERE id = $1{vp}"))
                .bind(id),
        )
        .await;

    let _ = guard.release().await;

    match person {
        Ok(Some(p)) => Json(serde_json::json!({ "data": p })).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Person not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch person: {e}");
            internal_error()
        }
    }
}

pub async fn update(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
    Json(payload): Json<PersonPayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "contacts.persons.edit") {
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

    let emails = payload.emails.unwrap_or(serde_json::json!([]));
    let contact_numbers = payload.contact_numbers.unwrap_or(serde_json::json!([]));
    let vp = view_permission_filter(user.id, &user.view_permission).replace("t.user_id", "user_id");

    let result = guard
        .fetch_optional(
            sqlx::query_as::<_, Person>(&format!(
                "UPDATE persons
             SET name = $1, emails = $2, contact_numbers = $3, job_title = $4,
                 organization_id = $5, user_id = $6, updated_at = NOW()
             WHERE id = $7{vp} RETURNING *"
            ))
            .bind(&payload.name)
            .bind(&emails)
            .bind(&contact_numbers)
            .bind(&payload.job_title)
            .bind(payload.organization_id)
            .bind(payload.user_id)
            .bind(id),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(Some(p)) => {
            Json(serde_json::json!({ "data": p, "message": "Person updated successfully." }))
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Person not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to update person: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to update person." })),
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
    if let Err(resp) = bouncer(&user, "contacts.persons.delete") {
        return resp;
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
        .execute(sqlx::query(&format!("DELETE FROM persons WHERE id = $1{vp}")).bind(id))
        .await;
    let _ = guard.release().await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            Json(serde_json::json!({ "message": "Person deleted successfully." })).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Person not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to delete person: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to delete person." })),
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
    if let Err(resp) = bouncer(&user, "contacts.persons.delete") {
        return resp;
    }
    if payload.ids.is_empty() {
        return Json(serde_json::json!({ "message": "No persons selected.", "deleted_count": 0 }))
            .into_response();
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
        .execute(
            sqlx::query(&format!(
                "DELETE FROM persons WHERE id = ANY($1::bigint[]){vp}"
            ))
            .bind(&payload.ids),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(r) => {
            let count = r.rows_affected();
            Json(serde_json::json!({ "message": format!("{count} person(s) deleted."), "deleted_count": count })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to mass delete persons: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to delete persons." })),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

pub async fn search(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Query(query): Query<SearchQuery>,
) -> Response {
    if let Err(resp) = bouncer(&user, "contacts.persons") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let search_term = query.q.unwrap_or_default();
    let persons = guard
        .fetch_all(
            sqlx::query_as::<_, Person>(
                "SELECT * FROM persons WHERE name ILIKE $1 ORDER BY name LIMIT 20",
            )
            .bind(format!("%{search_term}%")),
        )
        .await;

    let _ = guard.release().await;

    match persons {
        Ok(p) => Json(serde_json::json!({ "data": p })).into_response(),
        Err(e) => {
            tracing::error!("Failed to search persons: {e}");
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
