//! JSON API handlers.
//!
//! These handlers return `axum::Json` responses consumed by Vue.js components.
//! They share query logic with HTML handlers in `handlers/` via the mirror
//! routes pattern (see `docs/architecture/DESIGN_PATTERN.md`).

pub mod super_admin;
