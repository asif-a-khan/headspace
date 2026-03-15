//! Database migration runner.
//!
//! Handles main schema migrations, per-tenant schema creation, and
//! per-tenant table migrations. Called on application startup. Idempotent.

use std::path::Path;

use sqlx::PgPool;
use sqlx::migrate::Migrator;

use crate::models::company::Company;

/// Run all main schema migrations.
///
/// Creates the `main` schema if it doesn't exist, sets the search path,
/// then runs all migrations from `migrations/main/`.
pub async fn run_main_migrations(pool: &PgPool) -> anyhow::Result<()> {
    let mut conn = pool.acquire().await?;

    sqlx::query("CREATE SCHEMA IF NOT EXISTS main")
        .execute(&mut *conn)
        .await?;

    sqlx::query("SET search_path TO main")
        .execute(&mut *conn)
        .await?;

    sqlx::migrate!("migrations/main").run(&mut *conn).await?;

    // Reset search_path before returning the connection to the pool.
    // Without this, any code that later acquires this connection (e.g. the
    // session store) would search the wrong schema.
    sqlx::query("RESET search_path").execute(&mut *conn).await?;

    Ok(())
}

/// Create an empty PostgreSQL schema for a new tenant.
///
/// The schema name must contain only lowercase alphanumeric characters and
/// underscores. Tenant table migrations are applied separately when the
/// tenant panel is built.
pub async fn create_tenant_schema(pool: &PgPool, schema_name: &str) -> anyhow::Result<()> {
    validate_schema_name(schema_name)?;

    let sql = format!("CREATE SCHEMA IF NOT EXISTS {schema_name}");
    sqlx::query(&sql).execute(pool).await?;

    tracing::info!(schema = schema_name, "Tenant schema created");
    Ok(())
}

/// Run tenant table migrations for a single tenant schema.
///
/// Detaches a connection from the pool (not returned after use) to avoid
/// HRTB issues with `Migrator::run()`. This is safe because migrations
/// are infrequent (startup + new tenant creation). Each tenant schema gets
/// its own `_sqlx_migrations` tracking table.
pub async fn run_tenant_migrations(pool: &PgPool, schema_name: &str) -> anyhow::Result<()> {
    validate_schema_name(schema_name)?;

    let migrator = Migrator::new(Path::new("migrations/tenant")).await?;

    // Detach connection from pool so it's not returned with a modified search_path.
    // The pool creates a fresh replacement automatically.
    let pool_conn = pool.acquire().await?;
    let mut conn = pool_conn.detach();

    // Ensure the schema exists before setting search_path
    let create_sql = format!("CREATE SCHEMA IF NOT EXISTS {schema_name}");
    sqlx::query(&create_sql).execute(&mut conn).await?;

    let sql = format!("SET search_path TO {schema_name}, public");
    sqlx::query(&sql).execute(&mut conn).await?;

    migrator.run(&mut conn).await?;
    // conn is dropped here — closed, not returned to pool. That's fine for
    // infrequent migration operations.

    tracing::info!(schema = schema_name, "Tenant schema migrations applied");
    Ok(())
}

/// Run tenant migrations for all active tenants.
///
/// Queries the companies table and runs `run_tenant_migrations` for each.
pub async fn run_all_tenant_migrations(pool: &PgPool) -> anyhow::Result<()> {
    let tenants =
        sqlx::query_as::<_, Company>("SELECT * FROM main.companies WHERE is_active = true")
            .fetch_all(pool)
            .await?;

    for tenant in &tenants {
        if let Err(e) = run_tenant_migrations(pool, &tenant.schema_name).await {
            tracing::error!(
                schema = %tenant.schema_name,
                error = %e,
                "Failed to run tenant migrations"
            );
        }
    }

    Ok(())
}

/// Set up a newly created tenant: run migrations and seed default admin.
///
/// Called from tenant creation handler. Logs errors but does not fail
/// (the tenant record is already created).
pub async fn setup_new_tenant(pool: &PgPool, schema_name: &str, domain: &str) {
    if let Err(e) = run_tenant_migrations(pool, schema_name).await {
        tracing::error!("Failed to run tenant migrations: {e}");
        return;
    }
    if let Err(e) = super::seed::seed_default_tenant_admin(pool, schema_name, domain).await {
        tracing::error!("Failed to seed tenant admin: {e}");
    }
}

/// Validate that a schema name contains only safe characters.
pub(crate) fn validate_schema_name(name: &str) -> anyhow::Result<()> {
    if name.is_empty() {
        anyhow::bail!("Schema name cannot be empty");
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        anyhow::bail!(
            "Schema name '{name}' contains invalid characters (only lowercase alphanumeric and underscores allowed)"
        );
    }
    if name.starts_with('_') || name.chars().next().unwrap().is_ascii_digit() {
        anyhow::bail!("Schema name '{name}' must start with a lowercase letter");
    }
    Ok(())
}
