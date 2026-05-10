use async_trait::async_trait;
use chrono::{DateTime, Utc};
use garage_domain::{Booking, BookingId, CarId, ClientId};
use std::sync::Arc;

use crate::AppResult;

/// Persistence port for bookings.
#[async_trait]
pub trait BookingRepository: Send + Sync {
    async fn get(&self, id: BookingId) -> AppResult<Option<Booking>>;
    async fn save(&self, booking: &Booking) -> AppResult<()>;
    async fn list_by_client(&self, client_id: ClientId) -> AppResult<Vec<Booking>>;
    async fn list_by_car(&self, car_id: CarId) -> AppResult<Vec<Booking>>;
    async fn list_scheduled_between(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> AppResult<Vec<Booking>>;
}

#[async_trait]
impl<T> BookingRepository for Arc<T>
where
    T: BookingRepository + ?Sized,
{
    async fn get(&self, id: BookingId) -> AppResult<Option<Booking>> {
        (**self).get(id).await
    }

    async fn save(&self, booking: &Booking) -> AppResult<()> {
        (**self).save(booking).await
    }

    async fn list_by_client(&self, client_id: ClientId) -> AppResult<Vec<Booking>> {
        (**self).list_by_client(client_id).await
    }

    async fn list_by_car(&self, car_id: CarId) -> AppResult<Vec<Booking>> {
        (**self).list_by_car(car_id).await
    }

    async fn list_scheduled_between(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> AppResult<Vec<Booking>> {
        (**self).list_scheduled_between(from, to).await
    }
}
