//! Database seeder.
//!
//! Seeds default data on first run (empty database). Idempotent — does nothing
//! if data already exists.

use sqlx::{PgConnection, PgPool};

use crate::auth::password::hash_password;
use crate::models::company::Company;

/// Seed default super admin data if the database is empty.
///
/// Creates a default "Administrator" role with all permissions and a default
/// super admin account. Only runs if no super admins exist yet.
pub async fn seed_default_super_admin(pool: &PgPool) -> anyhow::Result<()> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM main.super_admins")
        .fetch_one(pool)
        .await?;

    if count.0 > 0 {
        tracing::debug!("Super admins already exist, skipping seed");
        return Ok(());
    }

    tracing::info!("No super admins found — seeding default data");

    // Create default administrator role
    let role_id: (i64,) = sqlx::query_as(
        "INSERT INTO main.super_roles (name, description, permission_type, permissions)
         VALUES ('Administrator', 'Full access to all super admin features', 'all', '[]'::jsonb)
         RETURNING id",
    )
    .fetch_one(pool)
    .await?;

    // Create default super admin
    let password_hash = hash_password("admin123")?;

    sqlx::query(
        "INSERT INTO main.super_admins (first_name, last_name, email, password_hash, role_id, status)
         VALUES ('Super', 'Admin', 'admin@headspace.local', $1, $2, true)",
    )
    .bind(&password_hash)
    .bind(role_id.0)
    .execute(pool)
    .await?;

    tracing::info!(
        email = "admin@headspace.local",
        password = "admin123",
        "Default super admin seeded"
    );

    Ok(())
}

/// Seed a demo tenant company if no companies exist yet.
///
/// Creates a "Demo" company with domain "demo" and schema "tenant_demo".
/// Only runs if the companies table is empty. The tenant schema/migrations/seeding
/// are handled separately by the startup sequence.
pub async fn seed_demo_tenant(pool: &PgPool) -> anyhow::Result<()> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM main.companies")
        .fetch_one(pool)
        .await?;

    if count.0 > 0 {
        tracing::debug!("Companies already exist, skipping demo tenant seed");
        return Ok(());
    }

    tracing::info!("No companies found — seeding demo tenant");

    sqlx::query(
        "INSERT INTO main.companies (name, domain, schema_name, is_active)
         VALUES ('Demo', 'demo', 'tenant_demo', true)",
    )
    .execute(pool)
    .await?;

    tracing::info!(
        domain = "demo",
        schema = "tenant_demo",
        "Demo tenant company seeded"
    );

    Ok(())
}

/// Seed default tenant admin data for a single tenant if none exists.
///
/// Creates a default "Administrator" role with all permissions and a default
/// admin user. Only runs if no users exist in the tenant schema yet.
///
/// Uses `detach()` to remove the connection from the pool after use. This
/// avoids HRTB issues with sqlx when the future must be `Send` (axum handlers).
/// The pool creates a fresh replacement automatically. Acceptable overhead
/// since seeding only runs on tenant creation and startup.
pub async fn seed_default_tenant_admin(
    pool: &PgPool,
    schema_name: &str,
    domain: &str,
) -> anyhow::Result<()> {
    super::migrate::validate_schema_name(schema_name)?;

    let pool_conn = pool.acquire().await?;
    let mut conn = pool_conn.detach();

    let sql = format!("SET search_path TO {schema_name}, public");
    sqlx::query(&sql).execute(&mut conn).await?;

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&mut conn)
        .await?;

    if count.0 > 0 {
        tracing::debug!(
            schema = schema_name,
            "Tenant admin already exists, skipping seed"
        );
        return Ok(());
    }

    tracing::info!(
        schema = schema_name,
        "No tenant admins found — seeding default data"
    );

    let role_id: (i64,) = sqlx::query_as(
        "INSERT INTO roles (name, description, permission_type, permissions)
         VALUES ('Administrator', 'Full access to all tenant features', 'all', '[]'::jsonb)
         RETURNING id",
    )
    .fetch_one(&mut conn)
    .await?;

    let email = format!("admin@{domain}.headspace.local");
    let password_hash = hash_password("admin123")?;

    let user_id: (i64,) = sqlx::query_as(
        "INSERT INTO users (first_name, last_name, email, password_hash, role_id, status)
         VALUES ('Admin', $1, $2, $3, $4, true)
         RETURNING id",
    )
    .bind(domain)
    .bind(&email)
    .bind(&password_hash)
    .bind(role_id.0)
    .fetch_one(&mut conn)
    .await?;

    // Create default "General" group and assign admin to it
    let group_id: (i64,) = sqlx::query_as(
        "INSERT INTO groups (name, description) VALUES ('General', 'Default group') RETURNING id",
    )
    .fetch_one(&mut conn)
    .await?;

    sqlx::query("INSERT INTO user_groups (user_id, group_id) VALUES ($1, $2)")
        .bind(user_id.0)
        .bind(group_id.0)
        .execute(&mut conn)
        .await?;

    // Seed system-defined attributes for each entity type
    seed_system_attributes(&mut conn).await?;

    // Seed default pipeline configuration
    seed_pipeline_defaults(&mut conn).await?;

    // Seed default tags
    seed_default_tags(&mut conn, user_id.0).await?;

    // conn is dropped here — closed, not returned to pool.

    tracing::info!(
        schema = schema_name,
        email = %email,
        password = "admin123",
        "Default tenant admin seeded"
    );

    Ok(())
}

