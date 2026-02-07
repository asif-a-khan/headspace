//! Safe tenant-scoped database connection wrapper.
//!
//! `TenantGuard` acquires a pooled connection, sets the tenant search_path,
//! and provides query delegation methods that keep `&mut PgConnection` confined
//! to each method body. This avoids the HRTB / `Send` issues that occur when
//! a wrapper struct exposes `&mut PgConnection` to the caller (where Axum
//! handlers require `Send` futures).
//!
//! # Safety guarantees
//!
//! - **`release()`** resets search_path and returns the connection to the pool.
//!   It consumes `self`, preventing further queries after reset.
//! - **`Drop`** detaches (closes) the connection if `release()` was not called,
//!   preventing a poisoned connection from returning to the pool.
//!
//! # Example
//!
//! ```rust,ignore
//! let mut guard = TenantGuard::acquire(db.reader(), &company.schema_name).await?;
//! let leads = guard.fetch_all(
//!     sqlx::query_as::<_, Lead>("SELECT * FROM leads ORDER BY created_at DESC")
//! ).await?;
//! guard.release().await?;
//! ```

use sqlx::postgres::{PgArguments, PgQueryResult, PgRow};
use sqlx::pool::PoolConnection;
use sqlx::{FromRow, PgPool, Postgres};

/// A tenant-scoped database connection that guarantees search_path cleanup.
pub struct TenantGuard {
    conn: Option<PoolConnection<Postgres>>,
    schema_name: String,
}

impl TenantGuard {
    /// Acquire a connection from the pool and set the tenant search_path.
    pub async fn acquire(pool: &PgPool, schema_name: &str) -> anyhow::Result<Self> {
        super::migrate::validate_schema_name(schema_name)?;

        let mut conn = pool.acquire().await?;

        let sql = format!("SET search_path TO {schema_name}, public");
        sqlx::query(&sql).execute(&mut *conn).await?;

        Ok(Self {
            conn: Some(conn),
            schema_name: schema_name.to_owned(),
        })
    }

    /// Execute a `query_as` and return an optional row.
    pub async fn fetch_optional<'q, T>(
        &mut self,
        query: sqlx::query::QueryAs<'q, Postgres, T, PgArguments>,
    ) -> Result<Option<T>, sqlx::Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
    {
        let conn = self.conn.as_mut().expect("TenantGuard: connection already released");
        query.fetch_optional(&mut **conn).await
    }

    /// Execute a `query_as` and return exactly one row.
    pub async fn fetch_one<'q, T>(
        &mut self,
        query: sqlx::query::QueryAs<'q, Postgres, T, PgArguments>,
    ) -> Result<T, sqlx::Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
    {
        let conn = self.conn.as_mut().expect("TenantGuard: connection already released");
        query.fetch_one(&mut **conn).await
    }

    /// Execute a `query_as` and return all rows.
    pub async fn fetch_all<'q, T>(
        &mut self,
        query: sqlx::query::QueryAs<'q, Postgres, T, PgArguments>,
    ) -> Result<Vec<T>, sqlx::Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
    {
        let conn = self.conn.as_mut().expect("TenantGuard: connection already released");
        query.fetch_all(&mut **conn).await
    }

    /// Execute a query (INSERT, UPDATE, DELETE, etc.) and return the result.
    pub async fn execute<'q>(
        &mut self,
        query: sqlx::query::Query<'q, Postgres, PgArguments>,
    ) -> Result<PgQueryResult, sqlx::Error> {
        let conn = self.conn.as_mut().expect("TenantGuard: connection already released");
        query.execute(&mut **conn).await
    }

    /// Reset the search_path and return the connection to the pool.
    ///
    /// If the RESET fails, the connection is detached (closed) instead of
    /// being returned to the pool in a poisoned state.
    pub async fn release(mut self) -> anyhow::Result<()> {
        if let Some(mut conn) = self.conn.take() {
            let result = sqlx::query("RESET search_path")
                .execute(&mut *conn)
                .await;

            if let Err(e) = result {
                tracing::warn!(
                    schema = %self.schema_name,
                    error = %e,
                    "RESET search_path failed — detaching connection"
                );
                let _ = conn.detach();
                return Err(e.into());
            }
            // conn drops here, returning to pool with clean search_path
        }
        Ok(())
    }

    /// Get the tenant schema name.
    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }
}

impl Drop for TenantGuard {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            // Can't do async in Drop — detach so the poisoned connection
            // is closed rather than returned to the pool.
            let _ = conn.detach();
            tracing::warn!(
                schema = %self.schema_name,
                "TenantGuard dropped without release() — connection detached"
            );
        }
    }
}
