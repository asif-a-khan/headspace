use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Attribute {
    pub id: i64,
    pub code: String,
    pub name: String,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub attr_type: String,
    pub entity_type: String,
    pub sort_order: i32,
    pub validation: Option<String>,
    pub is_required: bool,
    pub is_unique: bool,
    pub quick_add: bool,
    pub is_user_defined: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct AttributeOption {
    pub id: i64,
    pub name: String,
    pub sort_order: i32,
    pub attribute_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct AttributeValue {
    pub id: i64,
    pub entity_type: String,
    pub entity_id: i64,
    pub attribute_id: i64,
    pub text_value: Option<String>,
    pub boolean_value: Option<bool>,
    pub integer_value: Option<i64>,
    pub float_value: Option<f64>,
    pub date_value: Option<NaiveDate>,
    pub datetime_value: Option<DateTime<Utc>>,
    pub json_value: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
