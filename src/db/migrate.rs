//! Database migration runner.
//!
//! Handles both main schema migrations and per-tenant schema creation.
//! Called on application startup. Idempotent.

use sqlx::PgPool;

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

    sqlx::migrate!("migrations/main")
        .run(&mut *conn)
        .await?;

    // Reset search_path before returning the connection to the pool.
    // Without this, any code that later acquires this connection (e.g. the
    // session store) would search the wrong schema.
    sqlx::query("RESET search_path")
        .execute(&mut *conn)
        .await?;

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
