use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use rust_decimal::Decimal;
use serde::Deserialize;
use validator::Validate;

use crate::auth::bouncer::{bouncer, validate_payload};
use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::models::company::Company;
use crate::models::quote::{Quote, QuoteItem, QuoteRow, QuoteSearchRow};
use crate::models::tenant_admin::TenantUser;

use super::contacts::view_permission_filter;

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

#[derive(Deserialize, Validate)]
pub struct QuotePayload {
    #[validate(length(min = 1, message = "Subject is required."))]
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
    pub lead_id: Option<i64>,
}

pub async fn list(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    if let Err(resp) = bouncer(&user, "quotes") { return resp; }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let vp = view_permission_filter(user.id, &user.view_permission).replace("t.user_id", "q.user_id");
    let quotes = guard
        .fetch_all(sqlx::query_as::<_, QuoteRow>(&format!(
            "SELECT q.id, q.subject, q.grand_total, q.expired_at, q.person_id, q.user_id, q.created_at,
                    p.name AS person_name,
                    CONCAT(u.first_name, ' ', u.last_name) AS user_name
             FROM quotes q
             LEFT JOIN persons p ON p.id = q.person_id
             LEFT JOIN users u ON u.id = q.user_id
             WHERE true{vp}
             ORDER BY q.id DESC"
        )))
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

pub async fn search(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    if let Err(resp) = bouncer(&user, "quotes") { return resp; }

    let q = params.get("q").map(|s| s.as_str()).unwrap_or("");
    if q.len() < 2 {
        return Json(serde_json::json!({ "data": [] })).into_response();
    }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let pattern = format!("%{q}%");
    let results = guard
        .fetch_all(
            sqlx::query_as::<_, QuoteSearchRow>(
                "SELECT id, subject, grand_total FROM quotes WHERE subject ILIKE $1 ORDER BY id DESC LIMIT 10",
            )
            .bind(&pattern),
        )
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    Json(serde_json::json!({ "data": results })).into_response()
}

pub async fn store(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Json(payload): Json<QuotePayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "quotes.create") { return resp; }
    if let Err(resp) = validate_payload(&payload) { return resp; }

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

        // Auto-link to lead if lead_id provided
        if let Some(lead_id) = payload.lead_id {
            let _ = guard
                .execute(
                    sqlx::query(
                        "INSERT INTO lead_quotes (lead_id, quote_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                    )
                    .bind(lead_id)
                    .bind(quote.id),
                )
                .await;
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
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "quotes.edit") { return resp; }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let vp = view_permission_filter(user.id, &user.view_permission).replace("t.user_id", "user_id");
    let quote = guard
        .fetch_optional(sqlx::query_as::<_, Quote>(&format!(
            "SELECT * FROM quotes WHERE id = $1{vp}"
        )).bind(id))
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
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
    Json(payload): Json<QuotePayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "quotes.edit") { return resp; }
    if let Err(resp) = validate_payload(&payload) { return resp; }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let expired_at: Option<chrono::DateTime<chrono::Utc>> =
        payload.expired_at.as_deref().and_then(|s| s.parse().ok());
    let vp = view_permission_filter(user.id, &user.view_permission).replace("t.user_id", "user_id");

    let result = guard
        .fetch_optional(
            sqlx::query_as::<_, Quote>(&format!(
                "UPDATE quotes SET subject = $1, description = $2, billing_address = $3,
                    shipping_address = $4, discount_percent = $5, discount_amount = $6,
                    tax_amount = $7, adjustment_amount = $8, sub_total = $9, grand_total = $10,
                    expired_at = $11, person_id = $12, user_id = $13, updated_at = NOW()
                 WHERE id = $14{vp} RETURNING *"
            ))
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
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "quotes.delete") { return resp; }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let vp = view_permission_filter(user.id, &user.view_permission).replace("t.user_id", "user_id");
    let result = guard
        .execute(sqlx::query(&format!("DELETE FROM quotes WHERE id = $1{vp}")).bind(id))
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
    if let Err(resp) = bouncer(&user, "quotes.delete") { return resp; }
    if payload.ids.is_empty() {
        return Json(serde_json::json!({ "message": "No quotes selected.", "deleted_count": 0 })).into_response();
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
        .execute(sqlx::query(&format!("DELETE FROM quotes WHERE id = ANY($1::bigint[]){vp}")).bind(&payload.ids))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(r) => {
            let count = r.rows_affected();
            Json(serde_json::json!({ "message": format!("{count} quote(s) deleted."), "deleted_count": count })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to mass delete quotes: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Failed to delete quotes." }))).into_response()
        }
    }
}

// --- PDF Export ---

pub async fn download_pdf(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "quotes") { return resp; }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let vp = view_permission_filter(user.id, &user.view_permission).replace("t.user_id", "user_id");
    let quote = guard
        .fetch_optional(sqlx::query_as::<_, Quote>(&format!(
            "SELECT * FROM quotes WHERE id = $1{vp}"
        )).bind(id))
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

    // Fetch person name
    let person_name: Option<String> = if let Ok(Some(ref q)) = quote {
        if let Some(pid) = q.person_id {
            guard
                .fetch_optional(sqlx::query_as::<_, (String,)>(
                    "SELECT name FROM persons WHERE id = $1",
                ).bind(pid))
                .await
                .ok()
                .flatten()
                .map(|(n,)| n)
        } else {
            None
        }
    } else {
        None
    };

    let _ = guard.release().await;

    let quote = match quote {
        Ok(Some(q)) => q,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Quote not found." })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to fetch quote for PDF: {e}");
            return internal_error();
        }
    };

    match generate_quote_pdf(&quote, &items, &company.name, person_name.as_deref()) {
        Ok(bytes) => {
            let filename = format!("quote-{}.pdf", quote.id);
            (
                [
                    (axum::http::header::CONTENT_TYPE, "application/pdf".to_string()),
                    (
                        axum::http::header::CONTENT_DISPOSITION,
                        format!("attachment; filename=\"{filename}\""),
                    ),
                ],
                bytes,
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to generate PDF: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to generate PDF." })),
            )
                .into_response()
        }
    }
}

