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

    if let Some(id) = data.strip_prefix("client:cars:") {
        return match Uuid::parse_str(id) {
            Ok(id) => {
                handlers::cars::show_client_cars(
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

    if let Some(id) = data.strip_prefix("car:add:") {
        return match Uuid::parse_str(id) {
            Ok(id) => {
                handlers::cars::begin_add(
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

    if let Some(id) = data.strip_prefix("car:open:") {
        return match Uuid::parse_str(id) {
            Ok(id) => {
                handlers::cars::show_card(
                    &bot,
                    &dialogue,
                    chat_id,
                    container,
                    session,
                    garage_domain::CarId::from_uuid(id),
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

    if let Some(id) = data.strip_prefix("booking:open:") {
        return match Uuid::parse_str(id) {
            Ok(id) => {
                handlers::bookings::show_card(
                    &bot,
                    &dialogue,
                    chat_id,
                    container,
                    session,
                    garage_domain::BookingId::from_uuid(id),
                )
                .await
            }
            Err(_) => invalid_callback(&bot, &dialogue, chat_id, session).await,
        };
    }

    if let Some(id) = data.strip_prefix("booking:client:") {
        return match Uuid::parse_str(id) {
            Ok(id) => {
                handlers::bookings::select_client(
                    &bot,
                    &dialogue,
                    chat_id,
                    container,
                    session,
                    garage_domain::ClientId::from_uuid(id),
                )
                .await
            }
            Err(_) => invalid_callback(&bot, &dialogue, chat_id, session).await,
        };
    }

    if let Some(id) = data.strip_prefix("booking:car:") {
        return match Uuid::parse_str(id) {
            Ok(id) => {
                handlers::bookings::select_car(
                    &bot,
                    &dialogue,
                    chat_id,
                    container,
                    session,
                    garage_domain::CarId::from_uuid(id),
                )
                .await
            }
            Err(_) => invalid_callback(&bot, &dialogue, chat_id, session).await,
        };
    }

    if let Some(id) = data.strip_prefix("booking:complete:") {
        return match Uuid::parse_str(id) {
            Ok(id) => {
                handlers::bookings::complete(
                    &bot,
                    &dialogue,
                    chat_id,
                    container,
                    session,
                    garage_domain::BookingId::from_uuid(id),
                )
                .await
            }
            Err(_) => invalid_callback(&bot, &dialogue, chat_id, session).await,
        };
    }

    if let Some(id) = data.strip_prefix("booking:cancel:") {
        return match Uuid::parse_str(id) {
            Ok(id) => {
                handlers::bookings::cancel(
                    &bot,
                    &dialogue,
                    chat_id,
                    container,
                    session,
                    garage_domain::BookingId::from_uuid(id),
                )
                .await
            }
            Err(_) => invalid_callback(&bot, &dialogue, chat_id, session).await,
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
        "nav:bookings" => handlers::bookings::show_menu(&bot, &dialogue, chat_id, session).await,
        "client:add" => handlers::clients::begin_add(&bot, &dialogue, chat_id, session).await,
        "client:search" => handlers::clients::begin_search(&bot, &dialogue, chat_id, session).await,
        "client:confirm" => {
            handlers::clients::confirm(&bot, &dialogue, chat_id, container, session).await
        }
        "car:confirm" => {
            handlers::cars::confirm(&bot, &dialogue, chat_id, container, session).await
        }
        "booking:today" => {
            handlers::bookings::show_today(&bot, &dialogue, chat_id, container, session).await
        }
        "booking:tomorrow" => {
            handlers::bookings::show_tomorrow(&bot, &dialogue, chat_id, container, session).await
        }
        "booking:add" => handlers::bookings::begin_add(&bot, &dialogue, chat_id, session).await,
        "booking:confirm" => {
            handlers::bookings::confirm(&bot, &dialogue, chat_id, container, session).await
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

async fn invalid_callback(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    session: SessionData,
) -> HandlerResult {
    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            messages::errors::invalid_callback(),
            crate::keyboards::main::main_menu(),
        ),
    )
    .await
}
