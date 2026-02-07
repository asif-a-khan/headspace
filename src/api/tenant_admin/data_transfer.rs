use axum::extract::{Extension, Multipart};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::auth::bouncer::bouncer;
use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::models::company::Company;
use crate::models::tenant_admin::TenantUser;

use super::contacts::view_permission_filter;

// ── CSV Export Row structs (flat, no Option for cleaner CSV) ──

#[derive(serde::Serialize)]
struct LeadCsvRow {
    id: i64,
    title: String,
    description: String,
    lead_value: String,
    status: String,
    contact_person: String,
    source: String,
    lead_type: String,
    pipeline: String,
    stage: String,
    assigned_to: String,
    expected_close_date: String,
    created_at: String,
}

#[derive(serde::Serialize)]
struct PersonCsvRow {
    id: i64,
    name: String,
    email: String,
    phone: String,
    job_title: String,
    organization: String,
    assigned_to: String,
    created_at: String,
}

#[derive(serde::Serialize)]
struct OrganizationCsvRow {
    id: i64,
    name: String,
    assigned_to: String,
    created_at: String,
}

#[derive(serde::Serialize)]
struct ProductCsvRow {
    id: i64,
    sku: String,
    name: String,
    description: String,
    price: String,
    quantity: i32,
    created_at: String,
}

// ── sqlx Row structs for export queries ──

#[derive(sqlx::FromRow)]
struct LeadExportRow {
    id: i64,
    title: String,
    description: Option<String>,
    lead_value: Option<rust_decimal::Decimal>,
    status: Option<bool>,
    expected_close_date: Option<chrono::NaiveDate>,
    created_at: chrono::DateTime<chrono::Utc>,
    person_name: Option<String>,
    user_name: Option<String>,
    source_name: Option<String>,
    type_name: Option<String>,
    pipeline_name: Option<String>,
    stage_name: Option<String>,
}

#[derive(sqlx::FromRow)]
struct PersonExportRow {
    id: i64,
    name: String,
    emails: serde_json::Value,
    contact_numbers: serde_json::Value,
    job_title: Option<String>,
    organization_name: Option<String>,
    user_name: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
struct OrgExportRow {
    id: i64,
    name: String,
    user_name: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
struct ProductExportRow {
    id: i64,
    sku: String,
    name: String,
    description: Option<String>,
    price: rust_decimal::Decimal,
    quantity: i32,
    created_at: chrono::DateTime<chrono::Utc>,
}

// ── Helpers ──

fn csv_response(data: Vec<u8>, filename: &str) -> Response {
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        data,
    )
        .into_response()
}

fn internal_error() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": "An internal error occurred." })),
    )
        .into_response()
}

fn first_json_value(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::Array(arr) => arr
            .first()
            .and_then(|v| v.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        _ => String::new(),
    }
}

// ── EXPORT endpoints ──

pub async fn export_leads(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
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

    let sql = format!(
        "SELECT l.id, l.title, l.description, l.lead_value, l.status,
                l.expected_close_date, l.created_at,
                p.name AS person_name,
                CONCAT(u.first_name, ' ', u.last_name) AS user_name,
                ls.name AS source_name,
                lt.name AS type_name,
                pip.name AS pipeline_name,
                stg.name AS stage_name
         FROM leads l
         LEFT JOIN persons p ON p.id = l.person_id
         LEFT JOIN users u ON u.id = l.user_id
         LEFT JOIN lead_sources ls ON ls.id = l.lead_source_id
         LEFT JOIN lead_types lt ON lt.id = l.lead_type_id
         LEFT JOIN lead_pipeline_stages lps ON lps.id = l.lead_pipeline_stage_id
         LEFT JOIN lead_stages stg ON stg.id = lps.lead_stage_id
         LEFT JOIN lead_pipelines pip ON pip.id = l.lead_pipeline_id
         WHERE true{vp_lead}
         ORDER BY l.id DESC"
    );

    let rows = guard
        .fetch_all(sqlx::query_as::<_, LeadExportRow>(&sql))
        .await;
    let _ = guard.release().await;

    let rows = match rows {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to export leads: {e}");
            return internal_error();
        }
    };

    let mut wtr = csv::Writer::from_writer(Vec::new());
    for row in &rows {
        let status_str = match row.status {
            None => "Open".to_string(),
            Some(true) => "Won".to_string(),
            Some(false) => "Lost".to_string(),
        };
        let csv_row = LeadCsvRow {
            id: row.id,
            title: row.title.clone(),
            description: row.description.clone().unwrap_or_default(),
            lead_value: row
                .lead_value
                .map(|v| v.to_string())
                .unwrap_or_default(),
            status: status_str,
            contact_person: row.person_name.clone().unwrap_or_default(),
            source: row.source_name.clone().unwrap_or_default(),
            lead_type: row.type_name.clone().unwrap_or_default(),
            pipeline: row.pipeline_name.clone().unwrap_or_default(),
            stage: row.stage_name.clone().unwrap_or_default(),
            assigned_to: row.user_name.clone().unwrap_or_default(),
            expected_close_date: row
                .expected_close_date
                .map(|d| d.to_string())
                .unwrap_or_default(),
            created_at: row.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        };
        if let Err(e) = wtr.serialize(&csv_row) {
            tracing::error!("CSV serialize error: {e}");
            return internal_error();
        }
    }

    let data = wtr.into_inner().unwrap_or_default();
    csv_response(data, "leads.csv")
}

