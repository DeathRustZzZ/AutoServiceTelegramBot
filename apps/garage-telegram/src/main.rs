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
    let container = AppContainer::new(config)
        .await
        .expect("failed to initialize application container");

    bot::run(container).await;
}
