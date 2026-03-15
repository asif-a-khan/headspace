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
use crate::models::lead::{
    Lead, LeadKanbanCard, LeadProduct, LeadProductRow, LeadRow, LeadSearchRow,
};
use crate::models::tenant_admin::TenantUser;

use super::contacts::view_permission_filter;
use crate::api::pagination::CountRow;

#[derive(Deserialize)]
pub struct LeadProductInput {
    pub product_id: i64,
    pub quantity: i32,
    pub price: Decimal,
}

#[derive(Deserialize, Validate)]
pub struct LeadPayload {
    #[validate(length(min = 1, message = "Title is required."))]
    pub title: String,
    pub description: Option<String>,
    pub lead_value: Option<Decimal>,
    pub expected_close_date: Option<String>,
    pub person_id: Option<i64>,
    pub lead_source_id: Option<i64>,
    pub lead_type_id: Option<i64>,
    pub lead_pipeline_id: Option<i64>,
    pub lead_pipeline_stage_id: Option<i64>,
    pub user_id: Option<i64>,
    pub products: Option<Vec<LeadProductInput>>,
}

#[derive(Deserialize)]
pub struct StagePayload {
    pub lead_pipeline_stage_id: i64,
    pub lost_reason: Option<String>,
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub pipeline_id: Option<i64>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub sort_field: Option<String>,
    pub sort_dir: Option<String>,
}

pub async fn list(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Query(query): Query<ListQuery>,
) -> Response {
    if let Err(resp) = bouncer(&user, "leads") {
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
    let vp_lead = vp.replace("t.user_id", "l.user_id");
    let pipeline_filter = query
        .pipeline_id
        .map(|pid| format!(" AND l.lead_pipeline_id = {pid}"))
        .unwrap_or_default();

    let search_filter = query
        .search
        .as_deref()
        .filter(|s| s.len() >= 2)
        .map(|s| {
            let escaped = s.replace('\'', "''");
            format!(" AND (l.title ILIKE '%{escaped}%' OR p.name ILIKE '%{escaped}%')")
        })
        .unwrap_or_default();

    let where_clause = format!("WHERE true{vp_lead}{pipeline_filter}{search_filter}");
    let from_clause = "FROM leads l
         LEFT JOIN persons p ON p.id = l.person_id
         LEFT JOIN users u ON u.id = l.user_id
         LEFT JOIN lead_sources ls ON ls.id = l.lead_source_id
         LEFT JOIN lead_types lt ON lt.id = l.lead_type_id
         LEFT JOIN lead_pipeline_stages lps ON lps.id = l.lead_pipeline_stage_id
         LEFT JOIN lead_stages stg ON stg.id = lps.lead_stage_id
         LEFT JOIN lead_pipelines pip ON pip.id = l.lead_pipeline_id";

    // Pagination
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(15).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let allowed_sorts: &[(&str, &str)] = &[
        ("id", "l.id"),
        ("title", "l.title"),
        ("lead_value", "l.lead_value"),
        ("person_name", "p.name"),
        ("created_at", "l.created_at"),
    ];
    let sort_dir = match query.sort_dir.as_deref() {
        Some("asc" | "ASC") => "ASC",
        _ => "DESC",
    };
    let order_col = query
        .sort_field
        .as_deref()
        .and_then(|f| {
            allowed_sorts
                .iter()
                .find(|(name, _)| *name == f)
                .map(|(_, col)| *col)
        })
        .unwrap_or("l.id");
    let order_by = format!("{order_col} {sort_dir}");

    // Count
    let total = guard
        .fetch_one(sqlx::query_as::<_, CountRow>(&format!(
            "SELECT COUNT(*) AS count {from_clause} {where_clause}"
        )))
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    let sql = format!(
        "SELECT l.*,
                p.name AS person_name,
                CONCAT(u.first_name, ' ', u.last_name) AS user_name,
                ls.name AS source_name,
                lt.name AS type_name,
                stg.name AS stage_name,
                pip.name AS pipeline_name,
                CASE WHEN l.status IS NOT NULL THEN NULL
                     ELSE GREATEST(0, (NOW()::date - l.created_at::date) - COALESCE(pip.rotten_days, 30))
                END AS rotten_days
         {from_clause}
         {where_clause}
         ORDER BY {order_by}
         LIMIT {per_page} OFFSET {offset}",
    );

    let leads = guard.fetch_all(sqlx::query_as::<_, LeadRow>(&sql)).await;
    let _ = guard.release().await;

    let last_page = if per_page > 0 {
        (total + per_page - 1) / per_page
    } else {
        1
    };

    match leads {
        Ok(l) => Json(serde_json::json!({
            "data": l,
            "meta": { "total": total, "page": page, "per_page": per_page, "last_page": last_page }
        }))
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to list leads: {e}");
            internal_error()
        }
    }
}

