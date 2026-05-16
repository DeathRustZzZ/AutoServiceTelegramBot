//! Порт расписания визитов клиентов.
//!
//! Модуль описывает минимальные операции чтения и сохранения booking, которые
//! нужны прикладным сценариям без знания о конкретной SQL-схеме.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use garage_domain::{Booking, BookingId, CarId, ClientId};
use std::sync::Arc;

use crate::AppResult;

/// Порт хранения записей на обслуживание.
///
/// Booking используется для расписания визитов. Деньги и работы не попадают в
/// этот репозиторий, потому что они принадлежат `Repair`.
#[async_trait]
pub trait BookingRepository: Send + Sync {
    /// Возвращает запись или `None`, если id не найден.
    async fn get(&self, id: BookingId) -> AppResult<Option<Booking>>;
    /// Сохраняет запись после доменной мутации.
    async fn save(&self, booking: &Booking) -> AppResult<()>;
    /// Возвращает записи клиента.
    async fn list_by_client(&self, client_id: ClientId) -> AppResult<Vec<Booking>>;
    /// Возвращает записи автомобиля.
    async fn list_by_car(&self, car_id: CarId) -> AppResult<Vec<Booking>>;
    /// Возвращает scheduled-записи за UTC-диапазон.
    async fn list_scheduled_between(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> AppResult<Vec<Booking>>;
}

/// Делегирующая реализация для `Arc<dyn BookingRepository>`.
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
