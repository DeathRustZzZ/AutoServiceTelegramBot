use async_trait::async_trait;
use garage_domain::{Client, ClientId};
use std::sync::Arc;

use crate::AppResult;

/// Порт хранения клиентов.
///
/// `Client` - самостоятельный агрегат. Для базовых сценариев достаточно
/// загрузки по id и сохранения агрегата целиком.
#[async_trait]
pub trait ClientRepository: Send + Sync {
    /// Возвращает клиента или `None`, если id не найден.
    async fn get(&self, id: ClientId) -> AppResult<Option<Client>>;
    /// Сохраняет текущее состояние клиента.
    async fn save(&self, client: &Client) -> AppResult<()>;
}

/// Делегирующая реализация для shared repository object.
#[async_trait]
impl<T> ClientRepository for Arc<T>
where
    T: ClientRepository + ?Sized,
{
    async fn get(&self, id: ClientId) -> AppResult<Option<Client>> {
        (**self).get(id).await
    }

    async fn save(&self, client: &Client) -> AppResult<()> {
        (**self).save(client).await
    }
}
