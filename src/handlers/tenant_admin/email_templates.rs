use axum::extract::{Extension, Path};
use axum::response::{IntoResponse, Response};
use tower_sessions::Session;

use crate::db::Database;
use crate::db::guard::TenantGuard;
use crate::middleware::csrf::get_csrf_token;
use crate::models::company::Company;
use crate::models::email_template::EmailTemplate;
use crate::models::tenant_admin::TenantUser;
use crate::views::tenant_admin::{EmailTemplateCreate, EmailTemplateEdit, EmailTemplateIndex};

pub async fn index(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => return EmailTemplateIndex::new(csrf_token, "{}".to_string()).into_response(),
    };
    let templates = guard
        .fetch_all(sqlx::query_as::<_, EmailTemplate>(
            "SELECT * FROM email_templates ORDER BY id DESC",
        ))
        .await
        .unwrap_or_default();
    let _ = guard.release().await;
    let initial_data = serde_json::json!({
        "email_templates": templates,
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    EmailTemplateIndex::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn create(
    session: Session,
    Extension(_db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    let initial_data = serde_json::json!({
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    EmailTemplateCreate::new(csrf_token, initial_data.to_string()).into_response()
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
        Err(_) => return EmailTemplateEdit::new(csrf_token, "{}".to_string()).into_response(),
    };
    let template = guard
        .fetch_optional(
            sqlx::query_as::<_, EmailTemplate>("SELECT * FROM email_templates WHERE id = $1")
                .bind(id),
        )
        .await
        .ok()
        .flatten();
    let _ = guard.release().await;
    let initial_data = serde_json::json!({
        "email_template": template,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    EmailTemplateEdit::new(csrf_token, initial_data.to_string()).into_response()
}
