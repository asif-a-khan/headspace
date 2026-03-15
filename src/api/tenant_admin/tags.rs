use axum::Json;
use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::auth::bouncer::bouncer;
use crate::db::Database;
use crate::db::guard::TenantGuard;
use crate::models::company::Company;
use crate::models::tag::Tag;
use crate::models::tenant_admin::TenantUser;

#[derive(Deserialize)]
pub struct TagPayload {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Deserialize)]
pub struct AttachPayload {
    pub entity_type: String,
    pub entity_id: i64,
    pub tag_id: i64,
}

pub async fn list(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    if let Err(resp) = bouncer(&user, "tags") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let tags = guard
        .fetch_all(sqlx::query_as::<_, Tag>("SELECT * FROM tags ORDER BY name"))
        .await;

    let _ = guard.release().await;

    match tags {
        Ok(t) => Json(serde_json::json!({ "data": t })).into_response(),
        Err(e) => {
            tracing::error!("Failed to list tags: {e}");
            internal_error()
        }
    }
}

pub async fn store(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Json(payload): Json<TagPayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "tags.create") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let color = payload.color.unwrap_or_else(|| "#6366F1".to_string());

    let result = guard
        .fetch_one(
            sqlx::query_as::<_, Tag>(
                "INSERT INTO tags (name, color, user_id) VALUES ($1, $2, $3) RETURNING *",
            )
            .bind(&payload.name)
            .bind(&color)
            .bind(user.id),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(t) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "data": t, "message": "Tag created successfully." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to create tag: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to create tag." })),
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
    if let Err(resp) = bouncer(&user, "tags.edit") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let tag = guard
        .fetch_optional(sqlx::query_as::<_, Tag>("SELECT * FROM tags WHERE id = $1").bind(id))
        .await;

    let _ = guard.release().await;

    match tag {
        Ok(Some(t)) => Json(serde_json::json!({ "data": t })).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Tag not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch tag: {e}");
            internal_error()
        }
    }
}

pub async fn update(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
    Json(payload): Json<TagPayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "tags.edit") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard
        .fetch_optional(
            sqlx::query_as::<_, Tag>(
                "UPDATE tags SET name = $1, color = $2, updated_at = NOW() WHERE id = $3 RETURNING *",
            )
            .bind(&payload.name)
            .bind(&payload.color)
            .bind(id),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(Some(t)) => {
            Json(serde_json::json!({ "data": t, "message": "Tag updated successfully." }))
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Tag not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to update tag: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to update tag." })),
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
    if let Err(resp) = bouncer(&user, "tags.delete") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard
        .execute(sqlx::query("DELETE FROM tags WHERE id = $1").bind(id))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            Json(serde_json::json!({ "message": "Tag deleted successfully." })).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Tag not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to delete tag: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to delete tag." })),
            )
                .into_response()
        }
    }
}

pub async fn attach(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Json(payload): Json<AttachPayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "tags.edit") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let sql = match payload.entity_type.as_str() {
        "persons" => {
            "INSERT INTO person_tags (person_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        }
        "organizations" => {
            "INSERT INTO organization_tags (organization_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        }
        "leads" => "INSERT INTO lead_tags (lead_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        "products" => {
            "INSERT INTO product_tags (product_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        }
        _ => {
            let _ = guard.release().await;
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid entity type." })),
            )
                .into_response();
        }
    };

    let result = guard
        .execute(
            sqlx::query(sql)
                .bind(payload.entity_id)
                .bind(payload.tag_id),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(_) => Json(serde_json::json!({ "message": "Tag attached." })).into_response(),
        Err(e) => {
            tracing::error!("Failed to attach tag: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to attach tag." })),
            )
                .into_response()
        }
    }
}

pub async fn detach(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Json(payload): Json<AttachPayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "tags.edit") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let sql = match payload.entity_type.as_str() {
        "persons" => "DELETE FROM person_tags WHERE person_id = $1 AND tag_id = $2",
        "organizations" => {
            "DELETE FROM organization_tags WHERE organization_id = $1 AND tag_id = $2"
        }
        "leads" => "DELETE FROM lead_tags WHERE lead_id = $1 AND tag_id = $2",
        "products" => "DELETE FROM product_tags WHERE product_id = $1 AND tag_id = $2",
        _ => {
            let _ = guard.release().await;
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid entity type." })),
            )
                .into_response();
        }
    };

    let result = guard
        .execute(
            sqlx::query(sql)
                .bind(payload.entity_id)
                .bind(payload.tag_id),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(_) => Json(serde_json::json!({ "message": "Tag detached." })).into_response(),
        Err(e) => {
            tracing::error!("Failed to detach tag: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to detach tag." })),
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
