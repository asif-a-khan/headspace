use axum::extract::{Extension, Path};
use axum::response::{IntoResponse, Response};
use tower_sessions::Session;

use crate::db::Database;
use crate::middleware::csrf::get_csrf_token;
use crate::models::super_admin::SuperAdmin;
use crate::models::super_role::SuperRole;
use crate::views::super_admin::{AgentCreate, AgentEdit, AgentIndex};

pub async fn index(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(admin): Extension<SuperAdmin>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    let agents = sqlx::query_as::<_, SuperAdmin>(
        "SELECT sa.*, sr.permission_type, sr.permissions AS role_permissions
            FROM main.super_admins sa
            JOIN main.super_roles sr ON sr.id = sa.role_id
            ORDER BY sa.id DESC",
    )
    .fetch_all(db.reader())
    .await
    .unwrap_or_default();

    let initial_data = serde_json::json!({
        "agents": agents,
        "admin_name": admin.full_name(),
        "permission_type": admin.permission_type,
        "permissions": admin.role_permissions,
    });
    AgentIndex::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn create(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(admin): Extension<SuperAdmin>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    let roles = sqlx::query_as::<_, SuperRole>("SELECT * FROM main.super_roles ORDER BY name")
        .fetch_all(db.reader())
        .await
        .unwrap_or_default();

    let initial_data = serde_json::json!({
        "roles": roles,
        "admin_name": admin.full_name(),
    });
    AgentCreate::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn edit(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(admin): Extension<SuperAdmin>,
    Path(id): Path<i64>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    let agent = sqlx::query_as::<_, SuperAdmin>(
        "SELECT sa.*, sr.permission_type, sr.permissions AS role_permissions
            FROM main.super_admins sa
            JOIN main.super_roles sr ON sr.id = sa.role_id
            WHERE sa.id = $1",
    )
    .bind(id)
    .fetch_optional(db.reader())
    .await
    .ok()
    .flatten();

    let roles = sqlx::query_as::<_, SuperRole>("SELECT * FROM main.super_roles ORDER BY name")
        .fetch_all(db.reader())
        .await
        .unwrap_or_default();

    let initial_data = serde_json::json!({
        "agent": agent,
        "roles": roles,
        "admin_name": admin.full_name(),
    });
    AgentEdit::new(csrf_token, initial_data.to_string()).into_response()
}
