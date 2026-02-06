//! Tenant context management.
//!
//! Provides `set_tenant()` to scope database connections to a specific
//! tenant schema via `SET search_path`.

use sqlx::PgConnection;

/// Set the search path for a connection to a specific tenant schema.
///
/// This must be called on each connection before executing tenant-scoped
/// queries. The schema name is validated to prevent SQL injection.
pub async fn set_tenant(conn: &mut PgConnection, schema_name: &str) -> anyhow::Result<()> {
    super::migrate::validate_schema_name(schema_name)?;

    let sql = format!("SET search_path TO {schema_name}, public");
    sqlx::query(&sql).execute(&mut *conn).await?;

    Ok(())
}
