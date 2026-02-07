//! Tenant context management.
//!
//! Provides `set_tenant()` and `reset_tenant()` to scope database connections
//! to a specific tenant schema via `SET search_path`.
//!
//! **Usage pattern** for tenant-scoped queries:
//! ```rust,ignore
//! let mut conn = pool.acquire().await?;
//! set_tenant(&mut conn, schema_name).await?;
//! // ... do queries using &mut *conn ...
//! reset_tenant(&mut conn).await?;
//! ```
//!
//! Every code path that calls `set_tenant()` MUST call `reset_tenant()` before
//! the connection is returned to the pool — including error paths. If reset
//! is impractical on error, use `conn.detach()` to remove the connection from
//! the pool entirely.

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

/// Reset the search path to the default after tenant-scoped work is done.
///
/// Must be called before returning a connection to the pool to prevent
/// cross-tenant data leaks.
pub async fn reset_tenant(conn: &mut PgConnection) -> anyhow::Result<()> {
    sqlx::query("RESET search_path")
        .execute(&mut *conn)
        .await?;
    Ok(())
}
