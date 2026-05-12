use async_trait::async_trait;
use chrono::{DateTime, Utc};
use garage_app::{AppResult, BookingRepository};
use garage_domain::{Booking, BookingId, CarId, ClientId};

use crate::mappers;
use crate::models::BookingRow;
use crate::repositories::repository_error;

#[derive(Clone)]
pub struct PgBookingRepository {
    pool: sqlx::PgPool,
}

impl PgBookingRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BookingRepository for PgBookingRepository {
    async fn get(&self, id: BookingId) -> AppResult<Option<Booking>> {
        let row = sqlx::query_as::<_, BookingRow>(
            r#"
            SELECT
                id,
                client_id,
                car_id,
                scheduled_at,
                status,
                reason,
                notes,
                closed_at,
                created_at,
                updated_at
            FROM bookings
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| repository_error("get booking", error))?;

        row.as_ref().map(mappers::booking::to_domain).transpose()
    }

    async fn save(&self, booking: &Booking) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO bookings (
                id,
                client_id,
                car_id,
                scheduled_at,
                status,
                reason,
                notes,
                closed_at,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (id) DO UPDATE SET
                client_id = EXCLUDED.client_id,
                car_id = EXCLUDED.car_id,
                scheduled_at = EXCLUDED.scheduled_at,
                status = EXCLUDED.status,
                reason = EXCLUDED.reason,
                notes = EXCLUDED.notes,
                closed_at = EXCLUDED.closed_at,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(booking.id().as_uuid())
        .bind(booking.client_id().as_uuid())
        .bind(booking.car_id().as_uuid())
        .bind(booking.scheduled_at())
        .bind(booking.status().to_string())
        .bind(booking.reason().as_str())
        .bind(booking.notes().map(|notes| notes.as_str()))
        .bind(booking.closed_at())
        .bind(booking.created_at())
        .bind(booking.updated_at())
        .execute(&self.pool)
        .await
        .map_err(|error| repository_error("save booking", error))?;

        Ok(())
    }

    async fn list_by_client(&self, client_id: ClientId) -> AppResult<Vec<Booking>> {
        let rows = sqlx::query_as::<_, BookingRow>(
            r#"
            SELECT
                id,
                client_id,
                car_id,
                scheduled_at,
                status,
                reason,
                notes,
                closed_at,
                created_at,
                updated_at
            FROM bookings
            WHERE client_id = $1
            ORDER BY scheduled_at ASC, id ASC
            "#,
        )
        .bind(client_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|error| repository_error("list bookings by client", error))?;

        rows.iter().map(mappers::booking::to_domain).collect()
    }

    async fn list_by_car(&self, car_id: CarId) -> AppResult<Vec<Booking>> {
        let rows = sqlx::query_as::<_, BookingRow>(
            r#"
            SELECT
                id,
                client_id,
                car_id,
                scheduled_at,
                status,
                reason,
                notes,
                closed_at,
                created_at,
                updated_at
            FROM bookings
            WHERE car_id = $1
            ORDER BY scheduled_at ASC, id ASC
            "#,
        )
        .bind(car_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|error| repository_error("list bookings by car", error))?;

        rows.iter().map(mappers::booking::to_domain).collect()
    }

    async fn list_scheduled_between(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> AppResult<Vec<Booking>> {
        let rows = sqlx::query_as::<_, BookingRow>(
            r#"
            SELECT
                id,
                client_id,
                car_id,
                scheduled_at,
                status,
                reason,
                notes,
                closed_at,
                created_at,
                updated_at
            FROM bookings
            WHERE status = 'scheduled'
              AND scheduled_at >= $1
              AND scheduled_at < $2
            ORDER BY scheduled_at ASC, id ASC
            "#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| repository_error("list scheduled bookings between", error))?;

        rows.iter().map(mappers::booking::to_domain).collect()
    }
}
