use axum::Json;
use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::collections::HashMap;

use crate::db::Database;
use crate::db::guard::TenantGuard;
use crate::models::company::Company;
use crate::models::web_form::{WebForm, WebFormAttributeRow};

#[derive(sqlx::FromRow)]
struct IdRow {
    id: i64,
}

/// Public endpoint: submit a web form. Creates a person and optionally a lead.
pub async fn submit(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Path(form_id): Path<String>,
    Json(values): Json<HashMap<String, serde_json::Value>>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal error." })),
            )
                .into_response();
        }
    };

    // Find the web form by its public form_id
    let form = match guard
        .fetch_optional(
            sqlx::query_as::<_, WebForm>("SELECT * FROM web_forms WHERE form_id = $1")
                .bind(&form_id),
        )
        .await
    {
        Ok(Some(f)) => f,
        Ok(None) => {
            let _ = guard.release().await;
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Form not found." })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to find web form: {e}");
            let _ = guard.release().await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal error." })),
            )
                .into_response();
        }
    };

    // Fetch form attributes with their attribute details
    let form_attrs = guard
        .fetch_all(
            sqlx::query_as::<_, WebFormAttributeRow>(
                "SELECT wfa.id, wfa.name, wfa.placeholder, wfa.is_required, wfa.sort_order,
                    wfa.attribute_id, wfa.web_form_id,
                    a.name AS attribute_name, a.code AS attribute_code, a.type AS attribute_type
             FROM web_form_attributes wfa
             JOIN attributes a ON a.id = wfa.attribute_id
             WHERE wfa.web_form_id = $1
             ORDER BY wfa.sort_order, wfa.id",
            )
            .bind(form.id),
        )
        .await
        .unwrap_or_default();

    // Validate required fields
    for attr in &form_attrs {
        if attr.is_required {
            let code = attr.attribute_code.as_deref().unwrap_or("");
            if let Some(val) = values.get(code) {
                if val.is_null() || (val.is_string() && val.as_str().unwrap_or("").is_empty()) {
                    let label = attr
                        .name
                        .as_deref()
                        .or(attr.attribute_name.as_deref())
                        .unwrap_or(code);
                    let _ = guard.release().await;
                    return (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(serde_json::json!({
                            "error": format!("{} is required.", label)
                        })),
                    )
                        .into_response();
                }
            } else {
                let label = attr
                    .name
                    .as_deref()
                    .or(attr.attribute_name.as_deref())
                    .unwrap_or(code);
                let _ = guard.release().await;
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(serde_json::json!({
                        "error": format!("{} is required.", label)
                    })),
                )
                    .into_response();
            }
        }
    }

    // Extract common fields: name, email
    let person_name = values
        .get("name")
        .or_else(|| values.get("person_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();

    let person_email = values
        .get("email")
        .or_else(|| values.get("person_email"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Create person
    let person_id: i64 = match guard
        .fetch_one(
            sqlx::query_as::<_, IdRow>("INSERT INTO persons (name) VALUES ($1) RETURNING id")
                .bind(&person_name),
        )
        .await
    {
        Ok(row) => row.id,
        Err(e) => {
            tracing::error!("Failed to create person from web form: {e}");
            let _ = guard.release().await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to process submission." })),
            )
                .into_response();
        }
    };

    // Add email if provided
    if let Some(ref email) = person_email {
        let _ = guard
            .execute(
                sqlx::query(
                    "INSERT INTO person_emails (person_id, email, label) VALUES ($1, $2, 'work')",
                )
                .bind(person_id)
                .bind(email),
            )
            .await;
    }

    // Add phone if provided
    if let Some(phone) = values
        .get("phone")
        .or_else(|| values.get("person_phone"))
        .and_then(|v| v.as_str())
        && !phone.is_empty()
    {
        let _ = guard
            .execute(
                sqlx::query(
                    "INSERT INTO person_phones (person_id, phone, label) VALUES ($1, $2, 'work')",
                )
                .bind(person_id)
                .bind(phone),
            )
            .await;
    }

    // Create lead if configured
    if form.create_lead {
        let default_title = format!("Web Form: {}", person_name);
        let lead_title = values
            .get("title")
            .or_else(|| values.get("lead_title"))
            .and_then(|v| v.as_str())
            .unwrap_or(&default_title);

        // Get default pipeline
        let pipeline_id = guard
            .fetch_one(sqlx::query_as::<_, IdRow>(
                "SELECT id FROM pipelines ORDER BY is_default DESC, id ASC LIMIT 1",
            ))
            .await
            .map(|r| r.id)
            .unwrap_or(1);

        let stage_id = guard
            .fetch_one(sqlx::query_as::<_, IdRow>(
                "SELECT id FROM pipeline_stages WHERE pipeline_id = $1 ORDER BY sort_order ASC LIMIT 1",
            ).bind(pipeline_id))
            .await
            .map(|r| r.id)
            .unwrap_or(1);

        let _ = guard.execute(
            sqlx::query(
                "INSERT INTO leads (title, person_id, pipeline_id, lead_pipeline_stage_id, status)
                 VALUES ($1, $2, $3, $4, 'new')",
            )
            .bind(lead_title)
            .bind(person_id)
            .bind(pipeline_id)
            .bind(stage_id),
        ).await;
    }

    // Store attribute values for the person
    for attr in &form_attrs {
        let code = attr.attribute_code.as_deref().unwrap_or("");
        if let Some(val) = values.get(code)
            && !val.is_null()
        {
            let text_val = match val {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            if !text_val.is_empty() {
                let _ = guard.execute(
                        sqlx::query(
                            "INSERT INTO attribute_values (entity_type, entity_id, attribute_id, text_value)
                             VALUES ('persons', $1, $2, $3)",
                        )
                        .bind(person_id)
                        .bind(attr.attribute_id)
                        .bind(&text_val),
                    ).await;
            }
        }
    }

    let _ = guard.release().await;

    let response = match form.submit_success_action.as_str() {
        "redirect" => serde_json::json!({
            "success": true,
            "action": "redirect",
            "redirect_url": form.submit_success_content,
        }),
        _ => serde_json::json!({
            "success": true,
            "action": "message",
            "message": form.submit_success_content,
        }),
    };

    (StatusCode::OK, Json(response)).into_response()
}
