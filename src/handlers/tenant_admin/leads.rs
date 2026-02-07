use axum::extract::{Extension, Path};
use axum::response::{IntoResponse, Response};
use tower_sessions::Session;

use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::middleware::csrf::get_csrf_token;
use crate::models::company::Company;
use crate::models::lead::LeadRow;
use crate::models::pipeline::{LeadPipeline, LeadSource, LeadType, PipelineStageDetail};
use crate::models::tenant_admin::TenantUser;
use crate::views::tenant_admin::{LeadCreate, LeadEdit, LeadIndex, LeadKanbanView};

use crate::api::tenant_admin::contacts::view_permission_filter;

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
            return LeadIndex::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let vp = view_permission_filter(user.id, &user.view_permission);
    let sql = format!(
        "SELECT l.*,
                p.name AS person_name,
                CONCAT(u.first_name, ' ', u.last_name) AS user_name,
                ls.name AS source_name,
                lt.name AS type_name,
                stg.name AS stage_name,
                pip.name AS pipeline_name
         FROM leads l
         LEFT JOIN persons p ON p.id = l.person_id
         LEFT JOIN users u ON u.id = l.user_id
         LEFT JOIN lead_sources ls ON ls.id = l.lead_source_id
         LEFT JOIN lead_types lt ON lt.id = l.lead_type_id
         LEFT JOIN lead_pipeline_stages lps ON lps.id = l.lead_pipeline_stage_id
         LEFT JOIN lead_stages stg ON stg.id = lps.lead_stage_id
         LEFT JOIN lead_pipelines pip ON pip.id = l.lead_pipeline_id
         WHERE true{vp}
         ORDER BY l.id DESC",
        vp = vp.replace("t.user_id", "l.user_id"),
    );

    let leads = guard
        .fetch_all(sqlx::query_as::<_, LeadRow>(&sql))
        .await
        .unwrap_or_default();

    let pipelines = guard
        .fetch_all(sqlx::query_as::<_, LeadPipeline>(
            "SELECT * FROM lead_pipelines ORDER BY name",
        ))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "leads": leads,
        "pipelines": pipelines,
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    LeadIndex::new(csrf_token, initial_data.to_string()).into_response()
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
            return LeadCreate::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let pipelines = guard
        .fetch_all(sqlx::query_as::<_, LeadPipeline>(
            "SELECT * FROM lead_pipelines ORDER BY name",
        ))
        .await
        .unwrap_or_default();

    let stages = guard
        .fetch_all(sqlx::query_as::<_, PipelineStageDetail>(
            "SELECT lps.*, s.code AS stage_code, s.name AS stage_name
             FROM lead_pipeline_stages lps
             JOIN lead_stages s ON s.id = lps.lead_stage_id
             ORDER BY lps.sort_order",
        ))
        .await
        .unwrap_or_default();

    let sources = guard
        .fetch_all(sqlx::query_as::<_, LeadSource>(
            "SELECT * FROM lead_sources ORDER BY name",
        ))
        .await
        .unwrap_or_default();

    let types = guard
        .fetch_all(sqlx::query_as::<_, LeadType>(
            "SELECT * FROM lead_types ORDER BY name",
        ))
        .await
        .unwrap_or_default();

    let persons = guard
        .fetch_all(sqlx::query_as::<_, crate::models::person::Person>(
            "SELECT * FROM persons ORDER BY name",
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
        "pipelines": pipelines,
        "stages": stages,
        "sources": sources,
        "types": types,
        "persons": persons,
        "users": users,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    LeadCreate::new(csrf_token, initial_data.to_string()).into_response()
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
            return LeadEdit::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let lead = guard
        .fetch_optional(
            sqlx::query_as::<_, crate::models::lead::Lead>(
                "SELECT * FROM leads WHERE id = $1",
            )
            .bind(id),
        )
        .await
        .ok()
        .flatten();

    let pipelines = guard
        .fetch_all(sqlx::query_as::<_, LeadPipeline>(
            "SELECT * FROM lead_pipelines ORDER BY name",
        ))
        .await
        .unwrap_or_default();

    let stages = guard
        .fetch_all(sqlx::query_as::<_, PipelineStageDetail>(
            "SELECT lps.*, s.code AS stage_code, s.name AS stage_name
             FROM lead_pipeline_stages lps
             JOIN lead_stages s ON s.id = lps.lead_stage_id
             ORDER BY lps.sort_order",
        ))
        .await
        .unwrap_or_default();

    let sources = guard
        .fetch_all(sqlx::query_as::<_, LeadSource>(
            "SELECT * FROM lead_sources ORDER BY name",
        ))
        .await
        .unwrap_or_default();

    let types = guard
        .fetch_all(sqlx::query_as::<_, LeadType>(
            "SELECT * FROM lead_types ORDER BY name",
        ))
        .await
        .unwrap_or_default();

    let persons = guard
        .fetch_all(sqlx::query_as::<_, crate::models::person::Person>(
            "SELECT * FROM persons ORDER BY name",
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
        "lead": lead,
        "pipelines": pipelines,
        "stages": stages,
        "sources": sources,
        "types": types,
        "persons": persons,
        "users": users,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    LeadEdit::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn kanban_page(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => {
            return LeadKanbanView::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let pipelines = guard
        .fetch_all(sqlx::query_as::<_, LeadPipeline>(
            "SELECT * FROM lead_pipelines ORDER BY name",
        ))
        .await
        .unwrap_or_default();

    let stages = guard
        .fetch_all(sqlx::query_as::<_, PipelineStageDetail>(
            "SELECT lps.*, s.code AS stage_code, s.name AS stage_name
             FROM lead_pipeline_stages lps
             JOIN lead_stages s ON s.id = lps.lead_stage_id
             ORDER BY lps.sort_order",
        ))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "pipelines": pipelines,
        "stages": stages,
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    LeadKanbanView::new(csrf_token, initial_data.to_string()).into_response()
}
