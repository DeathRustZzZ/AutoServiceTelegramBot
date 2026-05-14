use async_trait::async_trait;
use garage_app::{AppResult, CarRepository};
use garage_domain::{Car, CarId, ClientId};

use crate::mappers;
use crate::models::CarRow;
use crate::repositories::repository_error;

#[derive(Clone)]
pub struct PgCarRepository {
    pool: sqlx::PgPool,
}

impl PgCarRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CarRepository for PgCarRepository {
    async fn get(&self, id: CarId) -> AppResult<Option<Car>> {
        let row = sqlx::query_as::<_, CarRow>(
            r#"
            SELECT
                id,
                client_id,
                make,
                model,
                year,
                license_plate,
                vin,
                notes,
                registration_document_photo_ref,
                status,
                created_at,
                updated_at
            FROM cars
            WHERE id = $1
              AND status = 'active'
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| repository_error("get car", error))?;

        row.as_ref().map(mappers::car::to_domain).transpose()
    }

    async fn save(&self, car: &Car) -> AppResult<()> {
        let year = car
            .year()
            .map(|year| i16::try_from(year.value()))
            .transpose()
            .map_err(|_| repository_error("save car", "year does not fit into i16"))?;

        sqlx::query(
            r#"
            INSERT INTO cars (
                id,
                client_id,
                make,
                model,
                year,
                license_plate,
                vin,
                notes,
                registration_document_photo_ref,
                status,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (id) DO UPDATE SET
                client_id = EXCLUDED.client_id,
                make = EXCLUDED.make,
                model = EXCLUDED.model,
                year = EXCLUDED.year,
                license_plate = EXCLUDED.license_plate,
                vin = EXCLUDED.vin,
                notes = EXCLUDED.notes,
                registration_document_photo_ref = EXCLUDED.registration_document_photo_ref,
                status = EXCLUDED.status,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(car.id().as_uuid())
        .bind(car.client_id().as_uuid())
        .bind(car.make().as_str())
        .bind(car.model().as_str())
        .bind(year)
        .bind(car.license_plate().map(|plate| plate.as_str()))
        .bind(car.vin().map(|vin| vin.as_str()))
        .bind(car.notes().map(|notes| notes.as_str()))
        .bind(
            car.registration_document_photo()
                .map(|photo| photo.as_str()),
        )
        .bind(car.status().to_string())
        .bind(car.created_at())
        .bind(car.updated_at())
        .execute(&self.pool)
        .await
        .map_err(|error| repository_error("save car", error))?;

        Ok(())
    }

    async fn list_by_client(&self, client_id: ClientId) -> AppResult<Vec<Car>> {
        let rows = sqlx::query_as::<_, CarRow>(
            r#"
            SELECT
                id,
                client_id,
                make,
                model,
                year,
                license_plate,
                vin,
                notes,
                registration_document_photo_ref,
                status,
                created_at,
                updated_at
            FROM cars
            WHERE client_id = $1
              AND status = 'active'
            ORDER BY created_at DESC, id ASC
            "#,
        )
        .bind(client_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|error| repository_error("list cars by client", error))?;

        rows.iter().map(mappers::car::to_domain).collect()
    }
}
