use axum::body::Body;
use axum::http::Request;
use axum::Router;
use http_body_util::BodyExt;
use tower::ServiceExt;

use headspace::config::Config;
use headspace::db::Database;
use headspace::routes::app_router;

use sqlx::postgres::PgPoolOptions;
use std::time::Duration as StdDuration;
use time::Duration;
use tokio::sync::OnceCell;
use tower_sessions::cookie::SameSite;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;

/// Shared app router — initialized once, cloned per test.
///
/// Migrations run once. The seeded data (admin@headspace.local / admin123)
/// is read-only across all auth tests so no per-test truncation is needed.
static TEST_APP: OnceCell<Router> = OnceCell::const_new();

async fn init_test_app() -> Router {
    let db_url = "postgres:///headspace_test?host=/run/postgresql";

    // Use a larger pool for tests — parallel tests share the pool across
    // handler queries AND session store operations.
    let pool = PgPoolOptions::new()
        .max_connections(30)
        .acquire_timeout(StdDuration::from_secs(10))
        .connect(db_url)
        .await
        .expect("Failed to connect to headspace_test database");
    let db = Database::from_pool(pool);

    // Run migrations
    headspace::db::migrate::run_main_migrations(db.writer())
        .await
        .expect("Failed to run migrations");

    // Truncate and re-seed (runs once at startup)
    sqlx::query("TRUNCATE main.super_admins, main.super_roles, main.companies RESTART IDENTITY CASCADE")
        .execute(db.writer())
        .await
        .expect("Failed to truncate tables");

    // Also clear stale sessions from previous test runs
    sqlx::query("DELETE FROM tower_sessions")
        .execute(db.writer())
        .await
        .ok(); // Ignore if table doesn't exist yet

    headspace::db::seed::seed_default_super_admin(db.writer())
        .await
        .expect("Failed to seed test data");

    // Session store
    let session_store = PostgresStore::new(db.writer().clone());
    session_store
        .migrate()
        .await
        .expect("Failed to create session table");

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::hours(1)));

    let config = Config {
        app_host: "127.0.0.1".to_string(),
        app_port: 0,
        primary_domain: "headspace.local".to_string(),
        database_writer_url: db_url.to_string(),
        database_reader_url: db_url.to_string(),
        session_secret: "test-secret-not-for-production".to_string(),
    };

    app_router(db, config, session_layer)
}

/// Get the shared test app router (initialized once, cloned per call).
pub async fn setup_test_app() -> Router {
    TEST_APP.get_or_init(init_test_app).await.clone()
}

/// Extract session cookies from a response's Set-Cookie headers.
fn extract_cookies(resp: &axum::response::Response) -> Vec<String> {
    resp.headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| {
            let s = v.to_str().ok()?;
            Some(s.split(';').next().unwrap_or(s).to_string())
        })
        .collect()
}

/// Build a Cookie header string from a list of cookie strings.
fn join_cookies(cookies: &[String]) -> String {
    cookies.join("; ")
}

/// GET the login page, extract the CSRF token and session cookies.
///
/// Returns `(csrf_token, cookie_header)`.
pub async fn get_csrf_and_cookie(app: &Router) -> (String, String) {
    let req = Request::builder()
        .uri("/super/login")
        .body(Body::empty())
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    let cookies = extract_cookies(&resp);

    // Extract CSRF token from HTML
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8_lossy(&body);
    let csrf_token = html
        .split("name=\"csrf-token\" content=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .expect("No CSRF token found in login page HTML")
        .to_string();

    (csrf_token, join_cookies(&cookies))
}

/// Perform a full login and return `(csrf_token, cookie_header)`.
///
/// The CSRF token and cookies are from the same session so they can be
/// used together for subsequent mutating requests (e.g. logout).
pub async fn login(app: &Router) -> (String, String) {
    let (csrf_token, cookie) = get_csrf_and_cookie(app).await;

    let req = Request::builder()
        .uri("/super/api/login")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("X-CSRF-Token", &csrf_token)
        .header("Cookie", &cookie)
        .body(Body::from(
            r#"{"email":"admin@headspace.local","password":"admin123"}"#,
        ))
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    let new_cookies = extract_cookies(&resp);

    // Must consume the body so tower-sessions flushes the session to the store.
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("\"success\":true"),
        "Login should succeed, got: {body_str}"
    );

    let final_cookie = if new_cookies.is_empty() {
        cookie
    } else {
        join_cookies(&new_cookies)
    };

    (csrf_token, final_cookie)
}

/// Read the full response body as a String.
#[allow(dead_code)]
pub async fn body_string(resp: axum::response::Response) -> String {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8_lossy(&bytes).to_string()
}

/// Read the full response body as parsed JSON.
#[allow(dead_code)]
pub async fn body_json(resp: axum::response::Response) -> serde_json::Value {
    let s = body_string(resp).await;
    serde_json::from_str(&s).expect("Response body is not valid JSON")
}
