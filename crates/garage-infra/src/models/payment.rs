use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Строка таблицы `payments`.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PaymentRow {
    pub id: Uuid,
    pub repair_id: Uuid,
    pub amount: i64,
    pub currency: String,
    pub method: String,
    pub comment: Option<String>,
    pub paid_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
