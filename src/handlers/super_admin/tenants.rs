use axum::extract::{Extension, Path};
use axum::response::{IntoResponse, Response};
use tower_sessions::Session;

use crate::db::Database;
use crate::middleware::csrf::get_csrf_token;
use crate::models::super_admin::SuperAdmin;
use crate::views::super_admin::{TenantCreate, TenantEdit, TenantIndex};

pub async fn index(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(admin): Extension<SuperAdmin>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    let tenants = sqlx::query_as::<_, crate::models::company::Company>(
        "SELECT * FROM main.companies ORDER BY id DESC",
    )
    .fetch_all(db.reader())
    .await
    .unwrap_or_default();

    let initial_data = serde_json::json!({
        "tenants": tenants,
        "admin_name": admin.full_name(),
        "permission_type": admin.permission_type,
        "permissions": admin.role_permissions,
    });

    TenantIndex::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn create(
    session: Session,
    Extension(admin): Extension<SuperAdmin>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    let initial_data = serde_json::json!({
        "admin_name": admin.full_name(),
    });
    TenantCreate::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn edit(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(admin): Extension<SuperAdmin>,
    Path(id): Path<i64>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    let tenant = sqlx::query_as::<_, crate::models::company::Company>(
        "SELECT * FROM main.companies WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(db.reader())
    .await
    .ok()
    .flatten();

    let initial_data = serde_json::json!({
        "tenant": tenant,
        "admin_name": admin.full_name(),
    });
    TenantEdit::new(csrf_token, initial_data.to_string()).into_response()
}
