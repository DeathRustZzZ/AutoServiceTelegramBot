use async_trait::async_trait;
use garage_app::{AppResult, PartSupplyRepository};
use garage_domain::{PartId, PartSupply, PartSupplyId};

use crate::mappers;
use crate::models::PartSupplyRow;
use crate::repositories::{quantity_to_i32, repository_error};

#[derive(Clone)]
pub struct PgPartSupplyRepository {
    pool: sqlx::PgPool,
}

impl PgPartSupplyRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PartSupplyRepository for PgPartSupplyRepository {
    async fn get(&self, id: PartSupplyId) -> AppResult<Option<PartSupply>> {
        let row = sqlx::query_as::<_, PartSupplyRow>(
            r#"
            SELECT
                id,
                part_id,
                quantity,
                expected_at,
                status,
                supplier,
                notes,
                created_at,
                updated_at
            FROM part_supplies
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| repository_error("get part supply", error))?;

        row.as_ref()
            .map(mappers::part_supply::to_domain)
            .transpose()
    }

    async fn save(&self, supply: &PartSupply) -> AppResult<()> {
        let quantity = quantity_to_i32("save part supply", "quantity", supply.quantity())?;

        sqlx::query(
            r#"
            INSERT INTO part_supplies (
                id,
                part_id,
                quantity,
                expected_at,
                status,
                supplier,
                notes,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO UPDATE SET
                part_id = EXCLUDED.part_id,
                quantity = EXCLUDED.quantity,
                expected_at = EXCLUDED.expected_at,
                status = EXCLUDED.status,
                supplier = EXCLUDED.supplier,
                notes = EXCLUDED.notes,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(supply.id().as_uuid())
        .bind(supply.part_id().as_uuid())
        .bind(quantity)
        .bind(supply.expected_at())
        .bind(supply.status().to_string())
        .bind(supply.supplier().map(|supplier| supplier.as_str()))
        .bind(supply.notes().map(|notes| notes.as_str()))
        .bind(supply.created_at())
        .bind(supply.updated_at())
        .execute(&self.pool)
        .await
        .map_err(|error| repository_error("save part supply", error))?;

        Ok(())
    }

    async fn list_by_part(&self, part_id: PartId) -> AppResult<Vec<PartSupply>> {
        let rows = sqlx::query_as::<_, PartSupplyRow>(
            r#"
            SELECT
                id,
                part_id,
                quantity,
                expected_at,
                status,
                supplier,
                notes,
                created_at,
                updated_at
            FROM part_supplies
            WHERE part_id = $1
            ORDER BY expected_at ASC, id ASC
            "#,
        )
        .bind(part_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|error| repository_error("list part supplies by part", error))?;

        rows.iter().map(mappers::part_supply::to_domain).collect()
    }
}
