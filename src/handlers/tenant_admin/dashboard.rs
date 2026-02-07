use axum::extract::Extension;
use axum::response::{IntoResponse, Response};
use tower_sessions::Session;

use crate::middleware::csrf::get_csrf_token;
use crate::models::company::Company;
use crate::models::tenant_admin::TenantUser;
use crate::views::tenant_admin::Dashboard;

pub async fn index(
    session: Session,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    let initial_data = serde_json::json!({
        "admin_name": user.full_name(),
        "company_name": company.name,
    })
    .to_string();
    Dashboard::new(csrf_token, initial_data).into_response()
}
