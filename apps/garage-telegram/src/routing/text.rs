use teloxide::prelude::*;

use crate::container::AppContainer;
use crate::handlers;
use crate::keyboards::reply;
use crate::messages;
use crate::state::{DialogState, HandlerResult, SessionData, UserDialogue};
use crate::ui::render::{render_screen, Screen};

pub async fn handle(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    session: SessionData,
    container: AppContainer,
) -> HandlerResult {
    let Some(text) = msg.text().map(str::trim).map(str::to_string) else {
        return Ok(());
    };

    if text.as_str() == "/start" {
        return handlers::start::start(bot, dialogue, msg, session).await;
    }

    if text.as_str() == reply::NAV_CLIENTS {
        return handlers::clients::show_menu(&bot, &dialogue, msg.chat.id, session).await;
    }

    if text.as_str() == reply::NAV_BOOKINGS {
        return handlers::bookings::show_menu(&bot, &dialogue, msg.chat.id, session).await;
    }

    if text.as_str() == reply::NAV_STOCK {
        return handlers::parts::show_menu(&bot, &dialogue, msg.chat.id, session).await;
    }

    if text.as_str() == reply::NAV_LOW_STOCK {
        return handlers::parts::show_low_stock(&bot, &dialogue, msg.chat.id, container, session)
            .await;
    }

    if text.as_str() == reply::NAV_REPAIRS {
        return handlers::repairs::show_menu(&bot, &dialogue, msg.chat.id, session).await;
    }

    match session.dialog.clone() {
        DialogState::AddClient(step) => {
            return handlers::clients::handle_add_text(bot, dialogue, msg, session, step, text)
                .await;
        }
        DialogState::SearchClient => {
            return handlers::clients::handle_search_text(
                bot, dialogue, msg, container, session, text,
            )
            .await;
        }
        DialogState::AddCar(step) => {
            return handlers::cars::handle_add_text(
                bot, dialogue, msg, container, session, step, text,
            )
            .await;
        }
        DialogState::AddBooking(step) => {
            return handlers::bookings::handle_add_text(
                bot, dialogue, msg, container, session, step, text,
            )
            .await;
        }
        DialogState::AddPart(step) => {
            return handlers::parts::handle_add_text(bot, dialogue, msg, session, step, text).await;
        }
        DialogState::SearchPart => {
            return handlers::parts::handle_search_text(
                bot, dialogue, msg, container, session, text,
            )
            .await;
        }
        DialogState::SetPartStock(step) => {
            return handlers::parts::handle_set_stock_text(
                bot, dialogue, msg, container, session, step, text,
            )
            .await;
        }
        DialogState::StartRepair(step) => {
            return handlers::repairs::handle_start_text(
                bot, dialogue, msg, container, session, step, text,
            )
            .await;
        }
        DialogState::Idle => {}
    }

    let screen = match text.as_str() {
        reply::NAV_BOOKINGS => Some(Screen::new(
            messages::main::not_implemented("Записи"),
            crate::keyboards::main::main_menu(),
        )),
        reply::NAV_CARS => Some(Screen::new(
            messages::main::not_implemented("Авто"),
            crate::keyboards::main::main_menu(),
        )),
        reply::NAV_SEARCH => Some(Screen::new(
            messages::main::not_implemented("Поиск"),
            crate::keyboards::main::main_menu(),
        )),
        _ => None,
    };

    if let Some(screen) = screen {
        render_screen(&bot, &dialogue, msg.chat.id, session, screen).await
    } else {
        handlers::errors::unknown_text(bot, msg).await
    }
}
