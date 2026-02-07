//! Domain models.
//!
//! Each model maps to a database table using sqlx::FromRow.

pub mod activity;
pub mod attribute;
pub mod company;
pub mod email_template;
pub mod group;
pub mod lead;
pub mod organization;
pub mod person;
pub mod pipeline;
pub mod product;
pub mod quote;
pub mod super_admin;
pub mod super_role;
pub mod tag;
pub mod tenant_admin;
pub mod warehouse;
pub mod web_form;
