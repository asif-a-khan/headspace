use axum::extract::{Extension, Path};
use axum::response::{IntoResponse, Response};
use tower_sessions::Session;

use crate::auth::acl::{flatten_acl, SUPER_ADMIN_ACL};
use crate::db::Database;
use crate::middleware::csrf::get_csrf_token;
use crate::models::super_admin::SuperAdmin;
use crate::models::super_role::SuperRole;
use crate::views::super_admin::{RoleCreate, RoleEdit, RoleIndex};

pub async fn index(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(admin): Extension<SuperAdmin>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    let roles = sqlx::query_as::<_, SuperRole>(
        "SELECT * FROM main.super_roles ORDER BY id DESC",
    )
    .fetch_all(db.reader())
    .await
    .unwrap_or_default();

    let initial_data = serde_json::json!({
        "roles": roles,
        "admin_name": admin.full_name(),
        "permission_type": admin.permission_type,
        "permissions": admin.role_permissions,
    });
    RoleIndex::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn create(
    session: Session,
    Extension(admin): Extension<SuperAdmin>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    let acl = flatten_acl(SUPER_ADMIN_ACL);
    let initial_data = serde_json::json!({
        "acl": acl,
        "admin_name": admin.full_name(),
    });
    RoleCreate::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn edit(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(admin): Extension<SuperAdmin>,
    Path(id): Path<i64>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    let role = sqlx::query_as::<_, SuperRole>(
        "SELECT * FROM main.super_roles WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(db.reader())
    .await
    .ok()
    .flatten();

    let acl = flatten_acl(SUPER_ADMIN_ACL);
    let initial_data = serde_json::json!({
        "role": role,
        "acl": acl,
        "admin_name": admin.full_name(),
    });
    RoleEdit::new(csrf_token, initial_data.to_string()).into_response()
}
