use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BookingRow {
    pub id: Uuid,
    pub client_id: Uuid,
    pub car_id: Uuid,
    pub scheduled_at: DateTime<Utc>,
    pub status: String,
    pub reason: String,
    pub notes: Option<String>,
    pub closed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
