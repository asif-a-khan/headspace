//! Database seeder.
//!
//! Seeds default data on first run (empty database). Idempotent — does nothing
//! if data already exists.

use sqlx::PgPool;

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
        tracing::debug!(schema = schema_name, "Tenant admin already exists, skipping seed");
        return Ok(());
    }

    tracing::info!(schema = schema_name, "No tenant admins found — seeding default data");

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

    // conn is dropped here — closed, not returned to pool.

    tracing::info!(
        schema = schema_name,
        email = %email,
        password = "admin123",
        "Default tenant admin seeded"
    );

    Ok(())
}

/// Seed default tenant admins for all active tenants.
pub async fn seed_all_tenant_admins(pool: &PgPool) -> anyhow::Result<()> {
    let tenants = sqlx::query_as::<_, Company>(
        "SELECT * FROM main.companies WHERE is_active = true",
    )
    .fetch_all(pool)
    .await?;

    for tenant in &tenants {
        if let Err(e) =
            seed_default_tenant_admin(pool, &tenant.schema_name, &tenant.domain).await
        {
            tracing::error!(
                schema = %tenant.schema_name,
                error = %e,
                "Failed to seed tenant admin"
            );
        }
    }

    Ok(())
}
