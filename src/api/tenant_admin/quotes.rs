use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::models::company::Company;
use crate::models::quote::{Quote, QuoteItem, QuoteRow};
use crate::models::tenant_admin::TenantUser;

#[derive(Deserialize)]
pub struct QuoteItemPayload {
    pub id: Option<i64>,
    pub sku: Option<String>,
    pub name: Option<String>,
    pub quantity: Option<i32>,
    pub price: Option<Decimal>,
    pub discount_percent: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub tax_percent: Option<Decimal>,
    pub tax_amount: Option<Decimal>,
    pub total: Option<Decimal>,
    pub product_id: Option<i64>,
    pub is_delete: Option<bool>,
}

#[derive(Deserialize)]
pub struct QuotePayload {
    pub subject: String,
    pub description: Option<String>,
    pub billing_address: Option<serde_json::Value>,
    pub shipping_address: Option<serde_json::Value>,
    pub discount_percent: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub tax_amount: Option<Decimal>,
    pub adjustment_amount: Option<Decimal>,
    pub sub_total: Option<Decimal>,
    pub grand_total: Option<Decimal>,
    pub expired_at: Option<String>,
    pub person_id: Option<i64>,
    pub user_id: Option<i64>,
    pub items: Option<Vec<QuoteItemPayload>>,
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

    let quotes = guard
        .fetch_all(sqlx::query_as::<_, QuoteRow>(
            "SELECT q.id, q.subject, q.grand_total, q.expired_at, q.person_id, q.user_id, q.created_at,
                    p.name AS person_name,
                    CONCAT(u.first_name, ' ', u.last_name) AS user_name
             FROM quotes q
             LEFT JOIN persons p ON p.id = q.person_id
             LEFT JOIN users u ON u.id = q.user_id
             ORDER BY q.id DESC",
        ))
        .await;

    let _ = guard.release().await;

    match quotes {
        Ok(q) => Json(serde_json::json!({ "data": q })).into_response(),
        Err(e) => {
            tracing::error!("Failed to list quotes: {e}");
            internal_error()
        }
    }
}

pub async fn store(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Json(payload): Json<QuotePayload>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let assigned_user = payload.user_id.unwrap_or(user.id);
    let expired_at: Option<chrono::DateTime<chrono::Utc>> =
        payload.expired_at.as_deref().and_then(|s| s.parse().ok());

    let result = guard
        .fetch_one(
            sqlx::query_as::<_, Quote>(
                "INSERT INTO quotes (subject, description, billing_address, shipping_address,
                    discount_percent, discount_amount, tax_amount, adjustment_amount,
                    sub_total, grand_total, expired_at, person_id, user_id)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                 RETURNING *",
            )
            .bind(&payload.subject)
            .bind(&payload.description)
            .bind(&payload.billing_address)
            .bind(&payload.shipping_address)
            .bind(&payload.discount_percent)
            .bind(&payload.discount_amount)
            .bind(&payload.tax_amount)
            .bind(&payload.adjustment_amount)
            .bind(&payload.sub_total)
            .bind(&payload.grand_total)
            .bind(expired_at)
            .bind(&payload.person_id)
            .bind(assigned_user),
        )
        .await;

    // Save line items
    if let Ok(ref quote) = result {
        if let Some(items) = &payload.items {
            for item in items {
                if item.is_delete.unwrap_or(false) {
                    continue;
                }
                let _ = guard
                    .execute(
                        sqlx::query(
                            "INSERT INTO quote_items (sku, name, quantity, price, discount_percent, discount_amount, tax_percent, tax_amount, total, product_id, quote_id)
                             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                        )
                        .bind(&item.sku)
                        .bind(&item.name)
                        .bind(item.quantity.unwrap_or(1))
                        .bind(item.price.unwrap_or_default())
                        .bind(&item.discount_percent)
                        .bind(&item.discount_amount)
                        .bind(&item.tax_percent)
                        .bind(&item.tax_amount)
                        .bind(item.total.unwrap_or_default())
                        .bind(&item.product_id)
                        .bind(quote.id),
                    )
                    .await;
            }
        }
    }

    let _ = guard.release().await;

    match result {
        Ok(q) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "data": q, "message": "Quote created successfully." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to create quote: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to create quote." })),
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

    let quote = guard
        .fetch_optional(sqlx::query_as::<_, Quote>("SELECT * FROM quotes WHERE id = $1").bind(id))
        .await;

    let items = guard
        .fetch_all(
            sqlx::query_as::<_, QuoteItem>(
                "SELECT * FROM quote_items WHERE quote_id = $1 ORDER BY id",
            )
            .bind(id),
        )
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    match quote {
        Ok(Some(q)) => {
            Json(serde_json::json!({ "data": q, "items": items })).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Quote not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch quote: {e}");
            internal_error()
        }
    }
}