pub async fn export_persons(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    if let Err(resp) = bouncer(&user, "contacts.persons") {
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
    let sql = format!(
        "SELECT t.id, t.name, t.emails, t.contact_numbers, t.job_title, t.created_at,
                o.name AS organization_name,
                CONCAT(u.first_name, ' ', u.last_name) AS user_name
         FROM persons t
         LEFT JOIN organizations o ON o.id = t.organization_id
         LEFT JOIN users u ON u.id = t.user_id
         WHERE true{vp}
         ORDER BY t.id DESC"
    );

    let rows = guard
        .fetch_all(sqlx::query_as::<_, PersonExportRow>(&sql))
        .await;
    let _ = guard.release().await;

    let rows = match rows {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to export persons: {e}");
            return internal_error();
        }
    };

    let mut wtr = csv::Writer::from_writer(Vec::new());
    for row in &rows {
        let csv_row = PersonCsvRow {
            id: row.id,
            name: row.name.clone(),
            email: first_json_value(&row.emails),
            phone: first_json_value(&row.contact_numbers),
            job_title: row.job_title.clone().unwrap_or_default(),
            organization: row.organization_name.clone().unwrap_or_default(),
            assigned_to: row.user_name.clone().unwrap_or_default(),
            created_at: row.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        };
        if let Err(e) = wtr.serialize(&csv_row) {
            tracing::error!("CSV serialize error: {e}");
            return internal_error();
        }
    }

    let data = wtr.into_inner().unwrap_or_default();
    csv_response(data, "persons.csv")
}

pub async fn export_organizations(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    if let Err(resp) = bouncer(&user, "contacts.organizations") {
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
    let sql = format!(
        "SELECT t.id, t.name, t.created_at,
                CONCAT(u.first_name, ' ', u.last_name) AS user_name
         FROM organizations t
         LEFT JOIN users u ON u.id = t.user_id
         WHERE true{vp}
         ORDER BY t.id DESC"
    );

    let rows = guard
        .fetch_all(sqlx::query_as::<_, OrgExportRow>(&sql))
        .await;
    let _ = guard.release().await;

    let rows = match rows {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to export organizations: {e}");
            return internal_error();
        }
    };

    let mut wtr = csv::Writer::from_writer(Vec::new());
    for row in &rows {
        let csv_row = OrganizationCsvRow {
            id: row.id,
            name: row.name.clone(),
            assigned_to: row.user_name.clone().unwrap_or_default(),
            created_at: row.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        };
        if let Err(e) = wtr.serialize(&csv_row) {
            tracing::error!("CSV serialize error: {e}");
            return internal_error();
        }
    }

    let data = wtr.into_inner().unwrap_or_default();
    csv_response(data, "organizations.csv")
}

pub async fn export_products(
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

    let rows = guard
        .fetch_all(sqlx::query_as::<_, ProductExportRow>(
            "SELECT id, sku, name, description, price, quantity, created_at
             FROM products ORDER BY id DESC",
        ))
        .await;
    let _ = guard.release().await;

    let rows = match rows {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to export products: {e}");
            return internal_error();
        }
    };

    let mut wtr = csv::Writer::from_writer(Vec::new());
    for row in &rows {
        let csv_row = ProductCsvRow {
            id: row.id,
            sku: row.sku.clone(),
            name: row.name.clone(),
            description: row.description.clone().unwrap_or_default(),
            price: row.price.to_string(),
            quantity: row.quantity,
            created_at: row.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        };
        if let Err(e) = wtr.serialize(&csv_row) {
            tracing::error!("CSV serialize error: {e}");
            return internal_error();
        }
    }

    let data = wtr.into_inner().unwrap_or_default();
    csv_response(data, "products.csv")
}

// ── IMPORT endpoints ──