fn generate_quote_pdf(
    quote: &Quote,
    items: &[QuoteItem],
    company_name: &str,
    person_name: Option<&str>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use genpdfi::elements::{Paragraph, TableLayout, Break};
    use genpdfi::style::Style;
    use genpdfi::{Alignment, Document, Element, Margins};

    let font_family = genpdfi::fonts::from_files("fonts", "NotoSans", None)?;
    let mut doc = Document::new(font_family);
    doc.set_title(format!("Quote #{}", quote.id));
    doc.set_paper_size(genpdfi::PaperSize::A4);

    let mut decorator = genpdfi::SimplePageDecorator::new();
    decorator.set_margins(Margins::trbl(20, 15, 20, 15));
    doc.set_page_decorator(decorator);

    let title_style = Style::new().bold().with_font_size(20);
    let heading_style = Style::new().bold().with_font_size(12);
    let bold_style = Style::new().bold();

    // Company name
    doc.push(Paragraph::new(company_name).aligned(Alignment::Left).styled(title_style));
    doc.push(Break::new(1.5));

    // Quote header
    doc.push(Paragraph::new(format!("Quote #{}", quote.id)).styled(heading_style));
    doc.push(Paragraph::new(format!("Subject: {}", quote.subject)));
    if let Some(name) = person_name {
        doc.push(Paragraph::new(format!("Contact: {}", name)));
    }
    doc.push(Paragraph::new(format!("Date: {}", quote.created_at.format("%Y-%m-%d"))));
    if let Some(ref exp) = quote.expired_at {
        doc.push(Paragraph::new(format!("Expires: {}", exp.format("%Y-%m-%d"))));
    }
    if let Some(ref desc) = quote.description {
        doc.push(Break::new(0.5));
        doc.push(Paragraph::new(desc.as_str()));
    }
    doc.push(Break::new(1.5));

    // Address section
    if let Some(ref addr) = quote.billing_address {
        doc.push(Paragraph::new("Billing Address").styled(bold_style));
        if let Some(obj) = addr.as_object() {
            let parts: Vec<String> = ["address", "city", "state", "postcode", "country"]
                .iter()
                .filter_map(|k| obj.get(*k).and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(String::from))
                .collect();
            if !parts.is_empty() {
                doc.push(Paragraph::new(parts.join(", ")));
            }
        }
        doc.push(Break::new(0.5));
    }

    if let Some(ref addr) = quote.shipping_address {
        doc.push(Paragraph::new("Shipping Address").styled(bold_style));
        if let Some(obj) = addr.as_object() {
            let parts: Vec<String> = ["address", "city", "state", "postcode", "country"]
                .iter()
                .filter_map(|k| obj.get(*k).and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(String::from))
                .collect();
            if !parts.is_empty() {
                doc.push(Paragraph::new(parts.join(", ")));
            }
        }
        doc.push(Break::new(0.5));
    }

    // Line items table
    doc.push(Paragraph::new("Line Items").styled(heading_style));
    doc.push(Break::new(0.5));

    let mut table = TableLayout::new(vec![1, 3, 1, 1, 1, 1]);
    table.set_cell_decorator(genpdfi::elements::FrameCellDecorator::new(true, true, false));

    // Header row
    table.row()
        .element(Paragraph::new("#").styled(bold_style))
        .element(Paragraph::new("Item").styled(bold_style))
        .element(Paragraph::new("Qty").aligned(Alignment::Center).styled(bold_style))
        .element(Paragraph::new("Price").aligned(Alignment::Right).styled(bold_style))
        .element(Paragraph::new("Disc %").aligned(Alignment::Right).styled(bold_style))
        .element(Paragraph::new("Total").aligned(Alignment::Right).styled(bold_style))
        .push()
        .ok();

    for (i, item) in items.iter().enumerate() {
        let name = item.name.as_deref().unwrap_or("-");
        let disc = item.discount_percent.map(|d| format!("{}%", d)).unwrap_or_default();
        table.row()
            .element(Paragraph::new(format!("{}", i + 1)))
            .element(Paragraph::new(name))
            .element(Paragraph::new(format!("{}", item.quantity)).aligned(Alignment::Center))
            .element(Paragraph::new(format!("${}", item.price)).aligned(Alignment::Right))
            .element(Paragraph::new(disc).aligned(Alignment::Right))
            .element(Paragraph::new(format!("${}", item.total)).aligned(Alignment::Right))
            .push()
            .ok();
    }

    doc.push(table);
    doc.push(Break::new(1.0));

    // Totals
    if let Some(ref sub) = quote.sub_total {
        doc.push(Paragraph::new(format!("Subtotal: ${sub}")).aligned(Alignment::Right));
    }
    if let Some(ref dp) = quote.discount_percent {
        if *dp > Decimal::ZERO {
            let da = quote.discount_amount.unwrap_or_default();
            doc.push(Paragraph::new(format!("Discount ({dp}%): -${da}")).aligned(Alignment::Right));
        }
    }
    if let Some(ref tax) = quote.tax_amount {
        if *tax > Decimal::ZERO {
            doc.push(Paragraph::new(format!("Tax: ${tax}")).aligned(Alignment::Right));
        }
    }
    if let Some(ref adj) = quote.adjustment_amount {
        if *adj != Decimal::ZERO {
            doc.push(Paragraph::new(format!("Adjustment: ${adj}")).aligned(Alignment::Right));
        }
    }

    doc.push(Break::new(0.5));
    let grand = quote.grand_total.unwrap_or_default();
    let total_style = Style::new().bold().with_font_size(14);
    doc.push(Paragraph::new(format!("Grand Total: ${grand}")).aligned(Alignment::Right).styled(total_style));

    let mut buf = Vec::new();
    doc.render(&mut buf)?;
    Ok(buf)
}

fn internal_error() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": "An internal error occurred." })),
    )
        .into_response()
}