pub async fn kanban(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Query(query): Query<ListQuery>,
) -> Response {
    if let Err(resp) = bouncer(&user, "leads") {
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
    let pipeline_filter = query
        .pipeline_id
        .map(|pid| format!(" AND l.lead_pipeline_id = {pid}"))
        .unwrap_or_default();

    let sql = format!(
        "SELECT l.id, l.title, l.lead_value, l.lead_pipeline_stage_id,
                p.name AS person_name,
                org.name AS organization_name,
                NULLIF(CONCAT(u.first_name, ' ', u.last_name), ' ') AS user_name,
                ls.name AS source_name,
                lt.name AS type_name,
                l.created_at,
                GREATEST(0, (NOW()::date - l.created_at::date) - COALESCE(pip.rotten_days, 30)) AS rotten_days,
                (SELECT json_agg(json_build_object('id', tg.id, 'name', tg.name, 'color', tg.color))
                 FROM lead_tags ltg JOIN tags tg ON tg.id = ltg.tag_id
                 WHERE ltg.lead_id = l.id) AS tags_json
         FROM leads l
         LEFT JOIN persons p ON p.id = l.person_id
         LEFT JOIN organizations org ON org.id = p.organization_id
         LEFT JOIN users u ON u.id = l.user_id
         LEFT JOIN lead_sources ls ON ls.id = l.lead_source_id
         LEFT JOIN lead_types lt ON lt.id = l.lead_type_id
         LEFT JOIN lead_pipelines pip ON pip.id = l.lead_pipeline_id
         WHERE 1=1{vp}{pipeline_filter}
         ORDER BY l.id DESC",
        vp = vp.replace("t.user_id", "l.user_id"),
        pipeline_filter = pipeline_filter,
    );

    let cards = guard
        .fetch_all(sqlx::query_as::<_, LeadKanbanCard>(&sql))
        .await;
    let _ = guard.release().await;

    match cards {
        Ok(c) => Json(serde_json::json!({ "data": c })).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch kanban data: {e}");
            internal_error()
        }
    }
}