#[derive(sqlx::FromRow)]
struct IdRow {
    id: i64,
}

pub async fn import_leads(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    mut multipart: Multipart,
) -> Response {
    if let Err(resp) = bouncer(&user, "leads.create") {
        return resp;
    }

    let csv_bytes = match read_csv_from_multipart(&mut multipart).await {
        Ok(b) => b,
        Err(resp) => return resp,
    };

    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(csv_bytes.as_slice());

    let headers = match rdr.headers() {
        Ok(h) => h.clone(),
        Err(e) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": format!("Invalid CSV headers: {e}") })),
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

    let mut imported = 0i64;
    let mut errors: Vec<String> = Vec::new();

    for (idx, result) in rdr.records().enumerate() {
        let row_num = idx + 2; // 1-indexed + header row
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                errors.push(format!("Row {row_num}: parse error — {e}"));
                continue;
            }
        };

        let get = |name: &str| -> String {
            headers
                .iter()
                .position(|h| h.eq_ignore_ascii_case(name))
                .and_then(|i| record.get(i))
                .unwrap_or("")
                .trim()
                .to_string()
        };

        let title = get("title");
        if title.is_empty() {
            errors.push(format!("Row {row_num}: title is required"));
            continue;
        }

        let description = get("description");
        let lead_value: Option<rust_decimal::Decimal> = {
            let v = get("lead_value");
            if v.is_empty() {
                None
            } else {
                match v.parse() {
                    Ok(d) => Some(d),
                    Err(_) => {
                        errors.push(format!("Row {row_num}: invalid lead_value '{v}'"));
                        continue;
                    }
                }
            }
        };

        // Lookup person by name
        let person_id: Option<i64> = {
            let name = get("contact_person");
            if name.is_empty() {
                None
            } else {
                guard
                    .fetch_optional(
                        sqlx::query_as::<_, IdRow>(
                            "SELECT id FROM persons WHERE name ILIKE $1 LIMIT 1",
                        )
                        .bind(&name),
                    )
                    .await
                    .ok()
                    .flatten()
                    .map(|r| r.id)
            }
        };

        // Lookup source by name
        let source_id: Option<i64> = {
            let name = get("source");
            if name.is_empty() {
                None
            } else {
                guard
                    .fetch_optional(
                        sqlx::query_as::<_, IdRow>(
                            "SELECT id FROM lead_sources WHERE name ILIKE $1 LIMIT 1",
                        )
                        .bind(&name),
                    )
                    .await
                    .ok()
                    .flatten()
                    .map(|r| r.id)
            }
        };

        // Lookup type by name
        let type_id: Option<i64> = {
            let name = get("lead_type");
            if name.is_empty() {
                None
            } else {
                guard
                    .fetch_optional(
                        sqlx::query_as::<_, IdRow>(
                            "SELECT id FROM lead_types WHERE name ILIKE $1 LIMIT 1",
                        )
                        .bind(&name),
                    )
                    .await
                    .ok()
                    .flatten()
                    .map(|r| r.id)
            }
        };

        // Lookup pipeline + stage
        let (pipeline_id, stage_id) = {
            let pip_name = get("pipeline");
            let stg_name = get("stage");

            if pip_name.is_empty() {
                // Use default pipeline + first stage
                let pip = guard
                    .fetch_optional(sqlx::query_as::<_, IdRow>(
                        "SELECT id FROM lead_pipelines WHERE is_default = true LIMIT 1",
                    ))
                    .await
                    .ok()
                    .flatten();

                if let Some(pip) = pip {
                    let stg = guard
                        .fetch_optional(
                            sqlx::query_as::<_, IdRow>(
                                "SELECT lps.id FROM lead_pipeline_stages lps
                                 WHERE lps.lead_pipeline_id = $1
                                 ORDER BY lps.sort_order LIMIT 1",
                            )
                            .bind(pip.id),
                        )
                        .await
                        .ok()
                        .flatten();
                    (Some(pip.id), stg.map(|s| s.id))
                } else {
                    (None, None)
                }
            } else {
                let pip = guard
                    .fetch_optional(
                        sqlx::query_as::<_, IdRow>(
                            "SELECT id FROM lead_pipelines WHERE name ILIKE $1 LIMIT 1",
                        )
                        .bind(&pip_name),
                    )
                    .await
                    .ok()
                    .flatten();

                if let Some(pip) = pip {
                    let stg = if stg_name.is_empty() {
                        guard
                            .fetch_optional(
                                sqlx::query_as::<_, IdRow>(
                                    "SELECT lps.id FROM lead_pipeline_stages lps
                                     WHERE lps.lead_pipeline_id = $1
                                     ORDER BY lps.sort_order LIMIT 1",
                                )
                                .bind(pip.id),
                            )
                            .await
                            .ok()
                            .flatten()
                    } else {
                        guard
                            .fetch_optional(
                                sqlx::query_as::<_, IdRow>(
                                    "SELECT lps.id FROM lead_pipeline_stages lps
                                     JOIN lead_stages ls ON ls.id = lps.lead_stage_id
                                     WHERE lps.lead_pipeline_id = $1 AND ls.name ILIKE $2
                                     LIMIT 1",
                                )
                                .bind(pip.id)
                                .bind(&stg_name),
                            )
                            .await
                            .ok()
                            .flatten()
                    };
                    (Some(pip.id), stg.map(|s| s.id))
                } else {
                    (None, None)
                }
            }
        };

        let result = guard
            .execute(
                sqlx::query(
                    "INSERT INTO leads (title, description, lead_value, person_id,
                        lead_source_id, lead_type_id, lead_pipeline_id,
                        lead_pipeline_stage_id, user_id)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                )
                .bind(&title)
                .bind(if description.is_empty() {
                    None
                } else {
                    Some(&description)
                })
                .bind(lead_value)
                .bind(person_id)
                .bind(source_id)
                .bind(type_id)
                .bind(pipeline_id)
                .bind(stage_id)
                .bind(user.id),
            )
            .await;

        match result {
            Ok(_) => imported += 1,
            Err(e) => errors.push(format!("Row {row_num}: insert failed — {e}")),
        }
    }

    let _ = guard.release().await;
    import_response(imported, errors)
}

