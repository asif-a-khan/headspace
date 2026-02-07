use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use validator::Validate;

use crate::auth::bouncer::{bouncer, validate_payload};
use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::models::company::Company;
use crate::models::tenant_admin::TenantUser;
use crate::models::web_form::{WebForm, WebFormAttributeRow};

#[derive(Deserialize, Validate)]
pub struct WebFormPayload {
    #[validate(length(min = 1, message = "Title is required."))]
    pub title: String,
    pub description: Option<String>,
    pub submit_button_label: Option<String>,
    pub submit_success_action: Option<String>,
    pub submit_success_content: Option<String>,
    pub create_lead: Option<bool>,
    pub background_color: Option<String>,
    pub form_background_color: Option<String>,
    pub form_title_color: Option<String>,
    pub form_submit_button_color: Option<String>,
    pub attribute_label_color: Option<String>,
    pub attributes: Option<Vec<WebFormAttrInput>>,
}

#[derive(Deserialize)]
pub struct WebFormAttrInput {
    pub attribute_id: i64,
    pub name: Option<String>,
    pub placeholder: Option<String>,
    pub is_required: Option<bool>,
    pub sort_order: Option<i32>,
}

pub async fn list(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.web_forms") { return resp; }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let forms = guard
        .fetch_all(sqlx::query_as::<_, WebForm>(
            "SELECT * FROM web_forms ORDER BY id DESC",
        ))
        .await;

    let _ = guard.release().await;

    match forms {
        Ok(f) => Json(serde_json::json!({ "data": f })).into_response(),
        Err(e) => {
            tracing::error!("Failed to list web forms: {e}");
            internal_error()
        }
    }
}

pub async fn store(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Json(payload): Json<WebFormPayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.web_forms.create") { return resp; }
    if let Err(resp) = validate_payload(&payload) { return resp; }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    // Generate a unique form_id
    let form_id = generate_form_id();
    let submit_button_label = payload.submit_button_label.as_deref().unwrap_or("Submit");
    let submit_success_action = payload.submit_success_action.as_deref().unwrap_or("message");
    let submit_success_content = payload.submit_success_content.as_deref().unwrap_or("Thank you for your submission.");
    let create_lead = payload.create_lead.unwrap_or(true);

    let result = guard
        .fetch_one(sqlx::query_as::<_, WebForm>(
            "INSERT INTO web_forms (form_id, title, description, submit_button_label, submit_success_action, submit_success_content, create_lead, background_color, form_background_color, form_title_color, form_submit_button_color, attribute_label_color)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING *",
        )
        .bind(&form_id)
        .bind(&payload.title)
        .bind(&payload.description)
        .bind(submit_button_label)
        .bind(submit_success_action)
        .bind(submit_success_content)
        .bind(create_lead)
        .bind(payload.background_color.as_deref().unwrap_or("#F7F8F9"))
        .bind(payload.form_background_color.as_deref().unwrap_or("#FFFFFF"))
        .bind(payload.form_title_color.as_deref().unwrap_or("#263238"))
        .bind(payload.form_submit_button_color.as_deref().unwrap_or("#0E90D9"))
        .bind(payload.attribute_label_color.as_deref().unwrap_or("#546E7A")))
        .await;

    let form = match result {
        Ok(f) => f,
        Err(e) => {
            tracing::error!("Failed to create web form: {e}");
            let _ = guard.release().await;
            return (StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "Failed to create web form." }))).into_response();
        }
    };

    // Insert attributes
    if let Some(ref attrs) = payload.attributes {
        for attr in attrs {
            let _ = guard.execute(
                sqlx::query(
                    "INSERT INTO web_form_attributes (web_form_id, attribute_id, name, placeholder, is_required, sort_order)
                     VALUES ($1, $2, $3, $4, $5, $6)",
                )
                .bind(form.id)
                .bind(attr.attribute_id)
                .bind(&attr.name)
                .bind(&attr.placeholder)
                .bind(attr.is_required.unwrap_or(false))
                .bind(attr.sort_order.unwrap_or(0)),
            ).await;
        }
    }

    let _ = guard.release().await;

    (StatusCode::CREATED, Json(serde_json::json!({ "data": form, "message": "Web form created successfully." }))).into_response()
}

