use async_trait::async_trait;
use garage_domain::{Part, PartId};
use std::sync::Arc;

use crate::AppResult;

/// Порт хранения складских позиций.
///
/// Поиск оставлен абстрактным: infra может искать по названию, SKU, индексам
/// PostgreSQL или другой стратегии, не меняя сервисы.
#[async_trait]
pub trait PartRepository: Send + Sync {
    /// Возвращает складскую позицию или `None`.
    async fn get(&self, id: PartId) -> AppResult<Option<Part>>;
    /// Сохраняет складскую позицию.
    async fn save(&self, part: &Part) -> AppResult<()>;
    /// Возвращает позиции с низким остатком.
    async fn list_low_stock(&self) -> AppResult<Vec<Part>>;
    /// Ищет позиции по пользовательскому запросу.
    async fn search(&self, query: &str) -> AppResult<Vec<Part>>;
}

/// Делегирующая реализация для `Arc<dyn PartRepository>`.
#[async_trait]
impl<T> PartRepository for Arc<T>
where
    T: PartRepository + ?Sized,
{
    async fn get(&self, id: PartId) -> AppResult<Option<Part>> {
        (**self).get(id).await
    }

    async fn save(&self, part: &Part) -> AppResult<()> {
        (**self).save(part).await
    }

    async fn list_low_stock(&self) -> AppResult<Vec<Part>> {
        (**self).list_low_stock().await
    }

    async fn search(&self, query: &str) -> AppResult<Vec<Part>> {
        (**self).search(query).await
    }
}
