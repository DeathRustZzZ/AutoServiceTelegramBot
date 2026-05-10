use async_trait::async_trait;
use chrono::{DateTime, Utc};
use garage_domain::{CarId, ClientId, Repair, RepairId};
use std::sync::Arc;

use crate::AppResult;

/// Persistence port for repairs.
#[async_trait]
pub trait RepairRepository: Send + Sync {
    async fn get(&self, id: RepairId) -> AppResult<Option<Repair>>;
    async fn save(&self, repair: &Repair) -> AppResult<()>;
    async fn list_by_client(&self, client_id: ClientId) -> AppResult<Vec<Repair>>;
    async fn list_by_car(&self, car_id: CarId) -> AppResult<Vec<Repair>>;
    async fn list_completed_between(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> AppResult<Vec<Repair>>;
}

#[async_trait]
impl<T> RepairRepository for Arc<T>
where
    T: RepairRepository + ?Sized,
{
    async fn get(&self, id: RepairId) -> AppResult<Option<Repair>> {
        (**self).get(id).await
    }

    async fn save(&self, repair: &Repair) -> AppResult<()> {
        (**self).save(repair).await
    }

    async fn list_by_client(&self, client_id: ClientId) -> AppResult<Vec<Repair>> {
        (**self).list_by_client(client_id).await
    }

    async fn list_by_car(&self, car_id: CarId) -> AppResult<Vec<Repair>> {
        (**self).list_by_car(car_id).await
    }

    async fn list_completed_between(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> AppResult<Vec<Repair>> {
        (**self).list_completed_between(from, to).await
    }
}