pub async fn search(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    if let Err(resp) = bouncer(&user, "leads") {
        return resp;
    }

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
            sqlx::query_as::<_, LeadSearchRow>(
                "SELECT id, title FROM leads WHERE title ILIKE $1 ORDER BY id DESC LIMIT 10",
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
    Json(payload): Json<LeadPayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "leads.create") {
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
    let expected_close: Option<chrono::NaiveDate> = payload
        .expected_close_date
        .as_deref()
        .and_then(|d| d.parse().ok());

    let result = guard
        .fetch_one(
            sqlx::query_as::<_, Lead>(
                "INSERT INTO leads (title, description, lead_value, expected_close_date,
                    person_id, lead_source_id, lead_type_id, lead_pipeline_id,
                    lead_pipeline_stage_id, user_id)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING *",
            )
            .bind(&payload.title)
            .bind(&payload.description)
            .bind(payload.lead_value)
            .bind(expected_close)
            .bind(payload.person_id)
            .bind(payload.lead_source_id)
            .bind(payload.lead_type_id)
            .bind(payload.lead_pipeline_id)
            .bind(payload.lead_pipeline_stage_id)
            .bind(assigned_user),
        )
        .await;

    // Insert products if provided
    if let Ok(ref lead) = result
        && let Some(ref products) = payload.products
    {
        for p in products {
            let amount = p.price * Decimal::from(p.quantity);
            let _ = guard
                .execute(
                    sqlx::query(
                        "INSERT INTO lead_products (lead_id, product_id, quantity, price, amount)
                         VALUES ($1, $2, $3, $4, $5)",
                    )
                    .bind(lead.id)
                    .bind(p.product_id)
                    .bind(p.quantity)
                    .bind(p.price)
                    .bind(amount),
                )
                .await;
        }
    }

    let _ = guard.release().await;

    match result {
        Ok(l) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "data": l, "message": "Lead created successfully." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to create lead: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to create lead." })),
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
    if let Err(resp) = bouncer(&user, "leads.edit") {
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
    let lead = guard
        .fetch_optional(
            sqlx::query_as::<_, Lead>(&format!("SELECT * FROM leads WHERE id = $1{vp}")).bind(id),
        )
        .await;

    let _ = guard.release().await;

    match lead {
        Ok(Some(l)) => Json(serde_json::json!({ "data": l })).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Lead not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch lead: {e}");
            internal_error()
        }
    }
}

pub async fn update(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
    Json(payload): Json<LeadPayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "leads.edit") {
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

    let expected_close: Option<chrono::NaiveDate> = payload
        .expected_close_date
        .as_deref()
        .and_then(|d| d.parse().ok());

    let vp = view_permission_filter(user.id, &user.view_permission).replace("t.user_id", "user_id");
    let result = guard
        .fetch_optional(
            sqlx::query_as::<_, Lead>(&format!(
                "UPDATE leads
                 SET title = $1, description = $2, lead_value = $3, expected_close_date = $4,
                     person_id = $5, lead_source_id = $6, lead_type_id = $7,
                     lead_pipeline_id = $8, lead_pipeline_stage_id = $9, user_id = $10,
                     updated_at = NOW()
                 WHERE id = $11{vp} RETURNING *"
            ))
            .bind(&payload.title)
            .bind(&payload.description)
            .bind(payload.lead_value)
            .bind(expected_close)
            .bind(payload.person_id)
            .bind(payload.lead_source_id)
            .bind(payload.lead_type_id)
            .bind(payload.lead_pipeline_id)
            .bind(payload.lead_pipeline_stage_id)
            .bind(payload.user_id)
            .bind(id),
        )
        .await;

    // Sync products if provided (delete all + re-insert)
    if let Ok(Some(_)) = &result
        && let Some(ref products) = payload.products
    {
        let _ = guard
            .execute(sqlx::query("DELETE FROM lead_products WHERE lead_id = $1").bind(id))
            .await;
        for p in products {
            let amount = p.price * Decimal::from(p.quantity);
            let _ = guard
                .execute(
                    sqlx::query(
                        "INSERT INTO lead_products (lead_id, product_id, quantity, price, amount)
                         VALUES ($1, $2, $3, $4, $5)",
                    )
                    .bind(id)
                    .bind(p.product_id)
                    .bind(p.quantity)
                    .bind(p.price)
                    .bind(amount),
                )
                .await;
        }
    }

    let _ = guard.release().await;

    match result {
        Ok(Some(l)) => {
            Json(serde_json::json!({ "data": l, "message": "Lead updated successfully." }))
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Lead not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to update lead: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to update lead." })),
            )
                .into_response()
        }
    }
}

