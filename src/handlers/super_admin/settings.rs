use axum::extract::Extension;
use axum::response::{IntoResponse, Response};
use tower_sessions::Session;

use crate::middleware::csrf::get_csrf_token;
use crate::models::super_admin::SuperAdmin;
use crate::views::super_admin::SettingsIndex;

pub async fn index(session: Session, Extension(admin): Extension<SuperAdmin>) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    let initial_data = serde_json::json!({
        "admin_name": admin.full_name(),
        "permission_type": admin.permission_type,
        "permissions": admin.role_permissions,
    });
    SettingsIndex::new(csrf_token, initial_data.to_string()).into_response()
}
