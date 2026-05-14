use teloxide::prelude::*;

use crate::handlers;
use crate::messages;
use crate::state::{HandlerResult, SessionData, UserDialogue};
use crate::ui::render::{render_screen, Screen};

pub async fn handle(
    bot: Bot,
    dialogue: UserDialogue,
    query: CallbackQuery,
    session: SessionData,
) -> HandlerResult {
    bot.answer_callback_query(query.id).await?;

    let Some(message) = query.message.as_ref() else {
        bot.send_message(query.from.id, messages::errors::callback_without_message())
            .await?;
        return Ok(());
    };

    let chat_id = message.chat().id;
    let data = query.data.as_deref().unwrap_or_default();

    match data {
        "nav:main" => {
            let mut session = session;
            session.reset_dialog();
            render_screen(
                &bot,
                &dialogue,
                chat_id,
                session,
                Screen::new(
                    messages::main::main_menu(),
                    crate::keyboards::main::main_menu(),
                ),
            )
            .await
        }
        "nav:clients" => handlers::clients::show_menu(&bot, &dialogue, chat_id, session).await,
        "client:add" => handlers::clients::begin_add(&bot, &dialogue, chat_id, session).await,
        "client:confirm" => {
            handlers::clients::confirm_placeholder(&bot, &dialogue, chat_id, session).await
        }
        "nav:bookings" => {
            render_screen(
                &bot,
                &dialogue,
                chat_id,
                session,
                Screen::new(
                    messages::main::not_implemented("Записи"),
                    crate::keyboards::main::main_menu(),
                ),
            )
            .await
        }
        "nav:cars" => {
            render_screen(
                &bot,
                &dialogue,
                chat_id,
                session,
                Screen::new(
                    messages::main::not_implemented("Авто"),
                    crate::keyboards::main::main_menu(),
                ),
            )
            .await
        }
        "nav:stock" => {
            render_screen(
                &bot,
                &dialogue,
                chat_id,
                session,
                Screen::new(
                    messages::main::not_implemented("Склад"),
                    crate::keyboards::main::main_menu(),
                ),
            )
            .await
        }
        "nav:low_stock" => {
            render_screen(
                &bot,
                &dialogue,
                chat_id,
                session,
                Screen::new(
                    messages::main::not_implemented("Остатки"),
                    crate::keyboards::main::main_menu(),
                ),
            )
            .await
        }
        "nav:search" => {
            render_screen(
                &bot,
                &dialogue,
                chat_id,
                session,
                Screen::new(
                    messages::main::not_implemented("Поиск"),
                    crate::keyboards::main::main_menu(),
                ),
            )
            .await
        }
        _ => Ok(()),
    }
}
