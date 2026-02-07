use axum::extract::{Extension, Path};
use axum::response::{IntoResponse, Response};
use tower_sessions::Session;

use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::middleware::csrf::get_csrf_token;
use crate::models::activity::ActivityRow;
use crate::models::company::Company;
use crate::models::tenant_admin::TenantUser;
use crate::views::tenant_admin::{ActivityCreate, ActivityEdit, ActivityIndex};

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
            return ActivityIndex::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let activities = guard
        .fetch_all(sqlx::query_as::<_, ActivityRow>(
            "SELECT a.id, a.title, a.type, a.comment, a.location,
                    a.schedule_from, a.schedule_to, a.is_done, a.user_id,
                    a.created_at, a.updated_at,
                    CONCAT(u.first_name, ' ', u.last_name) AS user_name
             FROM activities a
             LEFT JOIN users u ON u.id = a.user_id
             ORDER BY a.id DESC",
        ))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "activities": activities,
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    ActivityIndex::new(csrf_token, initial_data.to_string()).into_response()
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
        Err(_) => {
            return ActivityCreate::new(csrf_token, "{}".to_string()).into_response();
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
    ActivityCreate::new(csrf_token, initial_data.to_string()).into_response()
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
            return ActivityEdit::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let activity = guard
        .fetch_optional(
            sqlx::query_as::<_, crate::models::activity::Activity>(
                "SELECT * FROM activities WHERE id = $1",
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
        "activity": activity,
        "users": users,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    ActivityEdit::new(csrf_token, initial_data.to_string()).into_response()
}
