use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Строка таблицы `clients`.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ClientRow {
    pub id: Uuid,
    pub name: String,
    pub phone: String,
    pub notes: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
