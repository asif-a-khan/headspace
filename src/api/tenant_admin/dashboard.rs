use axum::extract::{Extension, Query};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::auth::bouncer::bouncer;
use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::models::company::Company;
use crate::models::tenant_admin::TenantUser;

use super::contacts::view_permission_filter;

#[derive(Deserialize)]
pub struct DateFilter {
    pub start: Option<String>, // YYYY-MM-DD
    pub end: Option<String>,   // YYYY-MM-DD
}

#[derive(sqlx::FromRow, Serialize)]
struct CountRow {
    count: Option<i64>,
}

#[derive(sqlx::FromRow, Serialize)]
struct SumRow {
    total: Option<rust_decimal::Decimal>,
}

#[derive(sqlx::FromRow, Serialize)]
struct AvgRow {
    avg: Option<rust_decimal::Decimal>,
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

#[derive(sqlx::FromRow, Serialize)]
struct TypeRevenue {
    type_name: Option<String>,
    total: Option<rust_decimal::Decimal>,
}

#[derive(sqlx::FromRow, Serialize)]
struct LeadsOverTime {
    period: Option<String>,
    total: Option<i64>,
    won: Option<i64>,
    lost: Option<i64>,
}

#[derive(sqlx::FromRow, Serialize)]
struct TopProduct {
    name: Option<String>,
    sku: Option<String>,
    total_revenue: Option<rust_decimal::Decimal>,
    total_qty: Option<i64>,
}

#[derive(sqlx::FromRow, Serialize)]
struct TopPerson {
    id: Option<i64>,
    name: Option<String>,
    email: Option<String>,
    total_leads: Option<i64>,
}

pub async fn stats(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Query(filter): Query<DateFilter>,
) -> Response {
    if let Err(resp) = bouncer(&user, "dashboard") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return Json(serde_json::json!({})).into_response();
        }
    };

    let vp = view_permission_filter(user.id, &user.view_permission);
    let vp_lead = vp.replace("t.user_id", "l.user_id");

    // Build date filter clause for leads.created_at
    let date_clause = match (&filter.start, &filter.end) {
        (Some(s), Some(e)) => format!(" AND l.created_at >= '{s}'::date AND l.created_at < ('{e}'::date + interval '1 day')"),
        (Some(s), None) => format!(" AND l.created_at >= '{s}'::date"),
        (None, Some(e)) => format!(" AND l.created_at < ('{e}'::date + interval '1 day')"),
        (None, None) => String::new(),
    };

    // Total leads
    let total_leads = guard
        .fetch_one(sqlx::query_as::<_, CountRow>(&format!(
            "SELECT COUNT(*) AS count FROM leads l WHERE true{vp_lead}{date_clause}"
        )))
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    // Open leads (status IS NULL)
    let open_leads = guard
        .fetch_one(sqlx::query_as::<_, CountRow>(&format!(
            "SELECT COUNT(*) AS count FROM leads l WHERE l.status IS NULL{vp_lead}{date_clause}"
        )))
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    // Won deals count and value
    let won_count = guard
        .fetch_one(sqlx::query_as::<_, CountRow>(&format!(
            "SELECT COUNT(*) AS count FROM leads l WHERE l.status = true{vp_lead}{date_clause}"
        )))
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    let won_value = guard
        .fetch_one(sqlx::query_as::<_, SumRow>(&format!(
            "SELECT COALESCE(SUM(l.lead_value), 0) AS total FROM leads l WHERE l.status = true{vp_lead}{date_clause}"
        )))
        .await
        .map(|r| r.total)
        .unwrap_or(None);

    // Lost deals count and value
    let lost_count = guard
        .fetch_one(sqlx::query_as::<_, CountRow>(&format!(
            "SELECT COUNT(*) AS count FROM leads l WHERE l.status = false{vp_lead}{date_clause}"
        )))
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    let lost_value = guard
        .fetch_one(sqlx::query_as::<_, SumRow>(&format!(
            "SELECT COALESCE(SUM(l.lead_value), 0) AS total FROM leads l WHERE l.status = false{vp_lead}{date_clause}"
        )))
        .await
        .map(|r| r.total)
        .unwrap_or(None);

    // Open revenue
    let open_revenue = guard
        .fetch_one(sqlx::query_as::<_, SumRow>(&format!(
            "SELECT COALESCE(SUM(l.lead_value), 0) AS total FROM leads l WHERE l.status IS NULL{vp_lead}{date_clause}"
        )))
        .await
        .map(|r| r.total)
        .unwrap_or(None);

    // Average lead value
    let avg_lead_value = guard
        .fetch_one(sqlx::query_as::<_, AvgRow>(&format!(
            "SELECT AVG(l.lead_value) AS avg FROM leads l WHERE l.lead_value > 0{vp_lead}{date_clause}"
        )))
        .await
        .map(|r| r.avg)
        .unwrap_or(None);

    // Total quotes
    let total_quotes = guard
        .fetch_one(sqlx::query_as::<_, CountRow>(
            "SELECT COUNT(*) AS count FROM quotes",
        ))
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    // Total persons
    let total_persons = guard
        .fetch_one(sqlx::query_as::<_, CountRow>(
            "SELECT COUNT(*) AS count FROM persons",
        ))
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    // Total organizations
    let total_organizations = guard
        .fetch_one(sqlx::query_as::<_, CountRow>(
            "SELECT COUNT(*) AS count FROM organizations",
        ))
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

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

    // Leads by stage (pipeline funnel)
    let leads_by_stage = guard
        .fetch_all(sqlx::query_as::<_, StageCount>(&format!(
            "SELECT s.name AS stage_name, COUNT(l.id) AS count
             FROM leads l
             JOIN lead_pipeline_stages lps ON lps.id = l.lead_pipeline_stage_id
             JOIN lead_stages s ON s.id = lps.lead_stage_id
             WHERE l.status IS NULL{vp_lead}{date_clause}
             GROUP BY s.name, lps.sort_order
             ORDER BY lps.sort_order",
        )))
        .await
        .unwrap_or_default();

    // Revenue by source (doughnut)
    let revenue_by_source = guard
        .fetch_all(sqlx::query_as::<_, SourceRevenue>(&format!(
            "SELECT ls.name AS source_name, COALESCE(SUM(l.lead_value), 0) AS total
             FROM leads l
             JOIN lead_sources ls ON ls.id = l.lead_source_id
             WHERE l.status = true{vp_lead}{date_clause}
             GROUP BY ls.name
             ORDER BY total DESC",
        )))
        .await
        .unwrap_or_default();

    // Revenue by type (doughnut)
    let revenue_by_type = guard
        .fetch_all(sqlx::query_as::<_, TypeRevenue>(&format!(
            "SELECT lt.name AS type_name, COALESCE(SUM(l.lead_value), 0) AS total
             FROM leads l
             JOIN lead_types lt ON lt.id = l.lead_type_id
             WHERE l.status = true{vp_lead}{date_clause}
             GROUP BY lt.name
             ORDER BY total DESC",
        )))
        .await
        .unwrap_or_default();

    // Leads over time (weekly buckets for chart)
    let leads_over_time = guard
        .fetch_all(sqlx::query_as::<_, LeadsOverTime>(&format!(
            "SELECT
               TO_CHAR(date_trunc('week', l.created_at), 'Mon DD') AS period,
               COUNT(*) AS total,
               COUNT(*) FILTER (WHERE l.status = true) AS won,
               COUNT(*) FILTER (WHERE l.status = false) AS lost
             FROM leads l
             WHERE true{vp_lead}{date_clause}
             GROUP BY date_trunc('week', l.created_at)
             ORDER BY date_trunc('week', l.created_at)",
        )))
        .await
        .unwrap_or_default();

    // Top products (by total revenue in quotes)
    let top_products = guard
        .fetch_all(sqlx::query_as::<_, TopProduct>(
            "SELECT p.name, p.sku,
               COALESCE(SUM(qi.total), 0) AS total_revenue,
               COALESCE(SUM(qi.quantity), 0)::bigint AS total_qty
             FROM quote_items qi
             JOIN products p ON p.id = qi.product_id
             GROUP BY p.id, p.name, p.sku
             ORDER BY total_revenue DESC
             LIMIT 5",
        ))
        .await
        .unwrap_or_default();

    // Top persons (by number of leads)
    let top_persons = guard
        .fetch_all(sqlx::query_as::<_, TopPerson>(&format!(
            "SELECT p.id, p.name,
               (SELECT pe.value FROM person_emails pe WHERE pe.person_id = p.id LIMIT 1) AS email,
               COUNT(l.id) AS total_leads
             FROM persons p
             JOIN leads l ON l.person_id = p.id
             WHERE true{vp_lead}{date_clause}
             GROUP BY p.id, p.name
             ORDER BY total_leads DESC
             LIMIT 5",
        )))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    Json(serde_json::json!({
        "total_leads": total_leads,
        "open_leads": open_leads,
        "won_deals": { "count": won_count, "value": won_value },
        "lost_deals": { "count": lost_count, "value": lost_value },
        "open_revenue": open_revenue,
        "avg_lead_value": avg_lead_value,
        "total_quotes": total_quotes,
        "total_persons": total_persons,
        "total_organizations": total_organizations,
        "activities_due": activities_due,
        "leads_by_stage": leads_by_stage,
        "revenue_by_source": revenue_by_source,
        "revenue_by_type": revenue_by_type,
        "leads_over_time": leads_over_time,
        "top_products": top_products,
        "top_persons": top_persons,
    }))
    .into_response()
}
