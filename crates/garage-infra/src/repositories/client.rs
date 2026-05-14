use async_trait::async_trait;
use garage_app::{AppResult, ClientRepository};
use garage_domain::{Client, ClientId};

use crate::mappers;
use crate::models::ClientRow;
use crate::repositories::repository_error;

#[derive(Clone)]
pub struct PgClientRepository {
    pool: sqlx::PgPool,
}

impl PgClientRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ClientRepository for PgClientRepository {
    async fn get(&self, id: ClientId) -> AppResult<Option<Client>> {
        let row = sqlx::query_as::<_, ClientRow>(
            r#"
            SELECT id, name, phone, notes, status, created_at, updated_at
            FROM clients
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| repository_error("get client", error))?;

        row.as_ref().map(mappers::client::to_domain).transpose()
    }

    async fn list(&self, limit: u32, offset: u32) -> AppResult<Vec<Client>> {
        let rows = sqlx::query_as::<_, ClientRow>(
            r#"
            SELECT id, name, phone, notes, status, created_at, updated_at
            FROM clients
            WHERE status = 'active'
            ORDER BY created_at DESC, id ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(i64::from(limit))
        .bind(i64::from(offset))
        .fetch_all(&self.pool)
        .await
        .map_err(|error| repository_error("list clients", error))?;

        rows.iter().map(mappers::client::to_domain).collect()
    }

    async fn search(&self, query: &str, limit: u32, offset: u32) -> AppResult<Vec<Client>> {
        let pattern = format!("%{}%", query.trim());
        let rows = sqlx::query_as::<_, ClientRow>(
            r#"
            SELECT id, name, phone, notes, status, created_at, updated_at
            FROM clients
            WHERE status = 'active'
              AND (name ILIKE $1 OR phone ILIKE $1)
            ORDER BY created_at DESC, id ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(pattern)
        .bind(i64::from(limit))
        .bind(i64::from(offset))
        .fetch_all(&self.pool)
        .await
        .map_err(|error| repository_error("search clients", error))?;

        rows.iter().map(mappers::client::to_domain).collect()
    }

    async fn save(&self, client: &Client) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO clients (
                id, name, phone, notes, status, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                phone = EXCLUDED.phone,
                notes = EXCLUDED.notes,
                status = EXCLUDED.status,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(client.id().as_uuid())
        .bind(client.name().as_str())
        .bind(client.phone().as_str())
        .bind(client.notes().map(|notes| notes.as_str()))
        .bind(client.status().to_string())
        .bind(client.created_at())
        .bind(client.updated_at())
        .execute(&self.pool)
        .await
        .map_err(|error| repository_error("save client", error))?;

        Ok(())
    }
}
