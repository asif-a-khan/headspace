use axum::extract::{Extension, Path};
use axum::response::{IntoResponse, Response};
use tower_sessions::Session;

use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::middleware::csrf::get_csrf_token;
use crate::models::attribute::{Attribute, AttributeOption};
use crate::models::company::Company;
use crate::models::tenant_admin::TenantUser;
use crate::views::tenant_admin::{AttributeCreate, AttributeEdit, AttributeIndex};

pub async fn index(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => {
            return AttributeIndex::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let attributes = guard
        .fetch_all(sqlx::query_as::<_, Attribute>(
            "SELECT * FROM attributes ORDER BY entity_type, sort_order, id",
        ))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "attributes": attributes,
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    AttributeIndex::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn create(
    session: Session,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    let initial_data = serde_json::json!({
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    AttributeCreate::new(csrf_token, initial_data.to_string()).into_response()
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
        Err(_) => {
            return AttributeEdit::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let attribute = guard
        .fetch_optional(
            sqlx::query_as::<_, Attribute>("SELECT * FROM attributes WHERE id = $1").bind(id),
        )
        .await
        .ok()
        .flatten();

    let options = guard
        .fetch_all(
            sqlx::query_as::<_, AttributeOption>(
                "SELECT * FROM attribute_options WHERE attribute_id = $1 ORDER BY sort_order",
            )
            .bind(id),
        )
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "attribute": attribute,
        "options": options,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    AttributeEdit::new(csrf_token, initial_data.to_string()).into_response()
}
