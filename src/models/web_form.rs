use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct WebForm {
    pub id: i64,
    pub form_id: String,
    pub title: String,
    pub description: Option<String>,
    pub submit_button_label: String,
    pub submit_success_action: String,
    pub submit_success_content: String,
    pub create_lead: bool,
    pub background_color: Option<String>,
    pub form_background_color: Option<String>,
    pub form_title_color: Option<String>,
    pub form_submit_button_color: Option<String>,
    pub attribute_label_color: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct WebFormAttribute {
    pub id: i64,
    pub name: Option<String>,
    pub placeholder: Option<String>,
    pub is_required: bool,
    pub sort_order: Option<i32>,
    pub attribute_id: i64,
    pub web_form_id: i64,
}

/// Joined view for listing web form attributes with their attribute details.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct WebFormAttributeRow {
    pub id: i64,
    pub name: Option<String>,
    pub placeholder: Option<String>,
    pub is_required: bool,
    pub sort_order: Option<i32>,
    pub attribute_id: i64,
    pub web_form_id: i64,
    pub attribute_name: Option<String>,
    pub attribute_code: Option<String>,
    pub attribute_type: Option<String>,
}
