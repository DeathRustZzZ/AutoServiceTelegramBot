use teloxide::prelude::*;
use uuid::Uuid;

use crate::container::AppContainer;
use crate::handlers;
use crate::messages;
use crate::state::{HandlerResult, SessionData, UserDialogue};
use crate::ui::render::{render_screen, Screen};

pub async fn handle(
    bot: Bot,
    dialogue: UserDialogue,
    query: CallbackQuery,
    session: SessionData,
    container: AppContainer,
) -> HandlerResult {
    bot.answer_callback_query(query.id.clone()).await?;

    let Some(message) = query.message.as_ref() else {
        bot.send_message(query.from.id, messages::errors::callback_without_message())
            .await?;
        return Ok(());
    };

    let chat_id = message.chat().id;
    let data = query.data.as_deref().unwrap_or_default();

    if let Some(page) = data.strip_prefix("client:list:") {
        return match page.parse::<usize>() {
            Ok(page) => {
                handlers::clients::show_list(&bot, &dialogue, chat_id, container, session, page)
                    .await
            }
            Err(_) => {
                render_screen(
                    &bot,
                    &dialogue,
                    chat_id,
                    session,
                    Screen::new(
                        messages::errors::invalid_callback(),
                        crate::keyboards::clients::clients_menu(),
                    ),
                )
                .await
            }
        };
    }

    if let Some(id) = data.strip_prefix("client:open:") {
        return match Uuid::parse_str(id) {
            Ok(id) => {
                handlers::clients::show_card(
                    &bot,
                    &dialogue,
                    chat_id,
                    container,
                    session,
                    garage_domain::ClientId::from_uuid(id),
                )
                .await
            }
            Err(_) => {
                render_screen(
                    &bot,
                    &dialogue,
                    chat_id,
                    session,
                    Screen::new(
                        messages::errors::invalid_callback(),
                        crate::keyboards::clients::clients_menu(),
                    ),
                )
                .await
            }
        };
    }

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
        "client:search" => handlers::clients::begin_search(&bot, &dialogue, chat_id, session).await,
        "client:confirm" => {
            handlers::clients::confirm(&bot, &dialogue, chat_id, container, session).await
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
