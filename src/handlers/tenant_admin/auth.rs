use axum::extract::Extension;
use axum::response::{IntoResponse, Response};
use tower_sessions::Session;

use crate::middleware::auth::clear_tenant_admin_session;
use crate::middleware::csrf::get_csrf_token;
use crate::models::company::Company;
use crate::views::tenant_admin::LoginPage;

pub async fn login_page(
    session: Session,
    Extension(company): Extension<Company>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    let initial_data = serde_json::json!({
        "company_name": company.name,
    })
    .to_string();
    LoginPage::new(csrf_token, initial_data).into_response()
}

pub async fn logout(session: Session) -> Response {
    let _ = clear_tenant_admin_session(&session).await;
    axum::response::Redirect::to("/admin/login").into_response()
}
