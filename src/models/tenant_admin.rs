use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct TenantUser {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    #[serde(skip)]
    pub password_hash: String,
    pub image: Option<String>,
    pub status: bool,
    pub role_id: i64,
    pub view_permission: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Joined from roles (optional, populated by JOIN queries)
    #[sqlx(default)]
    pub permission_type: Option<String>,
    #[sqlx(default)]
    #[serde(skip)]
    pub role_permissions: Option<serde_json::Value>,
}

impl TenantUser {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

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

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct TenantRole {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub permission_type: String,
    pub permissions: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