pub async fn show(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.web_forms") { return resp; }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let form = guard
        .fetch_optional(sqlx::query_as::<_, WebForm>("SELECT * FROM web_forms WHERE id = $1").bind(id))
        .await;

    let attrs = guard
        .fetch_all(sqlx::query_as::<_, WebFormAttributeRow>(
            "SELECT wfa.id, wfa.name, wfa.placeholder, wfa.is_required, wfa.sort_order,
                    wfa.attribute_id, wfa.web_form_id,
                    a.name AS attribute_name, a.code AS attribute_code, a.type AS attribute_type
             FROM web_form_attributes wfa
             JOIN attributes a ON a.id = wfa.attribute_id
             WHERE wfa.web_form_id = $1
             ORDER BY wfa.sort_order, wfa.id",
        ).bind(id))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    match form {
        Ok(Some(f)) => Json(serde_json::json!({ "data": f, "attributes": attrs })).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Web form not found." }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch web form: {e}");
            internal_error()
        }
    }
}

pub async fn update(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
    Json(payload): Json<WebFormPayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.web_forms.edit") { return resp; }
    if let Err(resp) = validate_payload(&payload) { return resp; }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let submit_button_label = payload.submit_button_label.as_deref().unwrap_or("Submit");
    let submit_success_action = payload.submit_success_action.as_deref().unwrap_or("message");
    let submit_success_content = payload.submit_success_content.as_deref().unwrap_or("Thank you for your submission.");
    let create_lead = payload.create_lead.unwrap_or(true);

    let result = guard
        .fetch_optional(sqlx::query_as::<_, WebForm>(
            "UPDATE web_forms SET title = $1, description = $2, submit_button_label = $3, submit_success_action = $4,
             submit_success_content = $5, create_lead = $6, background_color = $7, form_background_color = $8,
             form_title_color = $9, form_submit_button_color = $10, attribute_label_color = $11, updated_at = NOW()
             WHERE id = $12 RETURNING *",
        )
        .bind(&payload.title)
        .bind(&payload.description)
        .bind(submit_button_label)
        .bind(submit_success_action)
        .bind(submit_success_content)
        .bind(create_lead)
        .bind(payload.background_color.as_deref().unwrap_or("#F7F8F9"))
        .bind(payload.form_background_color.as_deref().unwrap_or("#FFFFFF"))
        .bind(payload.form_title_color.as_deref().unwrap_or("#263238"))
        .bind(payload.form_submit_button_color.as_deref().unwrap_or("#0E90D9"))
        .bind(payload.attribute_label_color.as_deref().unwrap_or("#546E7A"))
        .bind(id))
        .await;

    let form = match result {
        Ok(Some(f)) => f,
        Ok(None) => {
            let _ = guard.release().await;
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Web form not found." }))).into_response();
        }
        Err(e) => {
            tracing::error!("Failed to update web form: {e}");
            let _ = guard.release().await;
            return (StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "Failed to update web form." }))).into_response();
        }
    };

    // Sync attributes: delete old, insert new
    if let Some(ref attrs) = payload.attributes {
        let _ = guard.execute(sqlx::query("DELETE FROM web_form_attributes WHERE web_form_id = $1").bind(id)).await;
        for attr in attrs {
            let _ = guard.execute(
                sqlx::query(
                    "INSERT INTO web_form_attributes (web_form_id, attribute_id, name, placeholder, is_required, sort_order)
                     VALUES ($1, $2, $3, $4, $5, $6)",
                )
                .bind(form.id)
                .bind(attr.attribute_id)
                .bind(&attr.name)
                .bind(&attr.placeholder)
                .bind(attr.is_required.unwrap_or(false))
                .bind(attr.sort_order.unwrap_or(0)),
            ).await;
        }
    }

    let _ = guard.release().await;

    Json(serde_json::json!({ "data": form, "message": "Web form updated successfully." })).into_response()
}

pub async fn destroy(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.web_forms.delete") { return resp; }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard.execute(sqlx::query("DELETE FROM web_forms WHERE id = $1").bind(id)).await;
    let _ = guard.release().await;

    match result {
        Ok(r) if r.rows_affected() > 0 => Json(serde_json::json!({ "message": "Web form deleted successfully." })).into_response(),
        Ok(_) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Web form not found." }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to delete web form: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Failed to delete web form." }))).into_response()
        }
    }
}

fn internal_error() -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "An internal error occurred." }))).into_response()
}

fn generate_form_id() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789".chars().collect();
    (0..12).map(|_| chars[rng.random_range(0..chars.len())]).collect()
}
