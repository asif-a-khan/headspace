use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Quote {
    pub id: i64,
    pub subject: String,
    pub description: Option<String>,
    pub billing_address: Option<serde_json::Value>,
    pub shipping_address: Option<serde_json::Value>,
    pub discount_percent: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub tax_amount: Option<Decimal>,
    pub adjustment_amount: Option<Decimal>,
    pub sub_total: Option<Decimal>,
    pub grand_total: Option<Decimal>,
    pub expired_at: Option<DateTime<Utc>>,
    pub person_id: Option<i64>,
    pub user_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Quote with joined names for list display.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct QuoteRow {
    pub id: i64,
    pub subject: String,
    pub grand_total: Option<Decimal>,
    pub expired_at: Option<DateTime<Utc>>,
    pub person_id: Option<i64>,
    pub user_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub person_name: Option<String>,
    pub user_name: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct QuoteItem {
    pub id: i64,
    pub sku: Option<String>,
    pub name: Option<String>,
    pub quantity: i32,
    pub price: Decimal,
    pub discount_percent: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub tax_percent: Option<Decimal>,
    pub tax_amount: Option<Decimal>,
    pub total: Decimal,
    pub product_id: Option<i64>,
    pub quote_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