pub async fn update_stage(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
    Json(payload): Json<StagePayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "leads.edit") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    // Look up the target stage's code to determine won/lost/normal
    let stage_code = guard
        .fetch_optional(
            sqlx::query_as::<_, (String,)>(
                "SELECT ls.code FROM lead_pipeline_stages lps
             JOIN lead_stages ls ON ls.id = lps.lead_stage_id
             WHERE lps.id = $1",
            )
            .bind(payload.lead_pipeline_stage_id),
        )
        .await;

    let stage_code = match stage_code {
        Ok(Some((code,))) => code,
        Ok(None) => {
            let _ = guard.release().await;
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Invalid stage." })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to look up stage code: {e}");
            let _ = guard.release().await;
            return internal_error();
        }
    };

    // Derive status fields from stage code (mirrors Krayin's LeadRepository behavior)
    let (status_val, lost_reason, closed_at_expr): (Option<bool>, Option<String>, &str) =
        match stage_code.as_str() {
            "won" => (Some(true), None, "NOW()"),
            "lost" => (Some(false), payload.lost_reason, "NOW()"),
            _ => (None, None, "NULL"),
        };

    let vp = view_permission_filter(user.id, &user.view_permission).replace("t.user_id", "user_id");
    let result = guard
        .fetch_optional(
            sqlx::query_as::<_, Lead>(&format!(
                "UPDATE leads SET lead_pipeline_stage_id = $1, status = $2, lost_reason = $3,
                 closed_at = {closed_at_expr}, updated_at = NOW()
                 WHERE id = $4{vp} RETURNING *"
            ))
            .bind(payload.lead_pipeline_stage_id)
            .bind(status_val)
            .bind(&lost_reason)
            .bind(id),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(Some(l)) => {
            Json(serde_json::json!({ "data": l, "message": "Stage updated." })).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Lead not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to update lead stage: {e}");
            internal_error()
        }
    }
}

#[derive(Deserialize)]
pub struct StatusPayload {
    pub status: String,
    pub lost_reason: Option<String>,
}

pub async fn update_status(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
    Json(payload): Json<StatusPayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "leads.edit") {
        return resp;
    }

    let (status_val, lost_reason, set_closed): (Option<bool>, Option<String>, bool) = match payload
        .status
        .as_str()
    {
        "won" => (Some(true), None, true),
        "lost" => (Some(false), payload.lost_reason, true),
        "open" => (None, None, false),
        _ => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Status must be 'won', 'lost', or 'open'." })),
            )
                .into_response();
        }
    };

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let vp = view_permission_filter(user.id, &user.view_permission).replace("t.user_id", "user_id");
    let closed_at_expr = if set_closed { "NOW()" } else { "NULL" };

    // Also move to the won/lost pipeline stage if marking won/lost (mirrors Krayin)
    let stage_update = if let Some(sv) = status_val {
        let target_code = if sv { "won" } else { "lost" };
        guard
            .fetch_optional(
                sqlx::query_as::<_, (i64,)>(
                    "SELECT lps.id FROM lead_pipeline_stages lps
                 JOIN lead_stages ls ON ls.id = lps.lead_stage_id
                 WHERE lps.lead_pipeline_id = (SELECT lead_pipeline_id FROM leads WHERE id = $1)
                 AND ls.code = $2",
                )
                .bind(id)
                .bind(target_code),
            )
            .await
            .ok()
            .flatten()
            .map(|(sid,)| sid)
    } else {
        None
    };

    let stage_clause = if let Some(sid) = stage_update {
        format!(", lead_pipeline_stage_id = {sid}")
    } else {
        String::new()
    };

    let result = guard
        .fetch_optional(
            sqlx::query_as::<_, Lead>(&format!(
                "UPDATE leads SET status = $1, lost_reason = $2, closed_at = {closed_at_expr}{stage_clause}, updated_at = NOW()
                 WHERE id = $3{vp} RETURNING *"
            ))
            .bind(status_val)
            .bind(&lost_reason)
            .bind(id),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(Some(l)) => Json(serde_json::json!({ "data": l, "message": "Lead status updated." }))
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Lead not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to update lead status: {e}");
            internal_error()
        }
    }
}

pub async fn destroy(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "leads.delete") {
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
        .execute(sqlx::query(&format!("DELETE FROM leads WHERE id = $1{vp}")).bind(id))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            Json(serde_json::json!({ "message": "Lead deleted successfully." })).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Lead not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to delete lead: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to delete lead." })),
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
    if let Err(resp) = bouncer(&user, "leads.delete") {
        return resp;
    }
    if payload.ids.is_empty() {
        return Json(serde_json::json!({ "message": "No leads selected.", "deleted_count": 0 }))
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
                "DELETE FROM leads WHERE id = ANY($1::bigint[]){vp}"
            ))
            .bind(&payload.ids),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(r) => {
            let count = r.rows_affected();
            Json(serde_json::json!({ "message": format!("{count} lead(s) deleted."), "deleted_count": count })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to mass delete leads: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to delete leads." })),
            )
                .into_response()
        }
    }
}

