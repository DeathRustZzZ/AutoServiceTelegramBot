use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Строка таблицы `repair_parts`.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RepairPartRow {
    pub id: Uuid,
    pub repair_id: Uuid,
    pub part_id: Uuid,
    pub quantity: i32,
    pub unit_cost: i64,
    pub unit_price: i64,
    pub currency: String,
    pub created_at: DateTime<Utc>,
}
