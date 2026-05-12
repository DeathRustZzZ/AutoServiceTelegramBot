use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RepairRow {
    pub id: Uuid,
    pub client_id: Uuid,
    pub car_id: Uuid,
    pub booking_id: Option<Uuid>,
    pub status: String,
    pub description: String,
    pub labor_price: i64,
    pub parts_price: i64,
    pub parts_cost: i64,
    pub paid_amount: i64,
    pub currency: String,
    pub notes: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