// --- Lead Products ---

#[derive(Deserialize)]
pub struct LeadProductPayload {
    pub product_id: i64,
    pub quantity: i32,
    pub price: Decimal,
}

pub async fn list_products(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "leads") {
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
        .fetch_all(
            sqlx::query_as::<_, LeadProductRow>(
                "SELECT lp.id, lp.lead_id, lp.product_id, lp.quantity, lp.price, lp.amount,
                        p.name AS product_name, p.sku AS product_sku
                 FROM lead_products lp
                 JOIN products p ON p.id = lp.product_id
                 WHERE lp.lead_id = $1
                 ORDER BY lp.id",
            )
            .bind(id),
        )
        .await;

    let _ = guard.release().await;

    match products {
        Ok(p) => Json(serde_json::json!({ "data": p })).into_response(),
        Err(e) => {
            tracing::error!("Failed to list lead products: {e}");
            internal_error()
        }
    }
}

pub async fn add_product(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
    Json(payload): Json<LeadProductPayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "leads.edit") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let amount = payload.price * Decimal::from(payload.quantity);
    let result = guard
        .fetch_one(
            sqlx::query_as::<_, LeadProduct>(
                "INSERT INTO lead_products (lead_id, product_id, quantity, price, amount)
                 VALUES ($1, $2, $3, $4, $5) RETURNING *",
            )
            .bind(id)
            .bind(payload.product_id)
            .bind(payload.quantity)
            .bind(payload.price)
            .bind(amount),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(lp) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "data": lp, "message": "Product added to lead." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to add product to lead: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to add product." })),
            )
                .into_response()
        }
    }
}

pub async fn remove_product(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path((lead_id, product_line_id)): Path<(i64, i64)>,
) -> Response {
    if let Err(resp) = bouncer(&user, "leads.edit") {
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
        .execute(
            sqlx::query("DELETE FROM lead_products WHERE id = $1 AND lead_id = $2")
                .bind(product_line_id)
                .bind(lead_id),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            Json(serde_json::json!({ "message": "Product removed from lead." })).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Lead product not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to remove product from lead: {e}");
            internal_error()
        }
    }
}

// --- Lead Quotes ---

pub async fn list_quotes(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "leads") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let quotes = guard
        .fetch_all(
            sqlx::query_as::<_, crate::models::quote::Quote>(
                "SELECT q.* FROM quotes q
                 JOIN lead_quotes lq ON lq.quote_id = q.id
                 WHERE lq.lead_id = $1
                 ORDER BY q.id DESC",
            )
            .bind(id),
        )
        .await;

    let _ = guard.release().await;

    match quotes {
        Ok(q) => Json(serde_json::json!({ "data": q })).into_response(),
        Err(e) => {
            tracing::error!("Failed to list lead quotes: {e}");
            internal_error()
        }
    }
}

#[derive(Deserialize)]
pub struct LeadQuoteLinkPayload {
    pub quote_id: i64,
}

pub async fn link_quote(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
    Json(payload): Json<LeadQuoteLinkPayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "leads.edit") {
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
        .execute(
            sqlx::query(
                "INSERT INTO lead_quotes (lead_id, quote_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(id)
            .bind(payload.quote_id),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "message": "Quote linked to lead." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to link quote to lead: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to link quote." })),
            )
                .into_response()
        }
    }
}

pub async fn unlink_quote(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path((lead_id, quote_id)): Path<(i64, i64)>,
) -> Response {
    if let Err(resp) = bouncer(&user, "leads.edit") {
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
        .execute(
            sqlx::query("DELETE FROM lead_quotes WHERE lead_id = $1 AND quote_id = $2")
                .bind(lead_id)
                .bind(quote_id),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            Json(serde_json::json!({ "message": "Quote unlinked from lead." })).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Link not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to unlink quote from lead: {e}");
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
