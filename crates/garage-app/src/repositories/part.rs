use async_trait::async_trait;
use garage_domain::{Part, PartId};
use std::sync::Arc;

use crate::AppResult;

/// Persistence port for warehouse parts.
#[async_trait]
pub trait PartRepository: Send + Sync {
    async fn get(&self, id: PartId) -> AppResult<Option<Part>>;
    async fn save(&self, part: &Part) -> AppResult<()>;
    async fn list_low_stock(&self) -> AppResult<Vec<Part>>;
    async fn search(&self, query: &str) -> AppResult<Vec<Part>>;
}

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
