use async_trait::async_trait;
use garage_domain::{PartId, PartSupply, PartSupplyId};
use std::sync::Arc;

use crate::AppResult;

/// Persistence port for part supplies.
#[async_trait]
pub trait PartSupplyRepository: Send + Sync {
    async fn get(&self, id: PartSupplyId) -> AppResult<Option<PartSupply>>;
    async fn save(&self, supply: &PartSupply) -> AppResult<()>;
    async fn list_by_part(&self, part_id: PartId) -> AppResult<Vec<PartSupply>>;
}

#[async_trait]
impl<T> PartSupplyRepository for Arc<T>
where
    T: PartSupplyRepository + ?Sized,
{
    async fn get(&self, id: PartSupplyId) -> AppResult<Option<PartSupply>> {
        (**self).get(id).await
    }

    async fn save(&self, supply: &PartSupply) -> AppResult<()> {
        (**self).save(supply).await
    }

    async fn list_by_part(&self, part_id: PartId) -> AppResult<Vec<PartSupply>> {
        (**self).list_by_part(part_id).await
    }
}
