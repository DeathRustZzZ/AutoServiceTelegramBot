use crate::mappers;
use crate::models::{PaymentRow, RepairRow};
use crate::repositories::{currency_code, repository_error};
use crate::unit_of_work::transaction::SharedPgTransaction;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use garage_app::{AppResult, PaymentRepository, PaymentUnitOfWork, RepairRepository};
use garage_domain::{CarId, ClientId, Payment, PaymentId, Repair, RepairId};

pub struct PgPaymentUnitOfWork {
    tx: SharedPgTransaction,
    repairs: PgRepairTxRepository,
    payments: PgPaymentTxRepository,
}

impl PgPaymentUnitOfWork {
    pub async fn begin(pool: &sqlx::PgPool) -> AppResult<Self> {
        let tx = SharedPgTransaction::begin(pool, "begin payment unit of work").await?;

        Ok(Self {
            repairs: PgRepairTxRepository::new(tx.clone()),
            payments: PgPaymentTxRepository::new(tx.clone()),
            tx,
        })
    }
}

#[async_trait]
impl PaymentUnitOfWork for PgPaymentUnitOfWork {
    type Repairs = PgRepairTxRepository;
    type Payments = PgPaymentTxRepository;

    fn repairs(&self) -> &Self::Repairs {
        &self.repairs
    }

    fn payments(&self) -> &Self::Payments {
        &self.payments
    }

    async fn commit(&self) -> AppResult<()> {
        self.tx.commit("commit payment unit of work").await
    }

    async fn rollback(&self) -> AppResult<()> {
        self.tx.rollback("rollback payment unit of work").await
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
pub struct PgPaymentTxRepository {
    tx: SharedPgTransaction,
}

impl PgPaymentTxRepository {
    fn new(tx: SharedPgTransaction) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl PaymentRepository for PgPaymentTxRepository {
    async fn get(&self, id: PaymentId) -> AppResult<Option<Payment>> {
        let mut guard = self.tx.lock("get payment").await?;
        let tx = guard.transaction()?;

        let row = sqlx::query_as::<_, PaymentRow>(
            r#"
            SELECT
                id,
                repair_id,
                amount,
                currency,
                method,
                comment,
                paid_at,
                created_at
            FROM payments
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&mut **tx)
        .await
        .map_err(|error| repository_error("get payment", error))?;

        row.as_ref().map(mappers::payment::to_domain).transpose()
    }

    async fn save(&self, payment: &Payment) -> AppResult<()> {
        let amount = payment.amount();

        let mut guard = self.tx.lock("save payment").await?;
        let tx = guard.transaction()?;

        sqlx::query(
            r#"
            INSERT INTO payments (
                id,
                repair_id,
                amount,
                currency,
                method,
                comment,
                paid_at,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO UPDATE SET
                repair_id = EXCLUDED.repair_id,
                amount = EXCLUDED.amount,
                currency = EXCLUDED.currency,
                method = EXCLUDED.method,
                comment = EXCLUDED.comment,
                paid_at = EXCLUDED.paid_at
            "#,
        )
        .bind(payment.id().as_uuid())
        .bind(payment.repair_id().as_uuid())
        .bind(amount.amount_minor())
        .bind(currency_code(amount.currency()))
        .bind(payment.method().to_string())
        .bind(payment.comment().map(|comment| comment.as_str()))
        .bind(payment.paid_at())
        .bind(payment.created_at())
        .execute(&mut **tx)
        .await
        .map_err(|error| repository_error("save payment", error))?;

        Ok(())
    }

    async fn list_by_repair(&self, repair_id: RepairId) -> AppResult<Vec<Payment>> {
        let mut guard = self.tx.lock("list payments by repair").await?;
        let tx = guard.transaction()?;

        let rows = sqlx::query_as::<_, PaymentRow>(
            r#"
            SELECT
                id,
                repair_id,
                amount,
                currency,
                method,
                comment,
                paid_at,
                created_at
            FROM payments
            WHERE repair_id = $1
            ORDER BY paid_at ASC, id ASC
            "#,
        )
        .bind(repair_id.as_uuid())
        .fetch_all(&mut **tx)
        .await
        .map_err(|error| repository_error("list payments by repair", error))?;

        rows.iter().map(mappers::payment::to_domain).collect()
    }
}
