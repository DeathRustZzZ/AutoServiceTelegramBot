use async_trait::async_trait;
use garage_app::{AppResult, RepairPartRepository};
use garage_domain::{RepairId, RepairPart, RepairPartId};

use crate::mappers;
use crate::models::RepairPartRow;
use crate::repositories::{currency_code, quantity_to_i32, repository_error};

#[derive(Clone)]
pub struct PgRepairPartRepository {
    pool: sqlx::PgPool,
}

impl PgRepairPartRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RepairPartRepository for PgRepairPartRepository {
    async fn get(&self, id: RepairPartId) -> AppResult<Option<RepairPart>> {
        let row = sqlx::query_as::<_, RepairPartRow>(
            r#"
            SELECT
                id,
                repair_id,
                part_id,
                quantity,
                unit_cost,
                unit_price,
                currency,
                created_at
            FROM repair_parts
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| repository_error("get repair part", error))?;

        row.as_ref()
            .map(mappers::repair_part::to_domain)
            .transpose()
    }

    async fn save(&self, repair_part: &RepairPart) -> AppResult<()> {
        let quantity = quantity_to_i32("save repair part", "quantity", repair_part.quantity())?;
        let unit_cost = repair_part.unit_cost();
        let unit_price = repair_part.unit_price();

        sqlx::query(
            r#"
            INSERT INTO repair_parts (
                id,
                repair_id,
                part_id,
                quantity,
                unit_cost,
                unit_price,
                currency,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO UPDATE SET
                repair_id = EXCLUDED.repair_id,
                part_id = EXCLUDED.part_id,
                quantity = EXCLUDED.quantity,
                unit_cost = EXCLUDED.unit_cost,
                unit_price = EXCLUDED.unit_price,
                currency = EXCLUDED.currency
            "#,
        )
        .bind(repair_part.id().as_uuid())
        .bind(repair_part.repair_id().as_uuid())
        .bind(repair_part.part_id().as_uuid())
        .bind(quantity)
        .bind(unit_cost.amount_minor())
        .bind(unit_price.amount_minor())
        .bind(currency_code(unit_cost.currency()))
        .bind(repair_part.created_at())
        .execute(&self.pool)
        .await
        .map_err(|error| repository_error("save repair part", error))?;

        Ok(())
    }

    async fn list_by_repair(&self, repair_id: RepairId) -> AppResult<Vec<RepairPart>> {
        let rows = sqlx::query_as::<_, RepairPartRow>(
            r#"
            SELECT
                id,
                repair_id,
                part_id,
                quantity,
                unit_cost,
                unit_price,
                currency,
                created_at
            FROM repair_parts
            WHERE repair_id = $1
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .bind(repair_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|error| repository_error("list repair parts by repair", error))?;

        rows.iter().map(mappers::repair_part::to_domain).collect()
    }
}
