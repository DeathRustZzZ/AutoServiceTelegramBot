use async_trait::async_trait;
use chrono::{DateTime, Utc};
use garage_app::{
    AppResult, PartRepository, RepairPartRepository, RepairPartUnitOfWork, RepairRepository,
    StockMovementRepository,
};
use garage_domain::{
    CarId, ClientId, Part, PartId, Repair, RepairId, RepairPart, RepairPartId, StockMovement,
    StockMovementId,
};

use crate::mappers;
use crate::models::{PartRow, RepairPartRow, RepairRow, StockMovementRow};
use crate::repositories::{currency_code, quantity_to_i32, repository_error};
use crate::unit_of_work::transaction::SharedPgTransaction;

pub struct PgRepairPartUnitOfWork {
    tx: SharedPgTransaction,
    repairs: PgRepairTxRepository,
    parts: PgPartTxRepository,
    repair_parts: PgRepairPartTxRepository,
    stock_movements: PgStockMovementTxRepository,
}

impl PgRepairPartUnitOfWork {
    pub async fn begin(pool: &sqlx::PgPool) -> AppResult<Self> {
        let tx = SharedPgTransaction::begin(pool, "begin repair part unit of work").await?;

        Ok(Self {
            repairs: PgRepairTxRepository::new(tx.clone()),
            parts: PgPartTxRepository::new(tx.clone()),
            repair_parts: PgRepairPartTxRepository::new(tx.clone()),
            stock_movements: PgStockMovementTxRepository::new(tx.clone()),
            tx,
        })
    }
}

#[async_trait]
impl RepairPartUnitOfWork for PgRepairPartUnitOfWork {
    type Repairs = PgRepairTxRepository;
    type Parts = PgPartTxRepository;
    type RepairParts = PgRepairPartTxRepository;
    type StockMovements = PgStockMovementTxRepository;

    fn repairs(&self) -> &Self::Repairs {
        &self.repairs
    }

    fn parts(&self) -> &Self::Parts {
        &self.parts
    }

    fn repair_parts(&self) -> &Self::RepairParts {
        &self.repair_parts
    }

    fn stock_movements(&self) -> &Self::StockMovements {
        &self.stock_movements
    }

    async fn commit(&self) -> AppResult<()> {
        self.tx.commit("commit repair part unit of work").await
    }

    async fn rollback(&self) -> AppResult<()> {
        self.tx.rollback("rollback repair part unit of work").await
    }
}

#[derive(Clone)]
pub struct PgRepairTxRepository {
    tx: SharedPgTransaction,
}

