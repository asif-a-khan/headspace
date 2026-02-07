use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

use crate::db::Database;
use crate::models::company::Company;

#[derive(Deserialize)]
pub struct TenantPayload {
    pub name: String,
    pub domain: String,
    pub email: Option<String>,
    pub cname: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

pub async fn list(Extension(db): Extension<Database>) -> Response {
    let tenants = sqlx::query_as::<_, Company>(
        "SELECT * FROM main.companies ORDER BY id DESC",
    )
    .fetch_all(db.reader())
    .await
    .unwrap_or_default();

    Json(serde_json::json!({ "data": tenants })).into_response()
}

pub async fn store(
    Extension(db): Extension<Database>,
    Json(payload): Json<TenantPayload>,
) -> Response {
    let schema_name = format!("tenant_{}", payload.domain.replace(['-', '.'], "_"));

    let result = sqlx::query_as::<_, Company>(
        "INSERT INTO main.companies (name, domain, email, cname, description, is_active, schema_name)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING *",
    )
    .bind(&payload.name)
    .bind(&payload.domain)
    .bind(&payload.email)
    .bind(&payload.cname)
    .bind(&payload.description)
    .bind(payload.is_active.unwrap_or(true))
    .bind(&schema_name)
    .fetch_one(db.writer())
    .await;

    match result {
        Ok(tenant) => {
            // Create the tenant's PostgreSQL schema, run migrations + seed.
            // setup_new_tenant produces a !Send future (sqlx Migrator HRTB issue),
            // so we run it on a blocking thread via Handle::block_on which doesn't
            // require Send. Acceptable for infrequent tenant creation.
            if let Err(e) = crate::db::migrate::create_tenant_schema(db.writer(), &schema_name).await {
                tracing::error!("Failed to create tenant schema: {e}");
            } else {
                let pool = db.writer().clone();
                let schema = schema_name.clone();
                let domain = payload.domain.clone();
                tokio::task::spawn_blocking(move || {
                    tokio::runtime::Handle::current().block_on(async {
                        crate::db::migrate::setup_new_tenant(&pool, &schema, &domain).await;
                    });
                });
            }

            (
                StatusCode::CREATED,
                Json(serde_json::json!({ "data": tenant, "message": "Tenant created successfully." })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create tenant: {e}");
            let msg = if e.to_string().contains("duplicate key") {
                "A tenant with this domain already exists."
            } else {
                "Failed to create tenant."
            };
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
    Path(id): Path<i64>,
) -> Response {
    let tenant = sqlx::query_as::<_, Company>(
        "SELECT * FROM main.companies WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(db.reader())
    .await;

    match tenant {
        Ok(Some(t)) => Json(serde_json::json!({ "data": t })).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Tenant not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch tenant: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "An internal error occurred." })),
            )
                .into_response()
        }
    }
}

pub async fn update(
    Extension(db): Extension<Database>,
    Path(id): Path<i64>,
    Json(payload): Json<TenantPayload>,
) -> Response {
    let result = sqlx::query_as::<_, Company>(
        "UPDATE main.companies
         SET name = $1, domain = $2, email = $3, cname = $4, description = $5,
             is_active = $6, updated_at = NOW()
         WHERE id = $7
         RETURNING *",
    )
    .bind(&payload.name)
    .bind(&payload.domain)
    .bind(&payload.email)
    .bind(&payload.cname)
    .bind(&payload.description)
    .bind(payload.is_active.unwrap_or(true))
    .bind(id)
    .fetch_optional(db.writer())
    .await;

    match result {
        Ok(Some(tenant)) => {
            Json(serde_json::json!({ "data": tenant, "message": "Tenant updated successfully." }))
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Tenant not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to update tenant: {e}");
            let msg = if e.to_string().contains("duplicate key") {
                "A tenant with this domain already exists."
            } else {
                "Failed to update tenant."
            };
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": msg })),
            )
                .into_response()
        }
    }
}

pub async fn destroy(
    Extension(db): Extension<Database>,
    Path(id): Path<i64>,
) -> Response {
    let result = sqlx::query("DELETE FROM main.companies WHERE id = $1")
        .bind(id)
        .execute(db.writer())
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            Json(serde_json::json!({ "message": "Tenant deleted successfully." })).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Tenant not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to delete tenant: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to delete tenant." })),
            )
                .into_response()
        }
    }
}
