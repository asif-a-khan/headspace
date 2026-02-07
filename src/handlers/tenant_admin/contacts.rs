use axum::extract::{Extension, Path};
use axum::response::{IntoResponse, Response};
use tower_sessions::Session;

use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::middleware::csrf::get_csrf_token;
use crate::models::company::Company;
use crate::models::organization::OrganizationRow;
use crate::models::person::PersonRow;
use crate::models::tenant_admin::TenantUser;
use crate::views::tenant_admin::{
    OrganizationCreate, OrganizationEdit, OrganizationIndex, PersonCreate, PersonEdit, PersonIndex,
};

use crate::api::tenant_admin::contacts::view_permission_filter;

// -- Persons --

pub async fn persons_index(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => {
            return PersonIndex::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let vp = view_permission_filter(user.id, &user.view_permission);
    let sql = format!(
        "SELECT t.*, o.name AS organization_name,
                CONCAT(u.first_name, ' ', u.last_name) AS user_name
         FROM persons t
         LEFT JOIN organizations o ON o.id = t.organization_id
         LEFT JOIN users u ON u.id = t.user_id
         WHERE true{vp}
         ORDER BY t.id DESC"
    );

    let persons = guard
        .fetch_all(sqlx::query_as::<_, PersonRow>(&sql))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "persons": persons,
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    PersonIndex::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn persons_create(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => {
            return PersonCreate::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let orgs = guard
        .fetch_all(sqlx::query_as::<_, crate::models::organization::Organization>(
            "SELECT * FROM organizations ORDER BY name",
        ))
        .await
        .unwrap_or_default();

    let users = guard
        .fetch_all(sqlx::query_as::<_, TenantUser>(
            "SELECT * FROM users WHERE status = true ORDER BY first_name",
        ))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "organizations": orgs,
        "users": users,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    PersonCreate::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn persons_edit(
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
            return PersonEdit::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let person = guard
        .fetch_optional(
            sqlx::query_as::<_, crate::models::person::Person>(
                "SELECT * FROM persons WHERE id = $1",
            )
            .bind(id),
        )
        .await
        .ok()
        .flatten();

    let orgs = guard
        .fetch_all(sqlx::query_as::<_, crate::models::organization::Organization>(
            "SELECT * FROM organizations ORDER BY name",
        ))
        .await
        .unwrap_or_default();

    let users = guard
        .fetch_all(sqlx::query_as::<_, TenantUser>(
            "SELECT * FROM users WHERE status = true ORDER BY first_name",
        ))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "person": person,
        "organizations": orgs,
        "users": users,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    PersonEdit::new(csrf_token, initial_data.to_string()).into_response()
}

// -- Organizations --

pub async fn organizations_index(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => {
            return OrganizationIndex::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let vp = view_permission_filter(user.id, &user.view_permission);
    let sql = format!(
        "SELECT t.*, CONCAT(u.first_name, ' ', u.last_name) AS user_name
         FROM organizations t
         LEFT JOIN users u ON u.id = t.user_id
         WHERE true{vp}
         ORDER BY t.id DESC"
    );

    let orgs = guard
        .fetch_all(sqlx::query_as::<_, OrganizationRow>(&sql))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "organizations": orgs,
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    OrganizationIndex::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn organizations_create(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => {
            return OrganizationCreate::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let users = guard
        .fetch_all(sqlx::query_as::<_, TenantUser>(
            "SELECT * FROM users WHERE status = true ORDER BY first_name",
        ))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "users": users,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    OrganizationCreate::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn organizations_edit(
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
            return OrganizationEdit::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let org = guard
        .fetch_optional(
            sqlx::query_as::<_, crate::models::organization::Organization>(
                "SELECT * FROM organizations WHERE id = $1",
            )
            .bind(id),
        )
        .await
        .ok()
        .flatten();

    let users = guard
        .fetch_all(sqlx::query_as::<_, TenantUser>(
            "SELECT * FROM users WHERE status = true ORDER BY first_name",
        ))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "organization": org,
        "users": users,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    OrganizationEdit::new(csrf_token, initial_data.to_string()).into_response()
}
