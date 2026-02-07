//! Domain models.
//!
//! Each model maps to a database table using sqlx::FromRow.

pub mod company;
pub mod group;
pub mod super_admin;
pub mod super_role;
pub mod tenant_admin;
