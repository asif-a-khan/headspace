use axum::extract::{Extension, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::models::company::Company;
use crate::models::tenant_admin::TenantUser;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Serialize)]
pub struct SearchResult {
    pub entity_type: &'static str,
    pub id: i64,
    pub title: String,
    pub subtitle: Option<String>,
    pub url: String,
}

pub async fn global_search(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(_user): Extension<TenantUser>,
    Query(query): Query<SearchQuery>,
) -> Response {
    let term = match query.q {
        Some(ref q) if q.len() >= 2 => q.clone(),
        _ => {
            let empty: Vec<SearchResult> = Vec::new();
            return Json(serde_json::json!({ "data": empty })).into_response();
        }
    };

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal error." })),
            )
                .into_response();
        }
    };

    let like = format!("%{term}%");
    let mut results: Vec<SearchResult> = Vec::new();

    // Search leads
    if let Ok(rows) = guard
        .fetch_all(
            sqlx::query_as::<_, (i64, String)>(
                "SELECT id, title FROM leads WHERE title ILIKE $1 ORDER BY id DESC LIMIT 5",
            )
            .bind(&like),
        )
        .await
    {
        for (id, title) in rows {
            results.push(SearchResult {
                entity_type: "Lead",
                id,
                title,
                subtitle: None,
                url: format!("/admin/leads/{id}"),
            });
        }
    }

    // Search persons
    if let Ok(rows) = guard
        .fetch_all(
            sqlx::query_as::<_, (i64, String)>(
                "SELECT id, name FROM persons WHERE name ILIKE $1 ORDER BY id DESC LIMIT 5",
            )
            .bind(&like),
        )
        .await
    {
        for (id, name) in rows {
            results.push(SearchResult {
                entity_type: "Person",
                id,
                title: name,
                subtitle: None,
                url: format!("/admin/contacts/persons/{id}"),
            });
        }
    }

    // Search products
    if let Ok(rows) = guard
        .fetch_all(
            sqlx::query_as::<_, (i64, String, String)>(
                "SELECT id, name, sku FROM products WHERE name ILIKE $1 OR sku ILIKE $1 ORDER BY id DESC LIMIT 5",
            )
            .bind(&like),
        )
        .await
    {
        for (id, name, sku) in rows {
            results.push(SearchResult {
                entity_type: "Product",
                id,
                title: name,
                subtitle: Some(sku),
                url: format!("/admin/products/{id}/edit"),
            });
        }
    }

    // Search quotes
    if let Ok(rows) = guard
        .fetch_all(
            sqlx::query_as::<_, (i64, String)>(
                "SELECT id, subject FROM quotes WHERE subject ILIKE $1 ORDER BY id DESC LIMIT 5",
            )
            .bind(&like),
        )
        .await
    {
        for (id, subject) in rows {
            results.push(SearchResult {
                entity_type: "Quote",
                id,
                title: subject,
                subtitle: None,
                url: format!("/admin/quotes/{id}/edit"),
            });
        }
    }

    // Search organizations
    if let Ok(rows) = guard
        .fetch_all(
            sqlx::query_as::<_, (i64, String)>(
                "SELECT id, name FROM organizations WHERE name ILIKE $1 ORDER BY id DESC LIMIT 5",
            )
            .bind(&like),
        )
        .await
    {
        for (id, name) in rows {
            results.push(SearchResult {
                entity_type: "Organization",
                id,
                title: name,
                subtitle: None,
                url: format!("/admin/contacts/organizations/{id}/edit"),
            });
        }
    }

    let _ = guard.release().await;

    Json(serde_json::json!({ "data": results })).into_response()
}
