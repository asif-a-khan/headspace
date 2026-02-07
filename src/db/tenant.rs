//! Low-level tenant context management.
//!
//! **For handler and middleware code, use [`super::guard::TenantGuard`] instead.**
//! TenantGuard wraps these primitives with Drop safety — if a `?` or early
//! return skips cleanup, the connection is detached rather than returned to the
//! pool with a tenant search_path still active.
//!
//! These functions remain available for migration and seed code where
//! `TenantGuard`'s query delegation pattern doesn't apply (e.g. `Migrator::run()`
//! requires the `Acquire` trait on the connection directly).

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
