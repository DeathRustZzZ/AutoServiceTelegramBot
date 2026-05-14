use async_trait::async_trait;
use garage_app::{AppResult, StockMovementRepository};
use garage_domain::{PartId, StockMovement, StockMovementId};

use crate::mappers;
use crate::models::StockMovementRow;
use crate::repositories::{quantity_to_i32, repository_error};

#[derive(Clone)]
pub struct PgStockMovementRepository {
    pool: sqlx::PgPool,
}

impl PgStockMovementRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StockMovementRepository for PgStockMovementRepository {
    async fn get(&self, id: StockMovementId) -> AppResult<Option<StockMovement>> {
        let row = sqlx::query_as::<_, StockMovementRow>(
            r#"
            SELECT
                id,
                part_id,
                movement_type,
                quantity,
                reason,
                comment,
                occurred_at,
                created_at
            FROM stock_movements
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| repository_error("get stock movement", error))?;

        row.as_ref()
            .map(mappers::stock_movement::to_domain)
            .transpose()
    }

    async fn save(&self, movement: &StockMovement) -> AppResult<()> {
        let quantity = quantity_to_i32("save stock movement", "quantity", movement.quantity())?;

        sqlx::query(
            r#"
            INSERT INTO stock_movements (
                id,
                part_id,
                movement_type,
                quantity,
                reason,
                comment,
                occurred_at,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO UPDATE SET
                part_id = EXCLUDED.part_id,
                movement_type = EXCLUDED.movement_type,
                quantity = EXCLUDED.quantity,
                reason = EXCLUDED.reason,
                comment = EXCLUDED.comment,
                occurred_at = EXCLUDED.occurred_at
            "#,
        )
        .bind(movement.id().as_uuid())
        .bind(movement.part_id().as_uuid())
        .bind(movement.movement_type().to_string())
        .bind(quantity)
        .bind(movement.reason().to_string())
        .bind(movement.comment().map(|comment| comment.as_str()))
        .bind(movement.occurred_at())
        .bind(movement.created_at())
        .execute(&self.pool)
        .await
        .map_err(|error| repository_error("save stock movement", error))?;

        Ok(())
    }

    async fn list_by_part(&self, part_id: PartId) -> AppResult<Vec<StockMovement>> {
        let rows = sqlx::query_as::<_, StockMovementRow>(
            r#"
            SELECT
                id,
                part_id,
                movement_type,
                quantity,
                reason,
                comment,
                occurred_at,
                created_at
            FROM stock_movements
            WHERE part_id = $1
            ORDER BY occurred_at DESC, id ASC
            "#,
        )
        .bind(part_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|error| repository_error("list stock movements by part", error))?;

        rows.iter()
            .map(mappers::stock_movement::to_domain)
            .collect()
    }
}
