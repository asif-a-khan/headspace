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
use crate::models::organization::{Organization, OrganizationRow};
use crate::models::tenant_admin::TenantUser;

use super::contacts::view_permission_filter;

#[derive(Deserialize, Validate)]
pub struct OrganizationPayload {
    #[validate(length(min = 1, message = "Name is required."))]
    pub name: String,
    pub address: Option<serde_json::Value>,
    pub user_id: Option<i64>,
}

pub async fn list(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    if let Err(resp) = bouncer(&user, "contacts.organizations") {
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
        "SELECT t.*, CONCAT(u.first_name, ' ', u.last_name) AS user_name
         FROM organizations t
         LEFT JOIN users u ON u.id = t.user_id
         WHERE true{vp}
         ORDER BY t.id DESC"
    );

    let orgs = guard
        .fetch_all(sqlx::query_as::<_, OrganizationRow>(&sql))
        .await;

    let _ = guard.release().await;

    match orgs {
        Ok(o) => Json(serde_json::json!({ "data": o })).into_response(),
        Err(e) => {
            tracing::error!("Failed to list organizations: {e}");
            internal_error()
        }
    }
}

pub async fn store(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Json(payload): Json<OrganizationPayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "contacts.organizations.create") {
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

    let result = guard
        .fetch_one(sqlx::query_as::<_, Organization>(
            "INSERT INTO organizations (name, address, user_id) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(&payload.name)
        .bind(&payload.address)
        .bind(assigned_user))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(org) => (
            StatusCode::CREATED,
            Json(
                serde_json::json!({ "data": org, "message": "Organization created successfully." }),
            ),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to create organization: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to create organization." })),
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
    if let Err(resp) = bouncer(&user, "contacts.organizations.edit") {
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
    let org = guard
        .fetch_optional(
            sqlx::query_as::<_, Organization>(&format!(
                "SELECT * FROM organizations WHERE id = $1{vp}"
            ))
            .bind(id),
        )
        .await;

    let _ = guard.release().await;

    match org {
        Ok(Some(o)) => Json(serde_json::json!({ "data": o })).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Organization not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch organization: {e}");
            internal_error()
        }
    }
}

pub async fn update(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
    Json(payload): Json<OrganizationPayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "contacts.organizations.edit") {
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

    let vp = view_permission_filter(user.id, &user.view_permission).replace("t.user_id", "user_id");
    let result = guard
        .fetch_optional(
            sqlx::query_as::<_, Organization>(&format!(
                "UPDATE organizations SET name = $1, address = $2, user_id = $3, updated_at = NOW()
             WHERE id = $4{vp} RETURNING *"
            ))
            .bind(&payload.name)
            .bind(&payload.address)
            .bind(payload.user_id)
            .bind(id),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(Some(o)) => {
            Json(serde_json::json!({ "data": o, "message": "Organization updated successfully." }))
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Organization not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to update organization: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to update organization." })),
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
    if let Err(resp) = bouncer(&user, "contacts.organizations.delete") {
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
        .execute(sqlx::query(&format!("DELETE FROM organizations WHERE id = $1{vp}")).bind(id))
        .await;
    let _ = guard.release().await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            Json(serde_json::json!({ "message": "Organization deleted successfully." }))
                .into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Organization not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to delete organization: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to delete organization." })),
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
    if let Err(resp) = bouncer(&user, "contacts.organizations.delete") {
        return resp;
    }
    if payload.ids.is_empty() {
        return Json(
            serde_json::json!({ "message": "No organizations selected.", "deleted_count": 0 }),
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

    let vp = view_permission_filter(user.id, &user.view_permission).replace("t.user_id", "user_id");
    let result = guard
        .execute(
            sqlx::query(&format!(
                "DELETE FROM organizations WHERE id = ANY($1::bigint[]){vp}"
            ))
            .bind(&payload.ids),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(r) => {
            let count = r.rows_affected();
            Json(serde_json::json!({ "message": format!("{count} organization(s) deleted."), "deleted_count": count })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to mass delete organizations: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to delete organizations." })),
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