pub async fn import_persons(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    mut multipart: Multipart,
) -> Response {
    if let Err(resp) = bouncer(&user, "contacts.persons.create") {
        return resp;
    }

    let csv_bytes = match read_csv_from_multipart(&mut multipart).await {
        Ok(b) => b,
        Err(resp) => return resp,
    };

    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(csv_bytes.as_slice());

    let headers = match rdr.headers() {
        Ok(h) => h.clone(),
        Err(e) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": format!("Invalid CSV headers: {e}") })),
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

    let mut imported = 0i64;
    let mut errors: Vec<String> = Vec::new();

    for (idx, result) in rdr.records().enumerate() {
        let row_num = idx + 2;
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                errors.push(format!("Row {row_num}: parse error — {e}"));
                continue;
            }
        };

        let get = |name: &str| -> String {
            headers
                .iter()
                .position(|h| h.eq_ignore_ascii_case(name))
                .and_then(|i| record.get(i))
                .unwrap_or("")
                .trim()
                .to_string()
        };

        let name = get("name");
        if name.is_empty() {
            errors.push(format!("Row {row_num}: name is required"));
            continue;
        }

        let email = get("email");
        let phone = get("phone");
        let job_title = get("job_title");

        let emails = if email.is_empty() {
            serde_json::json!([])
        } else {
            serde_json::json!([{ "value": email, "label": "work" }])
        };
        let contact_numbers = if phone.is_empty() {
            serde_json::json!([])
        } else {
            serde_json::json!([{ "value": phone, "label": "work" }])
        };

        // Lookup organization by name
        let org_id: Option<i64> = {
            let org_name = get("organization");
            if org_name.is_empty() {
                None
            } else {
                guard
                    .fetch_optional(
                        sqlx::query_as::<_, IdRow>(
                            "SELECT id FROM organizations WHERE name ILIKE $1 LIMIT 1",
                        )
                        .bind(&org_name),
                    )
                    .await
                    .ok()
                    .flatten()
                    .map(|r| r.id)
            }
        };

        let result = guard
            .execute(
                sqlx::query(
                    "INSERT INTO persons (name, emails, contact_numbers, job_title, organization_id, user_id)
                     VALUES ($1, $2, $3, $4, $5, $6)",
                )
                .bind(&name)
                .bind(&emails)
                .bind(&contact_numbers)
                .bind(if job_title.is_empty() {
                    None
                } else {
                    Some(&job_title)
                })
                .bind(org_id)
                .bind(user.id),
            )
            .await;

        match result {
            Ok(_) => imported += 1,
            Err(e) => errors.push(format!("Row {row_num}: insert failed — {e}")),
        }
    }

    let _ = guard.release().await;
    import_response(imported, errors)
}

