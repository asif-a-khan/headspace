use axum::extract::{Extension, Path};
use axum::response::{IntoResponse, Response};
use tower_sessions::Session;

use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::middleware::csrf::get_csrf_token;
use crate::models::company::Company;
use crate::models::organization::OrganizationRow;
use crate::models::person::PersonRow;
use crate::models::tenant_admin::TenantUser;
use crate::views::tenant_admin::{
    OrganizationCreate, OrganizationEdit, OrganizationIndex, OrganizationShow, PersonCreate,
    PersonEdit, PersonIndex, PersonShow,
};

use crate::api::tenant_admin::contacts::view_permission_filter;

// -- Persons --

pub async fn persons_index(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => {
            return PersonIndex::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let vp = view_permission_filter(user.id, &user.view_permission);
    let sql = format!(
        "SELECT t.*, o.name AS organization_name,
                CONCAT(u.first_name, ' ', u.last_name) AS user_name
         FROM persons t
         LEFT JOIN organizations o ON o.id = t.organization_id
         LEFT JOIN users u ON u.id = t.user_id
         WHERE true{vp}
         ORDER BY t.id DESC"
    );

    let persons = guard
        .fetch_all(sqlx::query_as::<_, PersonRow>(&sql))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "persons": persons,
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    PersonIndex::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn persons_create(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => {
            return PersonCreate::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let orgs = guard
        .fetch_all(sqlx::query_as::<_, crate::models::organization::Organization>(
            "SELECT * FROM organizations ORDER BY name",
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
        "organizations": orgs,
        "users": users,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    PersonCreate::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn persons_edit(
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
            return PersonEdit::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let person = guard
        .fetch_optional(
            sqlx::query_as::<_, crate::models::person::Person>(
                "SELECT * FROM persons WHERE id = $1",
            )
            .bind(id),
        )
        .await
        .ok()
        .flatten();

    let orgs = guard
        .fetch_all(sqlx::query_as::<_, crate::models::organization::Organization>(
            "SELECT * FROM organizations ORDER BY name",
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
        "person": person,
        "organizations": orgs,
        "users": users,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    PersonEdit::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn persons_show(
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
            return PersonShow::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let person = guard
        .fetch_optional(
            sqlx::query_as::<_, crate::models::person::Person>(
                "SELECT * FROM persons WHERE id = $1",
            )
            .bind(id),
        )
        .await
        .ok()
        .flatten();

    if person.is_none() {
        let _ = guard.release().await;
        return PersonShow::new(csrf_token, "{}".to_string()).into_response();
    }
    let person = person.unwrap();

    // Organization
    let organization: Option<crate::models::organization::Organization> =
        if let Some(oid) = person.organization_id {
            guard
                .fetch_optional(
                    sqlx::query_as::<_, crate::models::organization::Organization>(
                        "SELECT * FROM organizations WHERE id = $1",
                    )
                    .bind(oid),
                )
                .await
                .ok()
                .flatten()
        } else {
            None
        };

    // Assigned user name
    let user_name: Option<String> = if let Some(uid) = person.user_id {
        guard
            .fetch_optional(
                sqlx::query_as::<_, (String,)>(
                    "SELECT CONCAT(first_name, ' ', last_name) FROM users WHERE id = $1",
                )
                .bind(uid),
            )
            .await
            .ok()
            .flatten()
            .map(|(n,)| n)
    } else {
        None
    };

    // Activities linked to this person
    let activities = guard
        .fetch_all(
            sqlx::query_as::<_, crate::models::activity::Activity>(
                "SELECT * FROM activities WHERE id IN (
                     SELECT activity_id FROM person_activities WHERE person_id = $1
                 ) ORDER BY created_at DESC",
            )
            .bind(id),
        )
        .await
        .unwrap_or_default();

    // Leads linked to this person
    let leads = guard
        .fetch_all(
            sqlx::query_as::<_, crate::models::lead::LeadRow>(
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
                 WHERE l.person_id = $1
                 ORDER BY l.id DESC",
            )
            .bind(id),
        )
        .await
        .unwrap_or_default();

    // Tags
    let tags = guard
        .fetch_all(
            sqlx::query_as::<_, crate::models::tag::Tag>(
                "SELECT t.* FROM tags t
                 JOIN person_tags pt ON pt.tag_id = t.id
                 WHERE pt.person_id = $1
                 ORDER BY t.name",
            )
            .bind(id),
        )
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "person": person,
        "organization": organization,
        "user_name": user_name,
        "activities": activities,
        "leads": leads,
        "tags": tags,
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    PersonShow::new(csrf_token, initial_data.to_string()).into_response()
}

// -- Organizations --

pub async fn organizations_index(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => {
            return OrganizationIndex::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let vp = view_permission_filter(user.id, &user.view_permission);
    let sql = format!(
        "SELECT t.*, CONCAT(u.first_name, ' ', u.last_name) AS user_name
         FROM organizations t
         LEFT JOIN users u ON u.id = t.user_id
         WHERE true{vp}
         ORDER BY t.id DESC"
    );

    let orgs = guard
        .fetch_all(sqlx::query_as::<_, OrganizationRow>(&sql))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "organizations": orgs,
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    OrganizationIndex::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn organizations_create(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    let csrf_token = get_csrf_token(&session).await.unwrap_or_default();

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => {
            return OrganizationCreate::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let users = guard
        .fetch_all(sqlx::query_as::<_, TenantUser>(
            "SELECT * FROM users WHERE status = true ORDER BY first_name",
        ))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "users": users,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    OrganizationCreate::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn organizations_edit(
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
            return OrganizationEdit::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let org = guard
        .fetch_optional(
            sqlx::query_as::<_, crate::models::organization::Organization>(
                "SELECT * FROM organizations WHERE id = $1",
            )
            .bind(id),
        )
        .await
        .ok()
        .flatten();

    let users = guard
        .fetch_all(sqlx::query_as::<_, TenantUser>(
            "SELECT * FROM users WHERE status = true ORDER BY first_name",
        ))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "organization": org,
        "users": users,
        "admin_name": user.full_name(),
        "company_name": company.name,
    });
    OrganizationEdit::new(csrf_token, initial_data.to_string()).into_response()
}

pub async fn organizations_show(
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
            return OrganizationShow::new(csrf_token, "{}".to_string()).into_response();
        }
    };

    let org = guard
        .fetch_optional(
            sqlx::query_as::<_, crate::models::organization::Organization>(
                "SELECT * FROM organizations WHERE id = $1",
            )
            .bind(id),
        )
        .await
        .ok()
        .flatten();

    if org.is_none() {
        let _ = guard.release().await;
        return OrganizationShow::new(csrf_token, "{}".to_string()).into_response();
    }
    let org = org.unwrap();

    // Assigned user name
    let user_name: Option<String> = if let Some(uid) = org.user_id {
        guard
            .fetch_optional(
                sqlx::query_as::<_, (String,)>(
                    "SELECT CONCAT(first_name, ' ', last_name) FROM users WHERE id = $1",
                )
                .bind(uid),
            )
            .await
            .ok()
            .flatten()
            .map(|(n,)| n)
    } else {
        None
    };

    // Persons linked to this org
    let persons = guard
        .fetch_all(
            sqlx::query_as::<_, crate::models::person::Person>(
                "SELECT * FROM persons WHERE organization_id = $1 ORDER BY name",
            )
            .bind(id),
        )
        .await
        .unwrap_or_default();

    // Leads for persons in this org
    let leads = guard
        .fetch_all(
            sqlx::query_as::<_, crate::models::lead::LeadRow>(
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
                 WHERE l.person_id IN (SELECT id FROM persons WHERE organization_id = $1)
                 ORDER BY l.id DESC",
            )
            .bind(id),
        )
        .await
        .unwrap_or_default();

    // Activities for persons in this org
    let activities = guard
        .fetch_all(
            sqlx::query_as::<_, crate::models::activity::Activity>(
                "SELECT * FROM activities WHERE id IN (
                     SELECT activity_id FROM person_activities
                     WHERE person_id IN (SELECT id FROM persons WHERE organization_id = $1)
                 ) ORDER BY created_at DESC",
            )
            .bind(id),
        )
        .await
        .unwrap_or_default();

    // Tags
    let tags = guard
        .fetch_all(
            sqlx::query_as::<_, crate::models::tag::Tag>(
                "SELECT t.* FROM tags t
                 JOIN organization_tags ot ON ot.tag_id = t.id
                 WHERE ot.organization_id = $1
                 ORDER BY t.name",
            )
            .bind(id),
        )
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    let initial_data = serde_json::json!({
        "organization": org,
        "user_name": user_name,
        "persons": persons,
        "leads": leads,
        "activities": activities,
        "tags": tags,
        "admin_name": user.full_name(),
        "company_name": company.name,
        "permission_type": user.permission_type,
        "permissions": user.role_permissions,
    });
    OrganizationShow::new(csrf_token, initial_data.to_string()).into_response()
}
