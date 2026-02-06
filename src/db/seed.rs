//! Database seeder.
//!
//! Seeds default data on first run (empty database). Idempotent — does nothing
//! if data already exists.

use sqlx::PgPool;

use crate::auth::password::hash_password;

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
