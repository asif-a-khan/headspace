use axum::extract::Extension;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::models::company::Company;
use crate::models::tenant_admin::TenantUser;

use super::contacts::view_permission_filter;

#[derive(sqlx::FromRow, Serialize)]
struct CountRow {
    count: Option<i64>,
}

#[derive(sqlx::FromRow, Serialize)]
struct SumRow {
    total: Option<rust_decimal::Decimal>,
}

#[derive(sqlx::FromRow, Serialize)]
struct StageCount {
    stage_name: Option<String>,
    count: Option<i64>,
}

#[derive(sqlx::FromRow, Serialize)]
struct SourceRevenue {
    source_name: Option<String>,
    total: Option<rust_decimal::Decimal>,
}

pub async fn stats(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return Json(serde_json::json!({})).into_response();
        }
    };

    let vp = view_permission_filter(user.id, &user.view_permission);
    let vp_lead = vp.replace("t.user_id", "l.user_id");

    // Total leads
    let total_leads = guard
        .fetch_one(sqlx::query_as::<_, CountRow>(&format!(
            "SELECT COUNT(*) AS count FROM leads l WHERE true{vp_lead}"
        )))
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    // Open leads (status IS NULL)
    let open_leads = guard
        .fetch_one(sqlx::query_as::<_, CountRow>(&format!(
            "SELECT COUNT(*) AS count FROM leads l WHERE l.status IS NULL{vp_lead}"
        )))
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    // Won deals count and value
    let won_count = guard
        .fetch_one(sqlx::query_as::<_, CountRow>(&format!(
            "SELECT COUNT(*) AS count FROM leads l WHERE l.status = true{vp_lead}"
        )))
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    let won_value = guard
        .fetch_one(sqlx::query_as::<_, SumRow>(&format!(
            "SELECT COALESCE(SUM(l.lead_value), 0) AS total FROM leads l WHERE l.status = true{vp_lead}"
        )))
        .await
        .map(|r| r.total)
        .unwrap_or(None);

    // Open revenue
    let open_revenue = guard
        .fetch_one(sqlx::query_as::<_, SumRow>(&format!(
            "SELECT COALESCE(SUM(l.lead_value), 0) AS total FROM leads l WHERE l.status IS NULL{vp_lead}"
        )))
        .await
        .map(|r| r.total)
        .unwrap_or(None);

    // Activities due today
    let activities_due = guard
        .fetch_one(sqlx::query_as::<_, CountRow>(
            "SELECT COUNT(*) AS count FROM activities
             WHERE is_done = false AND schedule_from IS NOT NULL
               AND schedule_from::date = CURRENT_DATE",
        ))
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    // Leads by stage
    let leads_by_stage = guard
        .fetch_all(sqlx::query_as::<_, StageCount>(&format!(
            "SELECT s.name AS stage_name, COUNT(l.id) AS count
             FROM leads l
             JOIN lead_pipeline_stages lps ON lps.id = l.lead_pipeline_stage_id
             JOIN lead_stages s ON s.id = lps.lead_stage_id
             WHERE l.status IS NULL{vp_lead}
             GROUP BY s.name, lps.sort_order
             ORDER BY lps.sort_order",
            vp_lead = vp_lead,
        )))
        .await
        .unwrap_or_default();

    // Revenue by source
    let revenue_by_source = guard
        .fetch_all(sqlx::query_as::<_, SourceRevenue>(&format!(
            "SELECT ls.name AS source_name, COALESCE(SUM(l.lead_value), 0) AS total
             FROM leads l
             JOIN lead_sources ls ON ls.id = l.lead_source_id
             WHERE l.status = true{vp_lead}
             GROUP BY ls.name
             ORDER BY total DESC",
            vp_lead = vp_lead,
        )))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    Json(serde_json::json!({
        "total_leads": total_leads,
        "open_leads": open_leads,
        "won_deals": { "count": won_count, "value": won_value },
        "open_revenue": open_revenue,
        "activities_due": activities_due,
        "leads_by_stage": leads_by_stage,
        "revenue_by_source": revenue_by_source,
    }))
    .into_response()
}
