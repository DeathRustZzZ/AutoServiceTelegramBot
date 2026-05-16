//! Точка входа Telegram-адаптера автосервиса.
//!
//! Этот бинарник отвечает только за инфраструктурный запуск: читает окружение,
//! поднимает контейнер зависимостей и передает управление teloxide dispatcher.
//! Бизнес-сценарии остаются в `garage-app`, а PostgreSQL-детали - в
//! `garage-infra`.

mod bot;
mod config;
mod container;
mod handlers;
mod keyboards;
mod messages;
mod routing;
mod state;
mod ui;

use crate::config::Config;
use crate::container::AppContainer;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config = Config::from_env();
    if config.owner_chat_id.is_none() {
        tracing::warn!("OWNER_CHAT_ID is not set; bot accepts messages from anyone");
    }

    let container = AppContainer::new(config)
        .await
        .expect("failed to initialize application container");

    bot::run(container).await;
}
