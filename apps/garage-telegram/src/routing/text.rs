use teloxide::prelude::*;

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

    if let DialogState::AddClient(step) = session.dialog {
        return handlers::clients::handle_add_text(bot, dialogue, msg, session, step, text).await;
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
        reply::NAV_STOCK => Some(Screen::new(
            messages::main::not_implemented("Склад"),
            crate::keyboards::main::main_menu(),
        )),
        reply::NAV_LOW_STOCK => Some(Screen::new(
            messages::main::not_implemented("Остатки"),
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