/// Seed system-defined attributes for all entity types.
///
/// These define metadata about core entity columns (is_required, validation, etc.)
/// and are not editable/deletable by tenant admins. Only runs if no attributes exist yet.
/// Matches Krayin CRM's default attribute set (41 attrs across 6 entity types, minus
/// warehouses which are not implemented).
async fn seed_system_attributes(conn: &mut PgConnection) -> anyhow::Result<()> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM attributes")
        .fetch_one(&mut *conn)
        .await?;

    if count.0 > 0 {
        return Ok(());
    }

    // (code, name, type, entity_type, sort_order, validation, is_required, is_unique, quick_add, lookup_type)
    #[allow(clippy::type_complexity)]
    let attrs: &[(
        &str,
        &str,
        &str,
        &str,
        i32,
        Option<&str>,
        bool,
        bool,
        bool,
        Option<&str>,
    )] = &[
        // Leads (9 attributes — matches Krayin)
        (
            "title", "Title", "text", "leads", 1, None, true, false, true, None,
        ),
        (
            "description",
            "Description",
            "textarea",
            "leads",
            2,
            None,
            false,
            false,
            true,
            None,
        ),
        (
            "lead_value",
            "Lead Value",
            "price",
            "leads",
            3,
            Some("decimal"),
            true,
            false,
            true,
            None,
        ),
        (
            "lead_source_id",
            "Source",
            "select",
            "leads",
            4,
            None,
            true,
            false,
            true,
            Some("lead_sources"),
        ),
        (
            "lead_type_id",
            "Type",
            "select",
            "leads",
            5,
            None,
            true,
            false,
            true,
            Some("lead_types"),
        ),
        (
            "user_id",
            "Sales Owner",
            "select",
            "leads",
            7,
            None,
            false,
            false,
            true,
            Some("users"),
        ),
        (
            "expected_close_date",
            "Expected Close Date",
            "date",
            "leads",
            8,
            None,
            false,
            false,
            true,
            None,
        ),
        (
            "lead_pipeline_id",
            "Pipeline",
            "lookup",
            "leads",
            9,
            None,
            true,
            false,
            true,
            Some("lead_pipelines"),
        ),
        (
            "lead_pipeline_stage_id",
            "Stage",
            "lookup",
            "leads",
            10,
            None,
            true,
            false,
            true,
            Some("lead_pipeline_stages"),
        ),
        // Persons (6 attributes — matches Krayin)
        (
            "name", "Name", "text", "persons", 1, None, true, false, true, None,
        ),
        (
            "emails", "Emails", "email", "persons", 2, None, true, true, true, None,
        ),
        (
            "contact_numbers",
            "Contact Numbers",
            "phone",
            "persons",
            3,
            Some("numeric"),
            false,
            true,
            true,
            None,
        ),
        (
            "job_title",
            "Job Title",
            "text",
            "persons",
            4,
            None,
            false,
            false,
            true,
            None,
        ),
        (
            "user_id",
            "Sales Owner",
            "lookup",
            "persons",
            5,
            None,
            false,
            false,
            true,
            Some("users"),
        ),
        (
            "organization_id",
            "Organization",
            "lookup",
            "persons",
            6,
            None,
            false,
            false,
            true,
            Some("organizations"),
        ),
        // Organizations (3 attributes — matches Krayin)
        (
            "name",
            "Name",
            "text",
            "organizations",
            1,
            None,
            true,
            true,
            true,
            None,
        ),
        (
            "address",
            "Address",
            "address",
            "organizations",
            2,
            None,
            false,
            false,
            true,
            None,
        ),
        (
            "user_id",
            "Sales Owner",
            "lookup",
            "organizations",
            3,
            None,
            false,
            false,
            true,
            Some("users"),
        ),
        // Products (5 attributes — matches Krayin)
        (
            "name", "Name", "text", "products", 1, None, true, false, true, None,
        ),
        (
            "description",
            "Description",
            "textarea",
            "products",
            2,
            None,
            false,
            false,
            true,
            None,
        ),
        (
            "sku", "SKU", "text", "products", 3, None, true, true, true, None,
        ),
        (
            "quantity",
            "Quantity",
            "text",
            "products",
            4,
            Some("numeric"),
            true,
            false,
            true,
            None,
        ),
        (
            "price",
            "Price",
            "price",
            "products",
            5,
            Some("decimal"),
            true,
            false,
            true,
            None,
        ),
        // Quotes (13 attributes — matches Krayin)
        (
            "user_id",
            "Sales Owner",
            "select",
            "quotes",
            1,
            None,
            true,
            false,
            true,
            Some("users"),
        ),
        (
            "subject", "Subject", "text", "quotes", 2, None, true, false, true, None,
        ),
        (
            "description",
            "Description",
            "textarea",
            "quotes",
            3,
            None,
            false,
            false,
            true,
            None,
        ),
        (
            "billing_address",
            "Billing Address",
            "address",
            "quotes",
            4,
            None,
            true,
            false,
            true,
            None,
        ),
        (
            "shipping_address",
            "Shipping Address",
            "address",
            "quotes",
            5,
            None,
            false,
            false,
            true,
            None,
        ),
        (
            "discount_percent",
            "Discount Percent",
            "text",
            "quotes",
            6,
            Some("decimal"),
            false,
            false,
            true,
            None,
        ),
        (
            "discount_amount",
            "Discount Amount",
            "price",
            "quotes",
            7,
            Some("decimal"),
            false,
            false,
            true,
            None,
        ),
        (
            "tax_amount",
            "Tax Amount",
            "price",
            "quotes",
            8,
            Some("decimal"),
            false,
            false,
            true,
            None,
        ),
        (
            "adjustment_amount",
            "Adjustment Amount",
            "price",
            "quotes",
            9,
            Some("decimal"),
            false,
            false,
            true,
            None,
        ),
        (
            "sub_total",
            "Sub Total",
            "price",
            "quotes",
            10,
            Some("decimal"),
            true,
            false,
            true,
            None,
        ),
        (
            "grand_total",
            "Grand Total",
            "price",
            "quotes",
            11,
            Some("decimal"),
            true,
            false,
            true,
            None,
        ),
        (
            "expired_at",
            "Expired At",
            "date",
            "quotes",
            12,
            None,
            true,
            false,
            true,
            None,
        ),
        (
            "person_id",
            "Person",
            "lookup",
            "quotes",
            13,
            None,
            true,
            false,
            true,
            Some("persons"),
        ),
    ];

    for (
        code,
        name,
        attr_type,
        entity_type,
        sort_order,
        validation,
        is_required,
        is_unique,
        quick_add,
        lookup_type,
    ) in attrs
    {
        sqlx::query(
            "INSERT INTO attributes (code, name, type, entity_type, sort_order, validation, is_required, is_unique, quick_add, is_user_defined, lookup_type)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, false, $10)",
        )
        .bind(code)
        .bind(name)
        .bind(attr_type)
        .bind(entity_type)
        .bind(sort_order)
        .bind(validation)
        .bind(is_required)
        .bind(is_unique)
        .bind(quick_add)
        .bind(lookup_type)
        .execute(&mut *conn)
        .await?;
    }

    tracing::info!("System-defined attributes seeded (36 attributes)");
    Ok(())
}

