use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PartRow {
    pub id: Uuid,
    pub name: String,
    pub sku: Option<String>,
    pub quantity: i32,
    pub min_quantity: i32,
    pub unit_price: i64,
    pub currency: String,
    pub notes: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
