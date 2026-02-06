use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct SuperAdmin {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    #[serde(skip)]
    pub password_hash: String,
    pub image: Option<String>,
    pub status: bool,
    pub role_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Joined from super_roles (optional, populated by JOIN queries)
    #[sqlx(default)]
    pub permission_type: Option<String>,
    #[sqlx(default)]
    #[serde(skip)]
    pub role_permissions: Option<serde_json::Value>,
}

impl SuperAdmin {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    /// Check if this admin has a specific permission.
    /// Returns true if permission_type is "all" or if the permission key
    /// is present in the role's permissions array.
    pub fn has_permission(&self, key: &str) -> bool {
        match self.permission_type.as_deref() {
            Some("all") => true,
            Some("custom") => self
                .role_permissions
                .as_ref()
                .and_then(|v| v.as_array())
                .map(|perms| perms.iter().any(|p| p.as_str() == Some(key)))
                .unwrap_or(false),
            _ => false,
        }
    }
}
