use async_trait::async_trait;
use garage_app::{AppResult, PartRepository};
use garage_domain::{Part, PartId};

use crate::mappers;
use crate::models::PartRow;
use crate::repositories::{currency_code, quantity_to_i32, repository_error};

#[derive(Clone)]
pub struct PgPartRepository {
    pool: sqlx::PgPool,
}

impl PgPartRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PartRepository for PgPartRepository {
    async fn get(&self, id: PartId) -> AppResult<Option<Part>> {
        let row = sqlx::query_as::<_, PartRow>(
            r#"
            SELECT
                id,
                name,
                sku,
                quantity,
                min_quantity,
                unit_price,
                currency,
                notes,
                status,
                created_at,
                updated_at
            FROM parts
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| repository_error("get part", error))?;

        row.as_ref().map(mappers::part::to_domain).transpose()
    }

    async fn save(&self, part: &Part) -> AppResult<()> {
        let quantity = quantity_to_i32("save part", "quantity", part.quantity())?;
        let min_quantity = quantity_to_i32("save part", "min_quantity", part.min_quantity())?;
        let unit_price = part.unit_price();

        sqlx::query(
            r#"
            INSERT INTO parts (
                id,
                name,
                sku,
                quantity,
                min_quantity,
                unit_price,
                currency,
                notes,
                status,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                sku = EXCLUDED.sku,
                quantity = EXCLUDED.quantity,
                min_quantity = EXCLUDED.min_quantity,
                unit_price = EXCLUDED.unit_price,
                currency = EXCLUDED.currency,
                notes = EXCLUDED.notes,
                status = EXCLUDED.status,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(part.id().as_uuid())
        .bind(part.name().as_str())
        .bind(part.sku().map(|sku| sku.as_str()))
        .bind(quantity)
        .bind(min_quantity)
        .bind(unit_price.amount_minor())
        .bind(currency_code(unit_price.currency()))
        .bind(part.notes().map(|notes| notes.as_str()))
        .bind(part.status().to_string())
        .bind(part.created_at())
        .bind(part.updated_at())
        .execute(&self.pool)
        .await
        .map_err(|error| repository_error("save part", error))?;

        Ok(())
    }

    async fn list_low_stock(&self) -> AppResult<Vec<Part>> {
        let rows = sqlx::query_as::<_, PartRow>(
            r#"
            SELECT
                id,
                name,
                sku,
                quantity,
                min_quantity,
                unit_price,
                currency,
                notes,
                status,
                created_at,
                updated_at
            FROM parts
            WHERE quantity <= min_quantity
            ORDER BY quantity ASC, name ASC, id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|error| repository_error("list low stock parts", error))?;

        rows.iter().map(mappers::part::to_domain).collect()
    }

    async fn search(&self, query: &str) -> AppResult<Vec<Part>> {
        let query = query.trim();
        let pattern = format!("%{query}%");

        let rows = sqlx::query_as::<_, PartRow>(
            r#"
            SELECT
                id,
                name,
                sku,
                quantity,
                min_quantity,
                unit_price,
                currency,
                notes,
                status,
                created_at,
                updated_at
            FROM parts
            WHERE $1 = '' OR name ILIKE $2 OR sku ILIKE $2
            ORDER BY name ASC, id ASC
            "#,
        )
        .bind(query)
        .bind(pattern)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| repository_error("search parts", error))?;

        rows.iter().map(mappers::part::to_domain).collect()
    }
}
