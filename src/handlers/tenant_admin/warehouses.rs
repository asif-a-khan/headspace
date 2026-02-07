use axum::extract::{Extension, Path};
use axum::response::{IntoResponse, Response};
use tower_sessions::Session;

use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::middleware::csrf::get_csrf_token;
use crate::models::company::Company;
use crate::models::warehouse::Warehouse;
use crate::models::tenant_admin::TenantUser;
use crate::views::tenant_admin::{WarehouseIndex, WarehouseCreate, WarehouseEdit};

pub async fn index(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => return WarehouseIndex::new(csrf_token, "{}".to_string()).into_response(),
    };
    let warehouses = guard
        .fetch_all(sqlx::query_as::<_, Warehouse>("SELECT * FROM warehouses ORDER BY id"))
        .await
        .unwrap_or_default();
    let _ = guard.release().await;
    let initial_data = serde_json::json!({
        "warehouses": warehouses,
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    WarehouseIndex::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn create(
    session: Session,
    Extension(_db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();
    let initial_data = serde_json::json!({
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    WarehouseCreate::new(csrf_token, initial_data.to_string()).into_response()
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
        Err(_) => return WarehouseEdit::new(csrf_token, "{}".to_string()).into_response(),
    };
    let warehouse = guard
        .fetch_optional(sqlx::query_as::<_, Warehouse>("SELECT * FROM warehouses WHERE id = $1").bind(id))
        .await
        .ok()
        .flatten();
    let _ = guard.release().await;
    let initial_data = serde_json::json!({
        "warehouse": warehouse,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    WarehouseEdit::new(csrf_token, initial_data.to_string()).into_response()
}
