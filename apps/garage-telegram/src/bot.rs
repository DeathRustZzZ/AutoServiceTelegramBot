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
    let bookings = container.bookings();
    let parts = container.parts();
    let repairs = container.repairs();
    let repair_parts = container.repair_parts();
    let payments = container.payments();
    let stock_movements = container.stock_movements();

    Dispatcher::builder(bot, routing::schema())
        .dependencies(dptree::deps![
            InMemStorage::<SessionData>::new(),
            pool,
            clients,
            cars,
            bookings,
            parts,
            repairs,
            repair_parts,
            payments,
            stock_movements,
            container
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
