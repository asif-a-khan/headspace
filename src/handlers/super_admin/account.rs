use axum::extract::Extension;
use axum::response::{IntoResponse, Response};
use tower_sessions::Session;

use crate::middleware::csrf::get_csrf_token;
use crate::models::super_admin::SuperAdmin;
use crate::views::super_admin::AccountEdit;

pub async fn edit(
    session: Session,
    Extension(admin): Extension<SuperAdmin>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    let initial_data = serde_json::json!({
        "account": {
            "first_name": admin.first_name,
            "last_name": admin.last_name,
            "email": admin.email,
        },
        "admin_name": admin.full_name(),
    });
    AccountEdit::new(csrf_token, initial_data.to_string()).into_response()
}
