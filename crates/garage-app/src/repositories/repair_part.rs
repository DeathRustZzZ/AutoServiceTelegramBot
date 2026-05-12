use async_trait::async_trait;
use garage_domain::{RepairId, RepairPart, RepairPartId};
use std::sync::Arc;

use crate::AppResult;

/// Порт хранения запчастей, использованных в ремонте.
///
/// `RepairPart` фиксирует строку расхода запчасти, но не списывает склад и не
/// меняет агрегированные суммы ремонта. Эти операции будут координироваться
/// прикладным сценарием.
#[async_trait]
pub trait RepairPartRepository: Send + Sync {
    /// Возвращает строку использованной запчасти или `None`.
    async fn get(&self, id: RepairPartId) -> AppResult<Option<RepairPart>>;
    /// Сохраняет строку использованной запчасти.
    async fn save(&self, repair_part: &RepairPart) -> AppResult<()>;
    /// Возвращает все использованные запчасти ремонта.
    async fn list_by_repair(&self, repair_id: RepairId) -> AppResult<Vec<RepairPart>>;
}

/// Делегирующая реализация для `Arc<dyn RepairPartRepository>`.
#[async_trait]
impl<T> RepairPartRepository for Arc<T>
where
    T: RepairPartRepository + ?Sized,
{
    async fn get(&self, id: RepairPartId) -> AppResult<Option<RepairPart>> {
        (**self).get(id).await
    }

    async fn save(&self, repair_part: &RepairPart) -> AppResult<()> {
        (**self).save(repair_part).await
    }

    async fn list_by_repair(&self, repair_id: RepairId) -> AppResult<Vec<RepairPart>> {
        (**self).list_by_repair(repair_id).await
    }
}
