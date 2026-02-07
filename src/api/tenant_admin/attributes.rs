use axum::extract::{Extension, Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use validator::Validate;

use crate::auth::bouncer::{bouncer, validate_payload};
use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::models::attribute::{Attribute, AttributeOption};
use crate::models::company::Company;
use crate::models::tenant_admin::TenantUser;

#[derive(Deserialize)]
pub struct ListQuery {
    pub entity_type: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct AttributePayload {
    #[validate(length(min = 1, message = "Code is required."))]
    pub code: String,
    #[validate(length(min = 1, message = "Name is required."))]
    pub name: String,
    #[serde(rename = "type")]
    pub attr_type: String,
    #[validate(length(min = 1, message = "Entity type is required."))]
    pub entity_type: String,
    pub sort_order: Option<i32>,
    pub validation: Option<String>,
    pub is_required: Option<bool>,
    pub is_unique: Option<bool>,
    pub quick_add: Option<bool>,
    pub options: Option<Vec<OptionPayload>>,
}

#[derive(Deserialize)]
pub struct OptionPayload {
    pub id: Option<i64>,
    pub name: String,
    pub sort_order: Option<i32>,
    #[serde(default)]
    pub is_delete: bool,
}

pub async fn list(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Query(query): Query<ListQuery>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.attributes") { return resp; }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let attributes = if let Some(ref entity_type) = query.entity_type {
        guard
            .fetch_all(
                sqlx::query_as::<_, Attribute>(
                    "SELECT * FROM attributes WHERE entity_type = $1 ORDER BY sort_order, id",
                )
                .bind(entity_type),
            )
            .await
    } else {
        guard
            .fetch_all(sqlx::query_as::<_, Attribute>(
                "SELECT * FROM attributes ORDER BY entity_type, sort_order, id",
            ))
            .await
    };

    let _ = guard.release().await;

    match attributes {
        Ok(attrs) => Json(serde_json::json!({ "data": attrs })).into_response(),
        Err(e) => {
            tracing::error!("Failed to list attributes: {e}");
            internal_error()
        }
    }
}

pub async fn store(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Json(payload): Json<AttributePayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.attributes.create") { return resp; }
    if let Err(resp) = validate_payload(&payload) { return resp; }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard
        .fetch_one(
            sqlx::query_as::<_, Attribute>(
                "INSERT INTO attributes (code, name, type, entity_type, sort_order, validation, is_required, is_unique, quick_add)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                 RETURNING *",
            )
            .bind(&payload.code)
            .bind(&payload.name)
            .bind(&payload.attr_type)
            .bind(&payload.entity_type)
            .bind(payload.sort_order.unwrap_or(0))
            .bind(&payload.validation)
            .bind(payload.is_required.unwrap_or(false))
            .bind(payload.is_unique.unwrap_or(false))
            .bind(payload.quick_add.unwrap_or(false)),
        )
        .await;

    match result {
        Ok(attr) => {
            // Create options if this is a select/multiselect type
            if let Some(options) = &payload.options {
                if attr.attr_type == "select" || attr.attr_type == "multiselect" {
                    for (i, opt) in options.iter().enumerate() {
                        let _ = guard
                            .execute(
                                sqlx::query(
                                    "INSERT INTO attribute_options (name, sort_order, attribute_id) VALUES ($1, $2, $3)",
                                )
                                .bind(&opt.name)
                                .bind(opt.sort_order.unwrap_or(i as i32))
                                .bind(attr.id),
                            )
                            .await;
                    }
                }
            }

            let _ = guard.release().await;
            (
                StatusCode::CREATED,
                Json(serde_json::json!({ "data": attr, "message": "Attribute created successfully." })),
            )
                .into_response()
        }
        Err(e) => {
            let _ = guard.release().await;
            let msg = if e.to_string().contains("duplicate key") {
                "An attribute with this code already exists for this entity type."
            } else {
                "Failed to create attribute."
            };
            tracing::error!("Failed to create attribute: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": msg })),
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
    if let Err(resp) = bouncer(&user, "settings.attributes.edit") { return resp; }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let attr = guard
        .fetch_optional(sqlx::query_as::<_, Attribute>("SELECT * FROM attributes WHERE id = $1").bind(id))
        .await;

    let options = guard
        .fetch_all(
            sqlx::query_as::<_, AttributeOption>(
                "SELECT * FROM attribute_options WHERE attribute_id = $1 ORDER BY sort_order",
            )
            .bind(id),
        )
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    match attr {
        Ok(Some(a)) => Json(serde_json::json!({ "data": a, "options": options })).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Attribute not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch attribute: {e}");
            internal_error()
        }
    }
}

