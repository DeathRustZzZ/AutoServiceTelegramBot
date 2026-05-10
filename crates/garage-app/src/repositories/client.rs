use async_trait::async_trait;
use garage_domain::{Client, ClientId};
use std::sync::Arc;

use crate::AppResult;

/// Persistence port for clients.
#[async_trait]
pub trait ClientRepository: Send + Sync {
    async fn get(&self, id: ClientId) -> AppResult<Option<Client>>;
    async fn save(&self, client: &Client) -> AppResult<()>;
}

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
