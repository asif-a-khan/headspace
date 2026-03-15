use axum::extract::{Extension, Path};
use axum::response::{IntoResponse, Response};
use tower_sessions::Session;

use crate::db::Database;
use crate::db::guard::TenantGuard;
use crate::middleware::csrf::get_csrf_token;
use crate::models::attribute::Attribute;
use crate::models::company::Company;
use crate::models::tenant_admin::TenantUser;
use crate::models::web_form::{WebForm, WebFormAttributeRow};
use crate::views::tenant_admin::{WebFormCreate, WebFormEdit, WebFormIndex};

pub async fn index(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => return WebFormIndex::new(csrf_token, "{}".to_string()).into_response(),
    };
    let forms = guard
        .fetch_all(sqlx::query_as::<_, WebForm>(
            "SELECT * FROM web_forms ORDER BY id DESC",
        ))
        .await
        .unwrap_or_default();
    let _ = guard.release().await;
    let initial_data = serde_json::json!({
        "web_forms": forms,
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    WebFormIndex::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn create(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => return WebFormCreate::new(csrf_token, "{}".to_string()).into_response(),
    };

    let attributes = guard
        .fetch_all(sqlx::query_as::<_, Attribute>(
            "SELECT * FROM attributes WHERE entity_type IN ('persons', 'leads') ORDER BY sort_order, name",
        ))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "attributes": attributes,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    WebFormCreate::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn edit(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => return WebFormEdit::new(csrf_token, "{}".to_string()).into_response(),
    };

    let form = guard
        .fetch_optional(
            sqlx::query_as::<_, WebForm>("SELECT * FROM web_forms WHERE id = $1").bind(id),
        )
        .await
        .ok()
        .flatten();

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
            .bind(id),
        )
        .await
        .unwrap_or_default();

    let attributes = guard
        .fetch_all(sqlx::query_as::<_, Attribute>(
            "SELECT * FROM attributes WHERE entity_type IN ('persons', 'leads') ORDER BY sort_order, name",
        ))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "web_form": form,
        "form_attributes": form_attrs,
        "attributes": attributes,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    WebFormEdit::new(csrf_token, initial_data.to_string()).into_response()
}
