use axum::extract::{Extension, Path};
use axum::response::{IntoResponse, Response};
use tower_sessions::Session;

use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::middleware::csrf::get_csrf_token;
use crate::models::company::Company;
use crate::models::pipeline::{LeadPipeline, LeadSource, LeadStage, LeadType, PipelineStageDetail};
use crate::models::tenant_admin::TenantUser;
use crate::views::tenant_admin::{
    PipelineCreate, PipelineEdit, PipelineIndex, SourceIndex, TypeIndex,
};

// -- Pipelines --

pub async fn pipelines_index(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => return PipelineIndex::new(csrf_token, "{}".to_string()).into_response(),
    };

    let pipelines = guard
        .fetch_all(sqlx::query_as::<_, LeadPipeline>("SELECT * FROM lead_pipelines ORDER BY id"))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "pipelines": pipelines,
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    PipelineIndex::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn pipelines_create(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => return PipelineCreate::new(csrf_token, "{}".to_string()).into_response(),
    };

    let stages = guard
        .fetch_all(sqlx::query_as::<_, LeadStage>("SELECT * FROM lead_stages ORDER BY id"))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "stages": stages,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    PipelineCreate::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn pipelines_edit(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => return PipelineEdit::new(csrf_token, "{}".to_string()).into_response(),
    };

    let pipeline = guard
        .fetch_optional(sqlx::query_as::<_, LeadPipeline>("SELECT * FROM lead_pipelines WHERE id = $1").bind(id))
        .await
        .ok()
        .flatten();

    let pipeline_stages = guard
        .fetch_all(sqlx::query_as::<_, PipelineStageDetail>(
            "SELECT ps.*, ls.code AS stage_code, ls.name AS stage_name
             FROM lead_pipeline_stages ps
             JOIN lead_stages ls ON ls.id = ps.lead_stage_id
             WHERE ps.lead_pipeline_id = $1
             ORDER BY ps.sort_order",
        ).bind(id))
        .await
        .unwrap_or_default();

    let stages = guard
        .fetch_all(sqlx::query_as::<_, LeadStage>("SELECT * FROM lead_stages ORDER BY id"))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "pipeline": pipeline,
        "pipeline_stages": pipeline_stages,
        "stages": stages,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    PipelineEdit::new(csrf_token, initial_data.to_string()).into_response()
}

// -- Sources --

pub async fn sources_index(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => return SourceIndex::new(csrf_token, "{}".to_string()).into_response(),
    };

    let sources = guard
        .fetch_all(sqlx::query_as::<_, LeadSource>("SELECT * FROM lead_sources ORDER BY id"))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "sources": sources,
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    SourceIndex::new(csrf_token, initial_data.to_string()).into_response()
}

// -- Types --

pub async fn types_index(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => return TypeIndex::new(csrf_token, "{}".to_string()).into_response(),
    };

    let types = guard
        .fetch_all(sqlx::query_as::<_, LeadType>("SELECT * FROM lead_types ORDER BY id"))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "types": types,
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    TypeIndex::new(csrf_token, initial_data.to_string()).into_response()
}