impl PgRepairTxRepository {
    fn new(tx: SharedPgTransaction) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl RepairRepository for PgRepairTxRepository {
    async fn get(&self, id: RepairId) -> AppResult<Option<Repair>> {
        let mut guard = self.tx.lock("get repair").await?;
        let tx = guard.transaction()?;

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
        .fetch_optional(&mut **tx)
        .await
        .map_err(|error| repository_error("get repair", error))?;

        row.as_ref().map(mappers::repair::to_domain).transpose()
    }

    async fn save(&self, repair: &Repair) -> AppResult<()> {
        let mut guard = self.tx.lock("save repair").await?;
        let tx = guard.transaction()?;

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
        .execute(&mut **tx)
        .await
        .map_err(|error| repository_error("save repair", error))?;

        Ok(())
    }

    async fn list_by_client(&self, client_id: ClientId) -> AppResult<Vec<Repair>> {
        let mut guard = self.tx.lock("list repairs by client").await?;
        let tx = guard.transaction()?;

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
        .fetch_all(&mut **tx)
        .await
        .map_err(|error| repository_error("list repairs by client", error))?;

        rows.iter().map(mappers::repair::to_domain).collect()
    }

    async fn list_by_car(&self, car_id: CarId) -> AppResult<Vec<Repair>> {
        let mut guard = self.tx.lock("list repairs by car").await?;
        let tx = guard.transaction()?;

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
        .fetch_all(&mut **tx)
        .await
        .map_err(|error| repository_error("list repairs by car", error))?;

        rows.iter().map(mappers::repair::to_domain).collect()
    }

    async fn list_active(&self) -> AppResult<Vec<Repair>> {
        let mut guard = self.tx.lock("list active repairs").await?;
        let tx = guard.transaction()?;

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
        .fetch_all(&mut **tx)
        .await
        .map_err(|error| repository_error("list active repairs", error))?;

        rows.iter().map(mappers::repair::to_domain).collect()
    }

    async fn list_completed_between(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> AppResult<Vec<Repair>> {
        let mut guard = self.tx.lock("list completed repairs between").await?;
        let tx = guard.transaction()?;

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
        .fetch_all(&mut **tx)
        .await
        .map_err(|error| repository_error("list completed repairs between", error))?;

        rows.iter().map(mappers::repair::to_domain).collect()
    }
}

#[derive(Clone)]
pub struct PgPartTxRepository {
    tx: SharedPgTransaction,
}

impl PgPartTxRepository {
    fn new(tx: SharedPgTransaction) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl PartRepository for PgPartTxRepository {
    async fn get(&self, id: PartId) -> AppResult<Option<Part>> {
        let mut guard = self.tx.lock("get part").await?;
        let tx = guard.transaction()?;

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
        .fetch_optional(&mut **tx)
        .await
        .map_err(|error| repository_error("get part", error))?;

        row.as_ref().map(mappers::part::to_domain).transpose()
    }

    async fn save(&self, part: &Part) -> AppResult<()> {
        let quantity = quantity_to_i32("save part", "quantity", part.quantity())?;
        let min_quantity = quantity_to_i32("save part", "min_quantity", part.min_quantity())?;
        let unit_price = part.unit_price();

        let mut guard = self.tx.lock("save part").await?;
        let tx = guard.transaction()?;

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
        .execute(&mut **tx)
        .await
        .map_err(|error| repository_error("save part", error))?;

        Ok(())
    }

    async fn list_low_stock(&self) -> AppResult<Vec<Part>> {
        let mut guard = self.tx.lock("list low stock parts").await?;
        let tx = guard.transaction()?;

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
            WHERE status = 'active'
              AND quantity <= min_quantity
            ORDER BY quantity ASC, name ASC, id ASC
            "#,
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(|error| repository_error("list low stock parts", error))?;

        rows.iter().map(mappers::part::to_domain).collect()
    }

    async fn search(&self, query: &str) -> AppResult<Vec<Part>> {
        let query = query.trim();
        let pattern = format!("%{query}%");

        let mut guard = self.tx.lock("search parts").await?;
        let tx = guard.transaction()?;

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
        .fetch_all(&mut **tx)
        .await
        .map_err(|error| repository_error("search parts", error))?;

        rows.iter().map(mappers::part::to_domain).collect()
    }
}

#[derive(Clone)]
pub struct PgRepairPartTxRepository {
    tx: SharedPgTransaction,
}

impl PgRepairPartTxRepository {
    fn new(tx: SharedPgTransaction) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl RepairPartRepository for PgRepairPartTxRepository {
    async fn get(&self, id: RepairPartId) -> AppResult<Option<RepairPart>> {
        let mut guard = self.tx.lock("get repair part").await?;
        let tx = guard.transaction()?;

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
        .fetch_optional(&mut **tx)
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

        let mut guard = self.tx.lock("save repair part").await?;
        let tx = guard.transaction()?;

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
        .execute(&mut **tx)
        .await
        .map_err(|error| repository_error("save repair part", error))?;

        Ok(())
    }

    async fn list_by_repair(&self, repair_id: RepairId) -> AppResult<Vec<RepairPart>> {
        let mut guard = self.tx.lock("list repair parts by repair").await?;
        let tx = guard.transaction()?;

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
        .fetch_all(&mut **tx)
        .await
        .map_err(|error| repository_error("list repair parts by repair", error))?;

        rows.iter().map(mappers::repair_part::to_domain).collect()
    }
}

#[derive(Clone)]
pub struct PgStockMovementTxRepository {
    tx: SharedPgTransaction,
}

impl PgStockMovementTxRepository {
    fn new(tx: SharedPgTransaction) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl StockMovementRepository for PgStockMovementTxRepository {
    async fn get(&self, id: StockMovementId) -> AppResult<Option<StockMovement>> {
        let mut guard = self.tx.lock("get stock movement").await?;
        let tx = guard.transaction()?;

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
        .fetch_optional(&mut **tx)
        .await
        .map_err(|error| repository_error("get stock movement", error))?;

        row.as_ref()
            .map(mappers::stock_movement::to_domain)
            .transpose()
    }

    async fn save(&self, movement: &StockMovement) -> AppResult<()> {
        let quantity = quantity_to_i32("save stock movement", "quantity", movement.quantity())?;

        let mut guard = self.tx.lock("save stock movement").await?;
        let tx = guard.transaction()?;

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
        .execute(&mut **tx)
        .await
        .map_err(|error| repository_error("save stock movement", error))?;

        Ok(())
    }

    async fn list_by_part(&self, part_id: PartId) -> AppResult<Vec<StockMovement>> {
        let mut guard = self.tx.lock("list stock movements by part").await?;
        let tx = guard.transaction()?;

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
        .fetch_all(&mut **tx)
        .await
        .map_err(|error| repository_error("list stock movements by part", error))?;

        rows.iter()
            .map(mappers::stock_movement::to_domain)
            .collect()
    }
}
