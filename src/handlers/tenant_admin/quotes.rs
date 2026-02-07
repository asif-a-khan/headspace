use axum::extract::{Extension, Path};
use axum::response::{IntoResponse, Response};
use tower_sessions::Session;

use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::middleware::csrf::get_csrf_token;
use crate::models::company::Company;
use crate::models::product::Product;
use crate::models::quote::{QuoteItem, QuoteRow};
use crate::models::tenant_admin::TenantUser;
use crate::views::tenant_admin::{QuoteCreate, QuoteEdit, QuoteIndex};

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
            return QuoteIndex::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let quotes = guard
        .fetch_all(sqlx::query_as::<_, QuoteRow>(
            "SELECT q.id, q.subject, q.grand_total, q.expired_at, q.person_id, q.user_id, q.created_at,
                    p.name AS person_name,
                    CONCAT(u.first_name, ' ', u.last_name) AS user_name
             FROM quotes q
             LEFT JOIN persons p ON p.id = q.person_id
             LEFT JOIN users u ON u.id = q.user_id
             ORDER BY q.id DESC",
        ))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "quotes": quotes,
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    QuoteIndex::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn create(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => {
            return QuoteCreate::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let persons = guard
        .fetch_all(sqlx::query_as::<_, crate::models::person::Person>(
            "SELECT * FROM persons ORDER BY name",
        ))
        .await
        .unwrap_or_default();

    let products = guard
        .fetch_all(sqlx::query_as::<_, Product>(
            "SELECT * FROM products ORDER BY name",
        ))
        .await
        .unwrap_or_default();

    let users = guard
        .fetch_all(sqlx::query_as::<_, TenantUser>(
            "SELECT * FROM users WHERE status = true ORDER BY first_name",
        ))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "persons": persons,
        "products": products,
        "users": users,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    QuoteCreate::new(csrf_token, initial_data.to_string()).into_response()
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
        Err(_) => {
            return QuoteEdit::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let quote = guard
        .fetch_optional(
            sqlx::query_as::<_, crate::models::quote::Quote>(
                "SELECT * FROM quotes WHERE id = $1",
            )
            .bind(id),
        )
        .await
        .ok()
        .flatten();

    let items = guard
        .fetch_all(
            sqlx::query_as::<_, QuoteItem>(
                "SELECT * FROM quote_items WHERE quote_id = $1 ORDER BY id",
            )
            .bind(id),
        )
        .await
        .unwrap_or_default();

    let persons = guard
        .fetch_all(sqlx::query_as::<_, crate::models::person::Person>(
            "SELECT * FROM persons ORDER BY name",
        ))
        .await
        .unwrap_or_default();

    let products = guard
        .fetch_all(sqlx::query_as::<_, Product>(
            "SELECT * FROM products ORDER BY name",
        ))
        .await
        .unwrap_or_default();

    let users = guard
        .fetch_all(sqlx::query_as::<_, TenantUser>(
            "SELECT * FROM users WHERE status = true ORDER BY first_name",
        ))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "quote": quote,
        "items": items,
        "persons": persons,
        "products": products,
        "users": users,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    QuoteEdit::new(csrf_token, initial_data.to_string()).into_response()
}
