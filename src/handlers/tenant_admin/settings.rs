use axum::extract::{Extension, Path};
use axum::response::{IntoResponse, Response};
use tower_sessions::Session;

use crate::auth::acl::{flatten_acl, TENANT_ADMIN_ACL};
use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::middleware::csrf::get_csrf_token;
use crate::models::company::Company;
use crate::models::group::Group;
use crate::models::tenant_admin::{TenantRole, TenantUser};
use crate::api::tenant_admin::config::load_tenant_config;
use crate::models::tag::Tag;
use crate::views::tenant_admin::{
    ConfigurationIndex, GroupCreate, GroupEdit, GroupIndex, RoleCreate, RoleEdit, RoleIndex,
    SettingsIndex, TagIndex, UserCreate, UserEdit, UserIndex,
};

pub async fn index(
    session: Session,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    let initial_data = serde_json::json!({
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    SettingsIndex::new(csrf_token, initial_data.to_string()).into_response()
}

// -- Users --

pub async fn users_index(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => {
            return UserIndex::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let users = guard
        .fetch_all(sqlx::query_as::<_, TenantUser>(
            "SELECT u.*, r.permission_type, r.permissions AS role_permissions
             FROM users u
             JOIN roles r ON r.id = u.role_id
             ORDER BY u.id DESC",
        ))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "users": users,
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    UserIndex::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn users_create(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => {
            return UserCreate::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let roles = guard
        .fetch_all(sqlx::query_as::<_, TenantRole>(
            "SELECT * FROM roles ORDER BY name",
        ))
        .await
        .unwrap_or_default();

    let groups = guard
        .fetch_all(sqlx::query_as::<_, Group>(
            "SELECT * FROM groups ORDER BY name",
        ))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "roles": roles,
        "groups": groups,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    UserCreate::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn users_edit(
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
            return UserEdit::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let edit_user = guard
        .fetch_optional(sqlx::query_as::<_, TenantUser>(
            "SELECT u.*, r.permission_type, r.permissions AS role_permissions
             FROM users u
             JOIN roles r ON r.id = u.role_id
             WHERE u.id = $1",
        )
        .bind(id))
        .await
        .ok()
        .flatten();

    let group_ids = guard
        .fetch_all(sqlx::query_as::<_, (i64,)>(
            "SELECT group_id FROM user_groups WHERE user_id = $1",
        )
        .bind(id))
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| r.0)
        .collect::<Vec<_>>();

    let roles = guard
        .fetch_all(sqlx::query_as::<_, TenantRole>(
            "SELECT * FROM roles ORDER BY name",
        ))
        .await
        .unwrap_or_default();

    let groups = guard
        .fetch_all(sqlx::query_as::<_, Group>(
            "SELECT * FROM groups ORDER BY name",
        ))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "user": edit_user,
        "group_ids": group_ids,
        "roles": roles,
        "groups": groups,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    UserEdit::new(csrf_token, initial_data.to_string()).into_response()
}

// -- Roles --

pub async fn roles_index(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => {
            return RoleIndex::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let roles = guard
        .fetch_all(sqlx::query_as::<_, TenantRole>(
            "SELECT * FROM roles ORDER BY id DESC",
        ))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "roles": roles,
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    RoleIndex::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn roles_create(
    session: Session,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    let acl = flatten_acl(TENANT_ADMIN_ACL);
    let initial_data = serde_json::json!({
        "acl": acl,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    RoleCreate::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn roles_edit(
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
            return RoleEdit::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let role = guard
        .fetch_optional(sqlx::query_as::<_, TenantRole>(
            "SELECT * FROM roles WHERE id = $1",
        )
        .bind(id))
        .await
        .ok()
        .flatten();

    let _ = guard.release().await;

    let acl = flatten_acl(TENANT_ADMIN_ACL);
    let initial_data = serde_json::json!({
        "role": role,
        "acl": acl,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    RoleEdit::new(csrf_token, initial_data.to_string()).into_response()
}

// -- Groups --

pub async fn groups_index(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => {
            return GroupIndex::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let groups = guard
        .fetch_all(sqlx::query_as::<_, Group>(
            "SELECT * FROM groups ORDER BY id DESC",
        ))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "groups": groups,
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    GroupIndex::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn groups_create(
    session: Session,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    let initial_data = serde_json::json!({
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    GroupCreate::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn groups_edit(
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
            return GroupEdit::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let group = guard
        .fetch_optional(sqlx::query_as::<_, Group>(
            "SELECT * FROM groups WHERE id = $1",
        )
        .bind(id))
        .await
        .ok()
        .flatten();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "group": group,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    GroupEdit::new(csrf_token, initial_data.to_string()).into_response()
}

// -- Configuration --

pub async fn configuration_index(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => {
            return ConfigurationIndex::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let config = load_tenant_config(&mut guard).await;
    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "config": config,
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    ConfigurationIndex::new(csrf_token, initial_data.to_string()).into_response()
}

// -- Tags --

pub async fn tags_index(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => return TagIndex::new(csrf_token, "{}".to_string()).into_response(),
    };

    let tags = guard
        .fetch_all(sqlx::query_as::<_, Tag>("SELECT * FROM tags ORDER BY name"))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "tags": tags,
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    TagIndex::new(csrf_token, initial_data.to_string()).into_response()
}