pub async fn update(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Path(id): Path<i64>,
    Json(payload): Json<QuotePayload>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let expired_at: Option<chrono::DateTime<chrono::Utc>> =
        payload.expired_at.as_deref().and_then(|s| s.parse().ok());

    let result = guard
        .fetch_optional(
            sqlx::query_as::<_, Quote>(
                "UPDATE quotes SET subject = $1, description = $2, billing_address = $3,
                    shipping_address = $4, discount_percent = $5, discount_amount = $6,
                    tax_amount = $7, adjustment_amount = $8, sub_total = $9, grand_total = $10,
                    expired_at = $11, person_id = $12, user_id = $13, updated_at = NOW()
                 WHERE id = $14 RETURNING *",
            )
            .bind(&payload.subject)
            .bind(&payload.description)
            .bind(&payload.billing_address)
            .bind(&payload.shipping_address)
            .bind(&payload.discount_percent)
            .bind(&payload.discount_amount)
            .bind(&payload.tax_amount)
            .bind(&payload.adjustment_amount)
            .bind(&payload.sub_total)
            .bind(&payload.grand_total)
            .bind(expired_at)
            .bind(&payload.person_id)
            .bind(&payload.user_id)
            .bind(id),
        )
        .await;

    // Handle items: update existing, insert new, delete marked
    if let Ok(Some(_)) = &result {
        if let Some(items) = &payload.items {
            for item in items {
                if item.is_delete.unwrap_or(false) {
                    if let Some(item_id) = item.id {
                        let _ = guard
                            .execute(sqlx::query("DELETE FROM quote_items WHERE id = $1").bind(item_id))
                            .await;
                    }
                } else if let Some(item_id) = item.id {
                    let _ = guard
                        .execute(
                            sqlx::query(
                                "UPDATE quote_items SET sku = $1, name = $2, quantity = $3, price = $4,
                                    discount_percent = $5, discount_amount = $6, tax_percent = $7,
                                    tax_amount = $8, total = $9, product_id = $10, updated_at = NOW()
                                 WHERE id = $11",
                            )
                            .bind(&item.sku)
                            .bind(&item.name)
                            .bind(item.quantity.unwrap_or(1))
                            .bind(item.price.unwrap_or_default())
                            .bind(&item.discount_percent)
                            .bind(&item.discount_amount)
                            .bind(&item.tax_percent)
                            .bind(&item.tax_amount)
                            .bind(item.total.unwrap_or_default())
                            .bind(&item.product_id)
                            .bind(item_id),
                        )
                        .await;
                } else {
                    let _ = guard
                        .execute(
                            sqlx::query(
                                "INSERT INTO quote_items (sku, name, quantity, price, discount_percent, discount_amount, tax_percent, tax_amount, total, product_id, quote_id)
                                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                            )
                            .bind(&item.sku)
                            .bind(&item.name)
                            .bind(item.quantity.unwrap_or(1))
                            .bind(item.price.unwrap_or_default())
                            .bind(&item.discount_percent)
                            .bind(&item.discount_amount)
                            .bind(&item.tax_percent)
                            .bind(&item.tax_amount)
                            .bind(item.total.unwrap_or_default())
                            .bind(&item.product_id)
                            .bind(id),
                        )
                        .await;
                }
            }
        }
    }

    let _ = guard.release().await;

    match result {
        Ok(Some(q)) => {
            Json(serde_json::json!({ "data": q, "message": "Quote updated successfully." }))
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Quote not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to update quote: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to update quote." })),
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
        .execute(sqlx::query("DELETE FROM quotes WHERE id = $1").bind(id))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            Json(serde_json::json!({ "message": "Quote deleted successfully." })).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Quote not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to delete quote: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to delete quote." })),
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
