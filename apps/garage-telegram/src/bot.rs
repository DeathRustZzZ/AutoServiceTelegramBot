//! Запуск Telegram dispatcher.
//!
//! Модуль связывает teloxide `Bot`, схему маршрутизации и зависимости,
//! доступные handler'ам через dptree. Здесь нет пользовательских сценариев:
//! все update'ы дальше проходят через `routing`.

use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::*;

use crate::container::AppContainer;
use crate::routing;
use crate::state::SessionData;

/// Запускает long polling dispatcher с зависимостями приложения.
///
/// Репозитории прокидываются отдельно, потому что некоторые handler'ы пока
/// читают данные напрямую, а транзакционные команды проходят через
/// `AppContainer`. Это сохраняет явную границу между Telegram-адаптером и
/// прикладным слоем.
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
