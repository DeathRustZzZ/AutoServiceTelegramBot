use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PartSupplyRow {
    pub id: Uuid,
    pub part_id: Uuid,
    pub quantity: i32,
    pub expected_at: DateTime<Utc>,
    pub status: String,
    pub supplier: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
