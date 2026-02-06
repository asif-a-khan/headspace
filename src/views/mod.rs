//! Askama template structs.
//!
//! Each template struct maps to an HTML file in the `templates/` directory.
//! Derive `askama::Template` for rendering and `askama_web::WebTemplate`
//! for axum `IntoResponse` integration.

pub mod super_admin;
pub mod vite;