/// Seed default pipeline configuration: sources, types, stages, and default pipeline.
async fn seed_pipeline_defaults(conn: &mut PgConnection) -> anyhow::Result<()> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM lead_pipelines")
        .fetch_one(&mut *conn)
        .await?;

    if count.0 > 0 {
        return Ok(());
    }

    // Lead sources
    for name in &["Email", "Web", "Web Form", "Phone", "Direct"] {
        sqlx::query("INSERT INTO lead_sources (name) VALUES ($1)")
            .bind(name)
            .execute(&mut *conn)
            .await?;
    }

    // Lead types
    for name in &["New Business", "Existing Business"] {
        sqlx::query("INSERT INTO lead_types (name) VALUES ($1)")
            .bind(name)
            .execute(&mut *conn)
            .await?;
    }

    // Lead stages (system-defined)
    // (code, name, is_user_defined)
    let stages: &[(&str, &str, bool)] = &[
        ("new", "New", false),
        ("follow-up", "Follow Up", false),
        ("prospect", "Prospect", false),
        ("negotiation", "Negotiation", false),
        ("won", "Won", false),
        ("lost", "Lost", false),
    ];

    let mut stage_ids = Vec::new();
    for (code, name, is_user_defined) in stages {
        let row: (i64,) = sqlx::query_as(
            "INSERT INTO lead_stages (code, name, is_user_defined) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(code)
        .bind(name)
        .bind(is_user_defined)
        .fetch_one(&mut *conn)
        .await?;
        stage_ids.push(row.0);
    }

    // Default pipeline
    let pipeline: (i64,) = sqlx::query_as(
        "INSERT INTO lead_pipelines (name, is_default, rotten_days) VALUES ('Default', true, 30) RETURNING id",
    )
    .fetch_one(&mut *conn)
    .await?;

    // Link stages to pipeline with probabilities
    // (stage_index, probability, sort_order)
    let pipeline_stages: &[(usize, i32, i32)] = &[
        (0, 100, 1), // New
        (1, 100, 2), // Follow Up
        (2, 100, 3), // Prospect
        (3, 100, 4), // Negotiation
        (4, 100, 5), // Won
        (5, 0, 6),   // Lost
    ];

    for (stage_idx, probability, sort_order) in pipeline_stages {
        sqlx::query(
            "INSERT INTO lead_pipeline_stages (lead_pipeline_id, lead_stage_id, probability, sort_order)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(pipeline.0)
        .bind(stage_ids[*stage_idx])
        .bind(probability)
        .bind(sort_order)
        .execute(&mut *conn)
        .await?;
    }

    tracing::info!("Pipeline defaults seeded (sources, types, stages, default pipeline)");
    Ok(())
}

/// Seed default tags for common CRM categorization.
async fn seed_default_tags(conn: &mut PgConnection, user_id: i64) -> anyhow::Result<()> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tags")
        .fetch_one(&mut *conn)
        .await?;

    if count.0 > 0 {
        return Ok(());
    }

    let tags: &[(&str, &str)] = &[
        ("Hot", "#ef4444"),
        ("Warm", "#f97316"),
        ("Cold", "#3b82f6"),
        ("VIP", "#8b5cf6"),
        ("Follow Up", "#eab308"),
    ];

    for (name, color) in tags {
        sqlx::query("INSERT INTO tags (name, color, user_id) VALUES ($1, $2, $3)")
            .bind(name)
            .bind(color)
            .bind(user_id)
            .execute(&mut *conn)
            .await?;
    }

    tracing::info!("Default tags seeded (5 tags)");
    Ok(())
}

/// Seed default tenant admins for all active tenants.
pub async fn seed_all_tenant_admins(pool: &PgPool) -> anyhow::Result<()> {
    let tenants =
        sqlx::query_as::<_, Company>("SELECT * FROM main.companies WHERE is_active = true")
            .fetch_all(pool)
            .await?;

    for tenant in &tenants {
        if let Err(e) = seed_default_tenant_admin(pool, &tenant.schema_name, &tenant.domain).await {
            tracing::error!(
                schema = %tenant.schema_name,
                error = %e,
                "Failed to seed tenant admin"
            );
        }
    }

    Ok(())
}
