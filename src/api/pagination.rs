use serde::{Deserialize, Serialize};

/// Reusable COUNT(*) row for pagination queries.
#[derive(sqlx::FromRow)]
pub struct CountRow {
    pub count: Option<i64>,
}

/// Query parameters for paginated list endpoints.
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub sort_field: Option<String>,
    pub sort_dir: Option<String>,
}

impl PaginationParams {
    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn per_page(&self) -> i64 {
        self.per_page.unwrap_or(15).clamp(1, 100)
    }

    pub fn offset(&self) -> i64 {
        (self.page() - 1) * self.per_page()
    }

    /// Returns a sanitized ORDER BY clause.
    /// `allowed` maps frontend field names → actual SQL column expressions.
    pub fn order_by(&self, allowed: &[(&str, &str)], default: &str) -> String {
        let dir = match self.sort_dir.as_deref() {
            Some("asc" | "ASC") => "ASC",
            _ => "DESC",
        };

        if let Some(ref field) = self.sort_field {
            for (name, col) in allowed {
                if *name == field.as_str() {
                    return format!("{col} {dir}");
                }
            }
        }

        format!("{default} {dir}")
    }
}

/// Pagination metadata included in API responses.
#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub last_page: i64,
}

impl PaginationMeta {
    pub fn new(total: i64, page: i64, per_page: i64) -> Self {
        let last_page = if per_page > 0 {
            (total + per_page - 1) / per_page
        } else {
            1
        };
        Self {
            total,
            page,
            per_page,
            last_page,
        }
    }
}