pub async fn import_organizations(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    mut multipart: Multipart,
) -> Response {
    if let Err(resp) = bouncer(&user, "contacts.organizations.create") {
        return resp;
    }

    let csv_bytes = match read_csv_from_multipart(&mut multipart).await {
        Ok(b) => b,
        Err(resp) => return resp,
    };

    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(csv_bytes.as_slice());

    let headers = match rdr.headers() {
        Ok(h) => h.clone(),
        Err(e) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": format!("Invalid CSV headers: {e}") })),
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

    let mut imported = 0i64;
    let mut errors: Vec<String> = Vec::new();

    for (idx, result) in rdr.records().enumerate() {
        let row_num = idx + 2;
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                errors.push(format!("Row {row_num}: parse error — {e}"));
                continue;
            }
        };

        let get = |name: &str| -> String {
            headers
                .iter()
                .position(|h| h.eq_ignore_ascii_case(name))
                .and_then(|i| record.get(i))
                .unwrap_or("")
                .trim()
                .to_string()
        };

        let name = get("name");
        if name.is_empty() {
            errors.push(format!("Row {row_num}: name is required"));
            continue;
        }

        let result = guard
            .execute(
                sqlx::query(
                    "INSERT INTO organizations (name, user_id) VALUES ($1, $2)",
                )
                .bind(&name)
                .bind(user.id),
            )
            .await;

        match result {
            Ok(_) => imported += 1,
            Err(e) => errors.push(format!("Row {row_num}: insert failed — {e}")),
        }
    }

    let _ = guard.release().await;
    import_response(imported, errors)
}

pub async fn import_products(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    mut multipart: Multipart,
) -> Response {
    if let Err(resp) = bouncer(&user, "products.create") {
        return resp;
    }

    let csv_bytes = match read_csv_from_multipart(&mut multipart).await {
        Ok(b) => b,
        Err(resp) => return resp,
    };

    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(csv_bytes.as_slice());

    let headers = match rdr.headers() {
        Ok(h) => h.clone(),
        Err(e) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": format!("Invalid CSV headers: {e}") })),
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

    let mut imported = 0i64;
    let mut errors: Vec<String> = Vec::new();

    for (idx, result) in rdr.records().enumerate() {
        let row_num = idx + 2;
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                errors.push(format!("Row {row_num}: parse error — {e}"));
                continue;
            }
        };

        let get = |name: &str| -> String {
            headers
                .iter()
                .position(|h| h.eq_ignore_ascii_case(name))
                .and_then(|i| record.get(i))
                .unwrap_or("")
                .trim()
                .to_string()
        };

        let name = get("name");
        let sku = get("sku");
        if name.is_empty() || sku.is_empty() {
            errors.push(format!("Row {row_num}: name and sku are required"));
            continue;
        }

        let description = get("description");
        let price: rust_decimal::Decimal = {
            let v = get("price");
            if v.is_empty() {
                rust_decimal::Decimal::ZERO
            } else {
                match v.parse() {
                    Ok(d) => d,
                    Err(_) => {
                        errors.push(format!("Row {row_num}: invalid price '{v}'"));
                        continue;
                    }
                }
            }
        };
        let quantity: i32 = {
            let v = get("quantity");
            if v.is_empty() {
                0
            } else {
                match v.parse() {
                    Ok(q) => q,
                    Err(_) => {
                        errors.push(format!("Row {row_num}: invalid quantity '{v}'"));
                        continue;
                    }
                }
            }
        };

        let result = guard
            .execute(
                sqlx::query(
                    "INSERT INTO products (sku, name, description, price, quantity)
                     VALUES ($1, $2, $3, $4, $5)",
                )
                .bind(&sku)
                .bind(&name)
                .bind(if description.is_empty() {
                    None
                } else {
                    Some(&description)
                })
                .bind(price)
                .bind(quantity),
            )
            .await;

        match result {
            Ok(_) => imported += 1,
            Err(e) => errors.push(format!("Row {row_num}: insert failed — {e}")),
        }
    }

    let _ = guard.release().await;
    import_response(imported, errors)
}

// ── Shared helpers ──

async fn read_csv_from_multipart(multipart: &mut Multipart) -> Result<Vec<u8>, Response> {
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("file") {
            return field.bytes().await.map(|b| b.to_vec()).map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": format!("Failed to read file: {e}") })),
                )
                    .into_response()
            });
        }
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": "No CSV file provided. Use field name 'file'." })),
    )
        .into_response())
}

fn import_response(imported: i64, errors: Vec<String>) -> Response {
    let status = if errors.is_empty() {
        StatusCode::OK
    } else if imported == 0 {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };

    (
        status,
        Json(serde_json::json!({
            "imported": imported,
            "errors": errors,
            "message": format!("{imported} record(s) imported successfully.{}",
                if errors.is_empty() { String::new() }
                else { format!(" {} error(s) encountered.", errors.len()) }
            )
        })),
    )
        .into_response()
}
