use axum::response::{IntoResponse, Response};
use tower_sessions::Session;

use crate::middleware::csrf::get_csrf_token;
use crate::middleware::auth::clear_super_admin_session;
use crate::views::super_admin::LoginPage;

pub async fn login_page(session: Session) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    LoginPage::new(csrf_token).into_response()
}

pub async fn logout(session: Session) -> Response {
    let _ = clear_super_admin_session(&session).await;
    axum::response::Redirect::to("/super/login").into_response()
}
