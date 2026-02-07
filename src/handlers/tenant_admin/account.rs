use axum::extract::Extension;
use axum::response::{IntoResponse, Response};
use tower_sessions::Session;

use crate::middleware::csrf::get_csrf_token;
use crate::models::company::Company;
use crate::models::tenant_admin::TenantUser;
use crate::views::tenant_admin::AccountEdit;

pub async fn edit(
    session: Session,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    let initial_data = serde_json::json!({
        "account": {
            "first_name": user.first_name,
            "last_name": user.last_name,
            "email": user.email,
        },
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    AccountEdit::new(csrf_token, initial_data.to_string()).into_response()
}
