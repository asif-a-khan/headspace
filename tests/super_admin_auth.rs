mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use common::{body_json, body_string, get_csrf_and_cookie, setup_test_app};

#[tokio::test]
async fn test_login_page_returns_200() {
    let app = setup_test_app().await;

    let req = Request::builder()
        .uri("/super/login")
        .body(Body::empty())
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body = body_string(resp).await;
    assert!(
        body.contains("name=\"csrf-token\" content=\""),
        "Login page should contain CSRF meta tag"
    );
    assert!(
        body.contains("__PAGE__"),
        "Login page should contain __PAGE__ variable"
    );
}

#[tokio::test]
async fn test_login_with_valid_credentials() {
    let app = setup_test_app().await;
    let (csrf_token, cookie) = get_csrf_and_cookie(&app).await;

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

    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_json(resp).await;
    assert_eq!(json["success"], true);
    assert_eq!(json["redirect"], "/super/tenants");
}

#[tokio::test]
async fn test_login_with_wrong_password() {
    let app = setup_test_app().await;
    let (csrf_token, cookie) = get_csrf_and_cookie(&app).await;

    let req = Request::builder()
        .uri("/super/api/login")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("X-CSRF-Token", &csrf_token)
        .header("Cookie", &cookie)
        .body(Body::from(
            r#"{"email":"admin@headspace.local","password":"wrongpassword"}"#,
        ))
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let json = body_json(resp).await;
    assert_eq!(json["error"], "Invalid credentials.");
}

#[tokio::test]
async fn test_login_with_nonexistent_email() {
    let app = setup_test_app().await;
    let (csrf_token, cookie) = get_csrf_and_cookie(&app).await;

    let req = Request::builder()
        .uri("/super/api/login")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("X-CSRF-Token", &csrf_token)
        .header("Cookie", &cookie)
        .body(Body::from(
            r#"{"email":"nobody@headspace.local","password":"admin123"}"#,
        ))
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let json = body_json(resp).await;
    assert_eq!(json["error"], "Invalid credentials.");
}

#[tokio::test]
async fn test_login_without_csrf_token() {
    let app = setup_test_app().await;
    let (_, cookie) = get_csrf_and_cookie(&app).await;

    let req = Request::builder()
        .uri("/super/api/login")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Cookie", &cookie)
        .body(Body::from(
            r#"{"email":"admin@headspace.local","password":"admin123"}"#,
        ))
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_protected_route_redirects_without_session() {
    let app = setup_test_app().await;

    let req = Request::builder()
        .uri("/super/tenants")
        .body(Body::empty())
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        resp.headers().get("location").unwrap().to_str().unwrap(),
        "/super/login"
    );
}

