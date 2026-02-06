//! Permission checking.
//!
//! Provides `has_permission()` for role-based access control.

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
