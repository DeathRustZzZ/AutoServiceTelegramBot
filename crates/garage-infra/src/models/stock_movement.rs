use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Строка таблицы `stock_movements`.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StockMovementRow {
    pub id: Uuid,
    pub part_id: Uuid,
    pub movement_type: String,
    pub quantity: i32,
    pub reason: String,
    pub comment: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
