use teloxide::prelude::*;
use uuid::Uuid;

use crate::container::AppContainer;
use crate::handlers;
use crate::messages;
use crate::state::{HandlerResult, SessionData, UserDialogue};
use crate::ui::render::{render_screen, Screen};
use crate::ui::reply_preset::send_reply_keyboard_notice;

pub async fn handle(
    bot: Bot,
    dialogue: UserDialogue,
    query: CallbackQuery,
    session: SessionData,
    container: AppContainer,
) -> HandlerResult {
    if !crate::routing::access::ensure_callback_access(&bot, &query, &container).await? {
        return Ok(());
    }

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
            Err(_) => invalid_callback(&bot, &dialogue, chat_id, session).await,
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
            Err(_) => invalid_callback(&bot, &dialogue, chat_id, session).await,
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
            Err(_) => invalid_callback(&bot, &dialogue, chat_id, session).await,
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
            Err(_) => invalid_callback(&bot, &dialogue, chat_id, session).await,
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
            Err(_) => invalid_callback(&bot, &dialogue, chat_id, session).await,
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

    if let Some(id) = data.strip_prefix("booking:start_repair:") {
        return match Uuid::parse_str(id) {
            Ok(id) => {
                handlers::repairs::begin_start_from_booking(
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

    if let Some(id) = data.strip_prefix("repair:open:") {
        return match Uuid::parse_str(id) {
            Ok(id) => {
                handlers::repairs::show_card(
                    &bot,
                    &dialogue,
                    chat_id,
                    container,
                    session,
                    garage_domain::RepairId::from_uuid(id),
                )
                .await
            }
            Err(_) => invalid_callback(&bot, &dialogue, chat_id, session).await,
        };
    }

    if let Some(id) = data.strip_prefix("repair:complete:") {
        return match Uuid::parse_str(id) {
            Ok(id) => {
                handlers::repairs::complete(
                    &bot,
                    &dialogue,
                    chat_id,
                    container,
                    session,
                    garage_domain::RepairId::from_uuid(id),
                )
                .await
            }
            Err(_) => invalid_callback(&bot, &dialogue, chat_id, session).await,
        };
    }

    if let Some(id) = data.strip_prefix("repair:cancel:") {
        return match Uuid::parse_str(id) {
            Ok(id) => {
                handlers::repairs::cancel(
                    &bot,
                    &dialogue,
                    chat_id,
                    container,
                    session,
                    garage_domain::RepairId::from_uuid(id),
                )
                .await
            }
            Err(_) => invalid_callback(&bot, &dialogue, chat_id, session).await,
        };
    }

    if let Some(id) = data.strip_prefix("repair:payment:") {
        return match Uuid::parse_str(id) {
            Ok(id) => {
                handlers::repairs::begin_payment(
                    &bot,
                    &dialogue,
                    chat_id,
                    container,
                    session,
                    garage_domain::RepairId::from_uuid(id),
                )
                .await
            }
            Err(_) => invalid_callback(&bot, &dialogue, chat_id, session).await,
        };
    }

    if let Some(id) = data.strip_prefix("repair:set_labor:") {
        return match Uuid::parse_str(id) {
            Ok(id) => {
                handlers::repairs::begin_set_labor(
                    &bot,
                    &dialogue,
                    chat_id,
                    container,
                    session,
                    garage_domain::RepairId::from_uuid(id),
                )
                .await
            }
            Err(_) => invalid_callback(&bot, &dialogue, chat_id, session).await,
        };
    }

    if let Some(id) = data.strip_prefix("repair:add_part:") {
        return match Uuid::parse_str(id) {
            Ok(id) => {
                handlers::repairs::begin_add_part(
                    &bot,
                    &dialogue,
                    chat_id,
                    container,
                    session,
                    garage_domain::RepairId::from_uuid(id),
                )
                .await
            }
            Err(_) => invalid_callback(&bot, &dialogue, chat_id, session).await,
        };
    }

    if let Some(id) = data.strip_prefix("repair:part_select:") {
        return match Uuid::parse_str(id) {
            Ok(id) => {
                handlers::repairs::select_part_for_repair(
                    &bot,
                    &dialogue,
                    chat_id,
                    session,
                    garage_domain::PartId::from_uuid(id),
                )
                .await
            }
            Err(_) => invalid_callback(&bot, &dialogue, chat_id, session).await,
        };
    }

    if let Some(id) = data.strip_prefix("part:open:") {
        return match Uuid::parse_str(id) {
            Ok(id) => {
                handlers::parts::show_card(
                    &bot,
                    &dialogue,
                    chat_id,
                    container,
                    session,
                    garage_domain::PartId::from_uuid(id),
                )
                .await
            }
            Err(_) => invalid_callback(&bot, &dialogue, chat_id, session).await,
        };
    }

    if let Some(id) = data.strip_prefix("part:set_stock:") {
        return match Uuid::parse_str(id) {
            Ok(id) => {
                handlers::parts::begin_set_stock(
                    &bot,
                    &dialogue,
                    chat_id,
                    session,
                    garage_domain::PartId::from_uuid(id),
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
            send_reply_keyboard_notice(&bot, chat_id, crate::keyboards::reply::global_navigation())
                .await;
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
        "nav:cancel" => {
            let mut session = session;
            session.reset_dialog();
            send_reply_keyboard_notice(&bot, chat_id, crate::keyboards::reply::global_navigation())
                .await;
            render_screen(
                &bot,
                &dialogue,
                chat_id,
                session,
                Screen::new("Действие отменено.", crate::keyboards::main::main_menu()),
            )
            .await
        }
        "nav:clients" => handlers::clients::show_menu(&bot, &dialogue, chat_id, session).await,
        "nav:bookings" => handlers::bookings::show_menu(&bot, &dialogue, chat_id, session).await,
        "nav:stock" => handlers::parts::show_menu(&bot, &dialogue, chat_id, session).await,
        "nav:low_stock" => {
            handlers::parts::show_low_stock(&bot, &dialogue, chat_id, container, session).await
        }
        "nav:repairs" => handlers::repairs::show_menu(&bot, &dialogue, chat_id, session).await,
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
        "part:add" => handlers::parts::begin_add(&bot, &dialogue, chat_id, session).await,
        "part:confirm" => {
            handlers::parts::confirm(&bot, &dialogue, chat_id, container, session).await
        }
        "part:search" => handlers::parts::begin_search(&bot, &dialogue, chat_id, session).await,
        "part:low_stock" => {
            handlers::parts::show_low_stock(&bot, &dialogue, chat_id, container, session).await
        }
        "repair:active" => {
            handlers::repairs::show_active(&bot, &dialogue, chat_id, container, session).await
        }
        "repair:confirm_start" => {
            handlers::repairs::confirm_start(&bot, &dialogue, chat_id, container, session).await
        }
        "repair:confirm_payment" => {
            handlers::repairs::confirm_payment(&bot, &dialogue, chat_id, container, session).await
        }
        "repair:confirm_part" => {
            handlers::repairs::confirm_repair_part(&bot, &dialogue, chat_id, container, session)
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
        _ => {
            tracing::warn!(
                chat_id = chat_id.0,
                callback_prefix = data.split(':').next().unwrap_or_default(),
                "unknown callback data"
            );
            invalid_callback(&bot, &dialogue, chat_id, session).await
        }
    }
}

async fn invalid_callback(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    session: SessionData,
) -> HandlerResult {
    tracing::warn!(chat_id = chat_id.0, "invalid callback data");
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
