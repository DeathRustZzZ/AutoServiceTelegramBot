use async_trait::async_trait;
use garage_app::{AppResult, PaymentRepository};
use garage_domain::{Payment, PaymentId, RepairId};

use crate::mappers;
use crate::models::PaymentRow;
use crate::repositories::{currency_code, repository_error};

#[derive(Clone)]
pub struct PgPaymentRepository {
    pool: sqlx::PgPool,
}

impl PgPaymentRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PaymentRepository for PgPaymentRepository {
    async fn get(&self, id: PaymentId) -> AppResult<Option<Payment>> {
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
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| repository_error("get payment", error))?;

        row.as_ref().map(mappers::payment::to_domain).transpose()
    }

    async fn save(&self, payment: &Payment) -> AppResult<()> {
        let amount = payment.amount();

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
        .execute(&self.pool)
        .await
        .map_err(|error| repository_error("save payment", error))?;

        Ok(())
    }

    async fn list_by_repair(&self, repair_id: RepairId) -> AppResult<Vec<Payment>> {
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
        .fetch_all(&self.pool)
        .await
        .map_err(|error| repository_error("list payments by repair", error))?;

        rows.iter().map(mappers::payment::to_domain).collect()
    }
}
