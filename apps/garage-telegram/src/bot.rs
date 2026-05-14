use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::*;

use crate::container::AppContainer;
use crate::routing;
use crate::state::SessionData;

pub async fn run(container: AppContainer) {
    let bot = Bot::new(container.bot_token());
    let pool = container.pool();
    let clients = container.clients();
    let cars = container.cars();

    Dispatcher::builder(bot, routing::schema())
        .dependencies(dptree::deps![
            InMemStorage::<SessionData>::new(),
            pool,
            clients,
            cars,
            container
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