pub async fn update(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
    Json(payload): Json<AttributePayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.attributes.edit") { return resp; }
    if let Err(resp) = validate_payload(&payload) { return resp; }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard
        .fetch_optional(
            sqlx::query_as::<_, Attribute>(
                "UPDATE attributes
                 SET name = $1, sort_order = $2, validation = $3,
                     is_required = $4, is_unique = $5, quick_add = $6, updated_at = NOW()
                 WHERE id = $7 AND is_user_defined = true
                 RETURNING *",
            )
            .bind(&payload.name)
            .bind(payload.sort_order.unwrap_or(0))
            .bind(&payload.validation)
            .bind(payload.is_required.unwrap_or(false))
            .bind(payload.is_unique.unwrap_or(false))
            .bind(payload.quick_add.unwrap_or(false))
            .bind(id),
        )
        .await;

    match result {
        Ok(Some(attr)) => {
            // Sync options for select/multiselect types
            if let Some(options) = &payload.options {
                if attr.attr_type == "select" || attr.attr_type == "multiselect" {
                    for (i, opt) in options.iter().enumerate() {
                        if opt.is_delete {
                            if let Some(opt_id) = opt.id {
                                let _ = guard
                                    .execute(
                                        sqlx::query("DELETE FROM attribute_options WHERE id = $1 AND attribute_id = $2")
                                            .bind(opt_id)
                                            .bind(attr.id),
                                    )
                                    .await;
                            }
                        } else if let Some(opt_id) = opt.id {
                            let _ = guard
                                .execute(
                                    sqlx::query(
                                        "UPDATE attribute_options SET name = $1, sort_order = $2, updated_at = NOW() WHERE id = $3 AND attribute_id = $4",
                                    )
                                    .bind(&opt.name)
                                    .bind(opt.sort_order.unwrap_or(i as i32))
                                    .bind(opt_id)
                                    .bind(attr.id),
                                )
                                .await;
                        } else {
                            let _ = guard
                                .execute(
                                    sqlx::query(
                                        "INSERT INTO attribute_options (name, sort_order, attribute_id) VALUES ($1, $2, $3)",
                                    )
                                    .bind(&opt.name)
                                    .bind(opt.sort_order.unwrap_or(i as i32))
                                    .bind(attr.id),
                                )
                                .await;
                        }
                    }
                }
            }

            let _ = guard.release().await;
            Json(serde_json::json!({ "data": attr, "message": "Attribute updated successfully." }))
                .into_response()
        }
        Ok(None) => {
            let _ = guard.release().await;
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Attribute not found or is a system attribute." })),
            )
                .into_response()
        }
        Err(e) => {
            let _ = guard.release().await;
            tracing::error!("Failed to update attribute: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to update attribute." })),
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
    if let Err(resp) = bouncer(&user, "settings.attributes.delete") { return resp; }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard
        .execute(sqlx::query("DELETE FROM attributes WHERE id = $1 AND is_user_defined = true").bind(id))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            Json(serde_json::json!({ "message": "Attribute deleted successfully." })).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Attribute not found or is a system attribute." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to delete attribute: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to delete attribute." })),
            )
                .into_response()
        }
    }
}

/// List options for a specific attribute.
pub async fn list_options(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.attributes") { return resp; }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let options = guard
        .fetch_all(
            sqlx::query_as::<_, AttributeOption>(
                "SELECT * FROM attribute_options WHERE attribute_id = $1 ORDER BY sort_order",
            )
            .bind(id),
        )
        .await;

    let _ = guard.release().await;

    match options {
        Ok(opts) => Json(serde_json::json!({ "data": opts })).into_response(),
        Err(e) => {
            tracing::error!("Failed to list attribute options: {e}");
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
