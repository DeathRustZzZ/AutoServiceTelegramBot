use async_trait::async_trait;
use garage_domain::{Car, CarId, ClientId};
use std::sync::Arc;

use crate::AppResult;

/// Порт хранения автомобилей.
///
/// Репозиторий ничего не знает о Telegram-сценариях. Он только загружает,
/// сохраняет и выбирает автомобили по владельцу.
#[async_trait]
pub trait CarRepository: Send + Sync {
    /// Возвращает автомобиль или `None`, если id не найден.
    async fn get(&self, id: CarId) -> AppResult<Option<Car>>;
    /// Сохраняет агрегат автомобиля целиком.
    async fn save(&self, car: &Car) -> AppResult<()>;
    /// Возвращает все автомобили клиента.
    async fn list_by_client(&self, client_id: ClientId) -> AppResult<Vec<Car>>;
}

/// Делегирующая реализация для `Arc<dyn CarRepository>`.
#[async_trait]
impl<T> CarRepository for Arc<T>
where
    T: CarRepository + ?Sized,
{
    async fn get(&self, id: CarId) -> AppResult<Option<Car>> {
        (**self).get(id).await
    }

    async fn save(&self, car: &Car) -> AppResult<()> {
        (**self).save(car).await
    }

    async fn list_by_client(&self, client_id: ClientId) -> AppResult<Vec<Car>> {
        (**self).list_by_client(client_id).await
    }
}
