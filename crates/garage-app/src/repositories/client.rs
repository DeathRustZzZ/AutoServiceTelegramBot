//! Порт хранения клиентов.
//!
//! Клиенты являются корневым справочником для большинства сценариев, поэтому
//! контракт включает постраничный список, поиск и сохранение агрегата.

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
    /// Возвращает страницу активных клиентов.
    async fn list(&self, limit: u32, offset: u32) -> AppResult<Vec<Client>>;
    /// Ищет активных клиентов по имени или телефону.
    async fn search(&self, query: &str, limit: u32, offset: u32) -> AppResult<Vec<Client>>;
    /// Сохраняет текущее состояние клиента.
    async fn save(&self, client: &Client) -> AppResult<()>;
}

/// Делегирующая реализация для разделяемого объекта репозитория.
#[async_trait]
impl<T> ClientRepository for Arc<T>
where
    T: ClientRepository + ?Sized,
{
    async fn get(&self, id: ClientId) -> AppResult<Option<Client>> {
        (**self).get(id).await
    }

    async fn list(&self, limit: u32, offset: u32) -> AppResult<Vec<Client>> {
        (**self).list(limit, offset).await
    }

    async fn search(&self, query: &str, limit: u32, offset: u32) -> AppResult<Vec<Client>> {
        (**self).search(query, limit, offset).await
    }

    async fn save(&self, client: &Client) -> AppResult<()> {
        (**self).save(client).await
    }
}
