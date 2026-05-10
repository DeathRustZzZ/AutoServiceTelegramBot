use async_trait::async_trait;
use garage_domain::{PartId, PartSupply, PartSupplyId};
use std::sync::Arc;

use crate::AppResult;

/// Порт хранения поставок запчастей.
///
/// Поставки хранят историю пополнений, но не являются вложенной коллекцией в
/// `Part`. Поэтому у них отдельный репозиторий.
#[async_trait]
pub trait PartSupplyRepository: Send + Sync {
    /// Возвращает поставку или `None`.
    async fn get(&self, id: PartSupplyId) -> AppResult<Option<PartSupply>>;
    /// Сохраняет поставку.
    async fn save(&self, supply: &PartSupply) -> AppResult<()>;
    /// Возвращает поставки по складской позиции.
    async fn list_by_part(&self, part_id: PartId) -> AppResult<Vec<PartSupply>>;
}

/// Делегирующая реализация для `Arc<dyn PartSupplyRepository>`.
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
