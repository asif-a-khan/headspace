use axum::Json;
use axum::extract::{Extension, Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use rust_decimal::Decimal;
use serde::Deserialize;
use validator::Validate;

use crate::auth::bouncer::{bouncer, validate_payload};
use crate::db::Database;
use crate::db::guard::TenantGuard;
use crate::models::company::Company;
use crate::models::product::Product;
use crate::models::tenant_admin::TenantUser;

#[derive(Deserialize, Validate)]
pub struct ProductPayload {
    #[validate(length(min = 1, message = "SKU is required."))]
    pub sku: String,
    #[validate(length(min = 1, message = "Name is required."))]
    pub name: String,
    pub description: Option<String>,
    pub price: Option<Decimal>,
    pub quantity: Option<i32>,
}

pub async fn list(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    if let Err(resp) = bouncer(&user, "products") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let products = guard
        .fetch_all(sqlx::query_as::<_, Product>(
            "SELECT * FROM products ORDER BY id DESC",
        ))
        .await;

    let _ = guard.release().await;

    match products {
        Ok(p) => Json(serde_json::json!({ "data": p })).into_response(),
        Err(e) => {
            tracing::error!("Failed to list products: {e}");
            internal_error()
        }
    }
}

pub async fn store(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Json(payload): Json<ProductPayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "products.create") {
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

    let price = payload.price.unwrap_or_default();
    let quantity = payload.quantity.unwrap_or(0);

    let result = guard
        .fetch_one(
            sqlx::query_as::<_, Product>(
                "INSERT INTO products (sku, name, description, price, quantity) VALUES ($1, $2, $3, $4, $5) RETURNING *",
            )
            .bind(&payload.sku)
            .bind(&payload.name)
            .bind(&payload.description)
            .bind(price)
            .bind(quantity),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(p) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "data": p, "message": "Product created successfully." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to create product: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to create product." })),
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
    if let Err(resp) = bouncer(&user, "products.edit") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let product = guard
        .fetch_optional(
            sqlx::query_as::<_, Product>("SELECT * FROM products WHERE id = $1").bind(id),
        )
        .await;

    let _ = guard.release().await;

    match product {
        Ok(Some(p)) => Json(serde_json::json!({ "data": p })).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Product not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch product: {e}");
            internal_error()
        }
    }
}

pub async fn update(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
    Json(payload): Json<ProductPayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "products.edit") {
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

    let result = guard
        .fetch_optional(
            sqlx::query_as::<_, Product>(
                "UPDATE products SET sku = $1, name = $2, description = $3, price = $4, quantity = $5, updated_at = NOW()
                 WHERE id = $6 RETURNING *",
            )
            .bind(&payload.sku)
            .bind(&payload.name)
            .bind(&payload.description)
            .bind(payload.price)
            .bind(payload.quantity)
            .bind(id),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(Some(p)) => {
            Json(serde_json::json!({ "data": p, "message": "Product updated successfully." }))
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Product not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to update product: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to update product." })),
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
    if let Err(resp) = bouncer(&user, "products.delete") {
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
        .execute(sqlx::query("DELETE FROM products WHERE id = $1").bind(id))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            Json(serde_json::json!({ "message": "Product deleted successfully." })).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Product not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to delete product: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to delete product." })),
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
    if let Err(resp) = bouncer(&user, "products.delete") {
        return resp;
    }
    if payload.ids.is_empty() {
        return Json(serde_json::json!({ "message": "No products selected.", "deleted_count": 0 }))
            .into_response();
    }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard
        .execute(
            sqlx::query("DELETE FROM products WHERE id = ANY($1::bigint[])").bind(&payload.ids),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(r) => {
            let count = r.rows_affected();
            Json(serde_json::json!({ "message": format!("{count} product(s) deleted."), "deleted_count": count })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to mass delete products: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to delete products." })),
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
    if let Err(resp) = bouncer(&user, "products") {
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
    let products = guard
        .fetch_all(
            sqlx::query_as::<_, Product>(
                "SELECT * FROM products WHERE name ILIKE $1 OR sku ILIKE $1 ORDER BY name LIMIT 20",
            )
            .bind(format!("%{search_term}%")),
        )
        .await;

    let _ = guard.release().await;

    match products {
        Ok(p) => Json(serde_json::json!({ "data": p })).into_response(),
        Err(e) => {
            tracing::error!("Failed to search products: {e}");
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
