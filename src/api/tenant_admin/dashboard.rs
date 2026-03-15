use axum::Json;
use axum::extract::{Extension, Query};
use axum::response::{IntoResponse, Response};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::auth::bouncer::bouncer;
use crate::db::Database;
use crate::db::guard::TenantGuard;
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

fn compute_progress(current: f64, previous: f64) -> f64 {
    if previous > 0.0 {
        ((current - previous) / previous) * 100.0
    } else if current > 0.0 {
        100.0
    } else {
        0.0
    }
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

    // Parse dates for current and previous periods
    let today = chrono::Utc::now().date_naive();
    let start_date = filter
        .start
        .as_ref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or_else(|| today - chrono::Duration::days(30));
    let end_date = filter
        .end
        .as_ref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or(today);

    let period_days = (end_date - start_date).num_days().max(1);
    let prev_end = start_date - chrono::Duration::days(1);
    let prev_start = prev_end - chrono::Duration::days(period_days - 1);

    let start_str = start_date.format("%Y-%m-%d").to_string();
    let end_str = end_date.format("%Y-%m-%d").to_string();
    let prev_start_str = prev_start.format("%Y-%m-%d").to_string();
    let prev_end_str = prev_end.format("%Y-%m-%d").to_string();

    // Build date filter clauses
    let date_clause = format!(
        " AND l.created_at >= '{start_str}'::date AND l.created_at < ('{end_str}'::date + interval '1 day')"
    );
    let prev_date_clause = format!(
        " AND l.created_at >= '{prev_start_str}'::date AND l.created_at < ('{prev_end_str}'::date + interval '1 day')"
    );

    // --- Current period KPIs ---

    let total_leads = guard
        .fetch_one(sqlx::query_as::<_, CountRow>(&format!(
            "SELECT COUNT(*) AS count FROM leads l WHERE true{vp_lead}{date_clause}"
        )))
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    let won_value = guard
        .fetch_one(sqlx::query_as::<_, SumRow>(&format!(
            "SELECT COALESCE(SUM(l.lead_value), 0) AS total FROM leads l WHERE l.status = true{vp_lead}{date_clause}"
        )))
        .await
        .map(|r| r.total.map(|d| d.to_string()).unwrap_or_else(|| "0".to_string()))
        .unwrap_or_else(|_| "0".to_string());

    let lost_value = guard
        .fetch_one(sqlx::query_as::<_, SumRow>(&format!(
            "SELECT COALESCE(SUM(l.lead_value), 0) AS total FROM leads l WHERE l.status = false{vp_lead}{date_clause}"
        )))
        .await
        .map(|r| r.total.map(|d| d.to_string()).unwrap_or_else(|| "0".to_string()))
        .unwrap_or_else(|_| "0".to_string());

    let avg_lead_value = guard
        .fetch_one(sqlx::query_as::<_, AvgRow>(&format!(
            "SELECT AVG(l.lead_value) AS avg FROM leads l WHERE l.lead_value > 0{vp_lead}{date_clause}"
        )))
        .await
        .map(|r| r.avg.map(|d| d.to_string()).unwrap_or_else(|| "0".to_string()))
        .unwrap_or_else(|_| "0".to_string());

    let total_quotes = guard
        .fetch_one(sqlx::query_as::<_, CountRow>(
            "SELECT COUNT(*) AS count FROM quotes",
        ))
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    let total_persons = guard
        .fetch_one(sqlx::query_as::<_, CountRow>(
            "SELECT COUNT(*) AS count FROM persons",
        ))
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    let total_organizations = guard
        .fetch_one(sqlx::query_as::<_, CountRow>(
            "SELECT COUNT(*) AS count FROM organizations",
        ))
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    // --- Previous period KPIs ---

    let prev_total_leads = guard
        .fetch_one(sqlx::query_as::<_, CountRow>(&format!(
            "SELECT COUNT(*) AS count FROM leads l WHERE true{vp_lead}{prev_date_clause}"
        )))
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    let prev_won_value: f64 = guard
        .fetch_one(sqlx::query_as::<_, SumRow>(&format!(
            "SELECT COALESCE(SUM(l.lead_value), 0) AS total FROM leads l WHERE l.status = true{vp_lead}{prev_date_clause}"
        )))
        .await
        .map(|r| r.total.map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0))
        .unwrap_or(0.0);

    let prev_lost_value: f64 = guard
        .fetch_one(sqlx::query_as::<_, SumRow>(&format!(
            "SELECT COALESCE(SUM(l.lead_value), 0) AS total FROM leads l WHERE l.status = false{vp_lead}{prev_date_clause}"
        )))
        .await
        .map(|r| r.total.map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0))
        .unwrap_or(0.0);

    let prev_avg_lead_value: f64 = guard
        .fetch_one(sqlx::query_as::<_, AvgRow>(&format!(
            "SELECT AVG(l.lead_value) AS avg FROM leads l WHERE l.lead_value > 0{vp_lead}{prev_date_clause}"
        )))
        .await
        .map(|r| r.avg.map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0))
        .unwrap_or(0.0);

    let prev_total_quotes = guard
        .fetch_one(sqlx::query_as::<_, CountRow>(&format!(
            "SELECT COUNT(*) AS count FROM quotes WHERE created_at >= '{prev_start_str}'::date AND created_at < ('{prev_end_str}'::date + interval '1 day')"
        )))
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    let prev_total_persons = guard
        .fetch_one(sqlx::query_as::<_, CountRow>(&format!(
            "SELECT COUNT(*) AS count FROM persons WHERE created_at >= '{prev_start_str}'::date AND created_at < ('{prev_end_str}'::date + interval '1 day')"
        )))
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    let prev_total_organizations = guard
        .fetch_one(sqlx::query_as::<_, CountRow>(&format!(
            "SELECT COUNT(*) AS count FROM organizations WHERE created_at >= '{prev_start_str}'::date AND created_at < ('{prev_end_str}'::date + interval '1 day')"
        )))
        .await
        .map(|r| r.count.unwrap_or(0))
        .unwrap_or(0);

    // Compute progress percentages
    let won_val_f: f64 = won_value.parse().unwrap_or(0.0);
    let lost_val_f: f64 = lost_value.parse().unwrap_or(0.0);
    let avg_val_f: f64 = avg_lead_value.parse().unwrap_or(0.0);

    let avg_leads_per_day = total_leads as f64 / period_days as f64;
    let prev_avg_leads_per_day = prev_total_leads as f64 / period_days as f64;

    // --- Chart data (unchanged) ---

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
        "total_leads": {
            "current": total_leads,
            "previous": prev_total_leads,
            "progress": compute_progress(total_leads as f64, prev_total_leads as f64),
        },
        "avg_lead_value": {
            "current": avg_lead_value,
            "previous": prev_avg_lead_value,
            "progress": compute_progress(avg_val_f, prev_avg_lead_value),
        },
        "avg_leads_per_day": {
            "current": format!("{:.1}", avg_leads_per_day),
            "previous": format!("{:.1}", prev_avg_leads_per_day),
            "progress": compute_progress(avg_leads_per_day, prev_avg_leads_per_day),
        },
        "total_quotations": {
            "current": total_quotes,
            "previous": prev_total_quotes,
            "progress": compute_progress(total_quotes as f64, prev_total_quotes as f64),
        },
        "total_persons": {
            "current": total_persons,
            "previous": prev_total_persons,
            "progress": compute_progress(total_persons as f64, prev_total_persons as f64),
        },
        "total_organizations": {
            "current": total_organizations,
            "previous": prev_total_organizations,
            "progress": compute_progress(total_organizations as f64, prev_total_organizations as f64),
        },
        "won_revenue": {
            "current": won_value,
            "progress": compute_progress(won_val_f, prev_won_value),
        },
        "lost_revenue": {
            "current": lost_value,
            "progress": compute_progress(lost_val_f, prev_lost_value),
        },
        "leads_by_stage": leads_by_stage,
        "revenue_by_source": revenue_by_source,
        "revenue_by_type": revenue_by_type,
        "leads_over_time": leads_over_time,
        "top_products": top_products,
        "top_persons": top_persons,
    }))
    .into_response()
}
