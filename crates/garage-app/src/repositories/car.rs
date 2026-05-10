use async_trait::async_trait;
use garage_domain::{Car, CarId, ClientId};
use std::sync::Arc;

use crate::AppResult;

/// Persistence port for cars.
#[async_trait]
pub trait CarRepository: Send + Sync {
    async fn get(&self, id: CarId) -> AppResult<Option<Car>>;
    async fn save(&self, car: &Car) -> AppResult<()>;
    async fn list_by_client(&self, client_id: ClientId) -> AppResult<Vec<Car>>;
}

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
