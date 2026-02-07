//! Permission checking (bouncer) and payload validation.
//!
//! Provides `has_permission()` for role-based access control, `bouncer()`
//! for guarding API handlers, and `validate_payload()` for input validation.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use validator::Validate;

use crate::models::tenant_admin::TenantUser;

/// Check if a role has a specific permission.
///
/// Returns true if `permission_type` is "all".
/// For "custom", checks the permissions JSON array for the key.
pub fn has_permission(
    permission_type: &str,
    permissions: &serde_json::Value,
    key: &str,
) -> bool {
    match permission_type {
        "all" => true,
        "custom" => permissions
            .as_array()
            .map(|perms| perms.iter().any(|p| p.as_str() == Some(key)))
            .unwrap_or(false),
        _ => false,
    }
}

/// Guard an API endpoint — returns `Err(Response)` with a 403 JSON body
/// if the user lacks the required permission.
///
/// Usage in handlers:
/// ```ignore
/// bouncer(&user, "leads.create")?;
/// ```
pub fn bouncer(user: &TenantUser, permission_key: &str) -> Result<(), Response> {
    if user.has_permission(permission_key) {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "Unauthorized",
                "message": format!("You do not have the '{}' permission.", permission_key),
            })),
        )
            .into_response())
    }
}

/// Validate a payload using the `validator` crate.
///
/// Returns `Err(Response)` with a 422 JSON body containing field-level errors.
///
/// Usage in handlers:
/// ```ignore
/// validate_payload(&payload)?;
/// ```
pub fn validate_payload(payload: &impl Validate) -> Result<(), Response> {
    match payload.validate() {
        Ok(()) => Ok(()),
        Err(errors) => {
            let mut field_errors = serde_json::Map::new();
            for (field, errs) in errors.field_errors() {
                let messages: Vec<String> = errs
                    .iter()
                    .map(|e| {
                        e.message
                            .as_ref()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| format!("Invalid value for {field}"))
                    })
                    .collect();
                field_errors.insert(field.to_string(), serde_json::json!(messages));
            }
            Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({
                    "error": "Validation failed",
                    "errors": field_errors,
                })),
            )
                .into_response())
        }
    }
}
