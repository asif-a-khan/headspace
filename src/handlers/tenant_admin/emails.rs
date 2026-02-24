use axum::extract::Extension;
use axum::response::{IntoResponse, Response};
use tower_sessions::Session;

use crate::api::tenant_admin::config::load_tenant_config;
use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::middleware::csrf::get_csrf_token;
use crate::models::company::Company;
use crate::models::tenant_admin::TenantUser;
use crate::views::tenant_admin::EmailIndex;

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
            return EmailIndex::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let config = load_tenant_config(&mut guard).await;

    // Get email counts per folder
    #[derive(sqlx::FromRow)]
    struct FolderCount {
        folder: String,
        count: Option<i64>,
    }

    let folder_counts = guard
        .fetch_all(sqlx::query_as::<_, FolderCount>(
            "SELECT folder, COUNT(*) AS count FROM emails GROUP BY folder",
        ))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let counts: std::collections::HashMap<String, i64> = folder_counts
        .into_iter()
        .map(|fc| (fc.folder, fc.count.unwrap_or(0)))
        .collect();

    let smtp_configured = config
        .get("email.smtp.host")
        .map(|h| !h.is_empty())
        .unwrap_or(false);

    let imap_configured = config
        .get("email.imap.host")
        .map(|h| !h.is_empty())
        .unwrap_or(false);
    let imap_enabled = config
        .get("email.imap.enabled")
        .map(|v| v == "true")
        .unwrap_or(false);
    let imap_last_sync_at = config
        .get("email.imap.last_sync_at")
        .cloned()
        .unwrap_or_default();

    let initial_data = serde_json::json!({
        "folder_counts": counts,
        "smtp_configured": smtp_configured,
        "imap_configured": imap_configured,
        "imap_enabled": imap_enabled,
        "imap_last_sync_at": imap_last_sync_at,
        "from_address": config.get("email.smtp.from_address").cloned().unwrap_or_default(),
        "from_name": config.get("email.smtp.from_name").cloned().unwrap_or_default(),
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    EmailIndex::new(csrf_token, initial_data.to_string()).into_response()
}
