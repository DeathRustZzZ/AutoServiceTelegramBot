use std::sync::Arc;

use garage_app::ClientService;
use garage_infra::db::pool::create_pool;
use garage_infra::repositories::client::PgClientRepository;
use sqlx::PgPool;

use crate::config::Config;

#[derive(Clone)]
pub struct AppContainer {
    config: Config,
    pool: PgPool,
    clients: Arc<PgClientRepository>,
    client_service: Arc<ClientService<PgClientRepository>>,
}

impl AppContainer {
    pub async fn new(config: Config) -> Result<Self, sqlx::Error> {
        let pool = create_pool(&config.database_url).await?;
        let clients = Arc::new(PgClientRepository::new(pool.clone()));
        let client_service = Arc::new(ClientService::new((*clients).clone()));

        Ok(Self {
            config,
            pool,
            clients,
            client_service,
        })
    }

    pub fn bot_token(&self) -> &str {
        &self.config.bot_token
    }

    pub fn pool(&self) -> PgPool {
        self.pool.clone()
    }

    pub fn clients(&self) -> Arc<PgClientRepository> {
        self.clients.clone()
    }

    pub fn client_service(&self) -> Arc<ClientService<PgClientRepository>> {
        self.client_service.clone()
    }
}
