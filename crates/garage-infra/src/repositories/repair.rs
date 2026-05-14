use async_trait::async_trait;
use chrono::{DateTime, Utc};
use garage_app::{AppResult, RepairRepository};
use garage_domain::{CarId, ClientId, Repair, RepairId};

use crate::mappers;
use crate::models::RepairRow;
use crate::repositories::{currency_code, repository_error};

#[derive(Clone)]
pub struct PgRepairRepository {
    pool: sqlx::PgPool,
}

impl PgRepairRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RepairRepository for PgRepairRepository {
    async fn get(&self, id: RepairId) -> AppResult<Option<Repair>> {
        let row = sqlx::query_as::<_, RepairRow>(
            r#"
            SELECT
                id,
                client_id,
                car_id,
                booking_id,
                status,
                description,
                labor_price,
                parts_price,
                parts_cost,
                paid_amount,
                currency,
                notes,
                started_at,
                completed_at,
                created_at,
                updated_at
            FROM repairs
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| repository_error("get repair", error))?;

        row.as_ref().map(mappers::repair::to_domain).transpose()
    }

    async fn save(&self, repair: &Repair) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO repairs (
                id,
                client_id,
                car_id,
                booking_id,
                status,
                description,
                labor_price,
                parts_price,
                parts_cost,
                paid_amount,
                currency,
                notes,
                started_at,
                completed_at,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            ON CONFLICT (id) DO UPDATE SET
                client_id = EXCLUDED.client_id,
                car_id = EXCLUDED.car_id,
                booking_id = EXCLUDED.booking_id,
                status = EXCLUDED.status,
                description = EXCLUDED.description,
                labor_price = EXCLUDED.labor_price,
                parts_price = EXCLUDED.parts_price,
                parts_cost = EXCLUDED.parts_cost,
                paid_amount = EXCLUDED.paid_amount,
                currency = EXCLUDED.currency,
                notes = EXCLUDED.notes,
                started_at = EXCLUDED.started_at,
                completed_at = EXCLUDED.completed_at,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(repair.id().as_uuid())
        .bind(repair.client_id().as_uuid())
        .bind(repair.car_id().as_uuid())
        .bind(repair.booking_id().map(|id| id.as_uuid()))
        .bind(repair.status().to_string())
        .bind(repair.description().as_str())
        .bind(repair.labor_price().amount_minor())
        .bind(repair.parts_price().amount_minor())
        .bind(repair.parts_cost().amount_minor())
        .bind(repair.paid_amount().amount_minor())
        .bind(currency_code(repair.currency()))
        .bind(repair.notes().map(|notes| notes.as_str()))
        .bind(repair.started_at())
        .bind(repair.completed_at())
        .bind(repair.created_at())
        .bind(repair.updated_at())
        .execute(&self.pool)
        .await
        .map_err(|error| repository_error("save repair", error))?;

        Ok(())
    }

    async fn list_by_client(&self, client_id: ClientId) -> AppResult<Vec<Repair>> {
        let rows = sqlx::query_as::<_, RepairRow>(
            r#"
            SELECT
                id,
                client_id,
                car_id,
                booking_id,
                status,
                description,
                labor_price,
                parts_price,
                parts_cost,
                paid_amount,
                currency,
                notes,
                started_at,
                completed_at,
                created_at,
                updated_at
            FROM repairs
            WHERE client_id = $1
            ORDER BY started_at DESC, id ASC
            "#,
        )
        .bind(client_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|error| repository_error("list repairs by client", error))?;

        rows.iter().map(mappers::repair::to_domain).collect()
    }

    async fn list_by_car(&self, car_id: CarId) -> AppResult<Vec<Repair>> {
        let rows = sqlx::query_as::<_, RepairRow>(
            r#"
            SELECT
                id,
                client_id,
                car_id,
                booking_id,
                status,
                description,
                labor_price,
                parts_price,
                parts_cost,
                paid_amount,
                currency,
                notes,
                started_at,
                completed_at,
                created_at,
                updated_at
            FROM repairs
            WHERE car_id = $1
            ORDER BY started_at DESC, id ASC
            "#,
        )
        .bind(car_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|error| repository_error("list repairs by car", error))?;

        rows.iter().map(mappers::repair::to_domain).collect()
    }

    async fn list_active(&self) -> AppResult<Vec<Repair>> {
        let rows = sqlx::query_as::<_, RepairRow>(
            r#"
            SELECT
                id,
                client_id,
                car_id,
                booking_id,
                status,
                description,
                labor_price,
                parts_price,
                parts_cost,
                paid_amount,
                currency,
                notes,
                started_at,
                completed_at,
                created_at,
                updated_at
            FROM repairs
            WHERE status = 'in_progress'
            ORDER BY updated_at DESC, id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|error| repository_error("list active repairs", error))?;

        rows.iter().map(mappers::repair::to_domain).collect()
    }

    async fn list_completed_between(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> AppResult<Vec<Repair>> {
        let rows = sqlx::query_as::<_, RepairRow>(
            r#"
            SELECT
                id,
                client_id,
                car_id,
                booking_id,
                status,
                description,
                labor_price,
                parts_price,
                parts_cost,
                paid_amount,
                currency,
                notes,
                started_at,
                completed_at,
                created_at,
                updated_at
            FROM repairs
            WHERE status = 'completed'
              AND completed_at >= $1
              AND completed_at < $2
            ORDER BY completed_at ASC, id ASC
            "#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| repository_error("list completed repairs between", error))?;

        rows.iter().map(mappers::repair::to_domain).collect()
    }
}
