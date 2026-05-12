use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CarRow {
    pub id: Uuid,
    pub client_id: Uuid,
    pub make: String,
    pub model: String,
    pub year: Option<i16>,
    pub license_plate: Option<String>,
    pub vin: Option<String>,
    pub notes: Option<String>,
    pub registration_document_photo_ref: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
