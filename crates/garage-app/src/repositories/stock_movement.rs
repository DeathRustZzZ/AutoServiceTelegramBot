//! Порт хранения движений склада.
//!
//! Движения объясняют изменение остатка и дают аудит склада, но не являются
//! источником текущего количества: это ответственность агрегата `Part`.

use async_trait::async_trait;
use garage_domain::{PartId, StockMovement, StockMovementId};
use std::sync::Arc;

use crate::AppResult;

/// Порт хранения исторических движений склада.
///
/// `StockMovement` объясняет изменение остатка, но не меняет `Part.quantity`.
/// Будущий use case будет сохранять изменение `Part` и движение склада вместе.
#[async_trait]
pub trait StockMovementRepository: Send + Sync {
    /// Возвращает движение склада или `None`.
    async fn get(&self, id: StockMovementId) -> AppResult<Option<StockMovement>>;
    /// Сохраняет движение склада.
    async fn save(&self, movement: &StockMovement) -> AppResult<()>;
    /// Возвращает историю движений по складской позиции.
    async fn list_by_part(&self, part_id: PartId) -> AppResult<Vec<StockMovement>>;
}

/// Делегирующая реализация для `Arc<dyn StockMovementRepository>`.
#[async_trait]
impl<T> StockMovementRepository for Arc<T>
where
    T: StockMovementRepository + ?Sized,
{
    async fn get(&self, id: StockMovementId) -> AppResult<Option<StockMovement>> {
        (**self).get(id).await
    }

    async fn save(&self, movement: &StockMovement) -> AppResult<()> {
        (**self).save(movement).await
    }

    async fn list_by_part(&self, part_id: PartId) -> AppResult<Vec<StockMovement>> {
        (**self).list_by_part(part_id).await
    }
}
