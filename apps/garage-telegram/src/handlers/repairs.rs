use chrono::Utc;
use garage_app::{AppError, BookingDetails, StartRepairCommand};
use garage_domain::{BookingId, Currency, Money, RepairDescription, RepairId, RepairNotes};
use teloxide::prelude::*;

use crate::container::AppContainer;
use crate::keyboards;
use crate::messages;
use crate::state::{DialogState, HandlerResult, SessionData, StartRepairStep, UserDialogue};
use crate::ui::render::{render_screen, Screen};

pub async fn show_menu(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    mut session: SessionData,
) -> HandlerResult {
    session.reset_dialog();

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(messages::repairs::menu(), keyboards::repairs::menu()),
    )
    .await
}

pub async fn show_active(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    mut session: SessionData,
) -> HandlerResult {
    let items = match container
        .repair_query_service()
        .list_active_repair_details()
        .await
    {
        Ok(items) => items,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };

    session.reset_dialog();

    let screen = if items.is_empty() {
        Screen::new(
            messages::repairs::active_empty(),
            keyboards::repairs::active_empty(),
        )
    } else {
        Screen::new(
            messages::repairs::active_list(&items),
            keyboards::repairs::active_list(&items),
        )
    };

    render_screen(bot, dialogue, chat_id, session, screen).await
}

pub async fn begin_start_from_booking(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    mut session: SessionData,
    booking_id: BookingId,
) -> HandlerResult {
    let details = match container
        .booking_service()
        .get_booking_details(booking_id)
        .await
    {
        Ok(details) => details,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };

    if !details.booking.is_scheduled() {
        return render_screen(
            bot,
            dialogue,
            chat_id,
            session,
            Screen::new(
                "Ремонт можно начать только из запланированной записи.",
                keyboards::repairs::back_to_booking(booking_id),
            ),
        )
        .await;
    }

    session.start_repair_draft.reset();
    session.start_repair_draft.booking_id = Some(booking_id);
    session.dialog = DialogState::StartRepair(StartRepairStep::AwaitingDescription);

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            messages::repairs::ask_description(),
            keyboards::repairs::back_to_booking(details.booking.id()),
        ),
    )
    .await
}

pub async fn handle_start_text(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    container: AppContainer,
    mut session: SessionData,
    step: StartRepairStep,
    text: String,
) -> HandlerResult {
    match step {
        StartRepairStep::AwaitingDescription => {
            session.start_repair_draft.description = Some(text);
            session.dialog = DialogState::StartRepair(StartRepairStep::AwaitingNotes);
            let keyboard = start_back_keyboard(&session.start_repair_draft.booking_id);

            render_screen(
                &bot,
                &dialogue,
                msg.chat.id,
                session,
                Screen::new(messages::repairs::ask_notes(), keyboard),
            )
            .await
        }
        StartRepairStep::AwaitingNotes => {
            session.start_repair_draft.notes = optional_string(text);
            session.dialog = DialogState::StartRepair(StartRepairStep::Confirm);

            let details = match load_draft_booking(&container, &session).await {
                Ok(Some(details)) => details,
                Ok(None) => {
                    return render_missing_draft(&bot, &dialogue, msg.chat.id, session).await
                }
                Err(error) => {
                    return render_app_error(&bot, &dialogue, msg.chat.id, session, &error).await;
                }
            };

            render_screen(
                &bot,
                &dialogue,
                msg.chat.id,
                session.clone(),
                Screen::new(
                    messages::repairs::confirm_start(
                        &details,
                        &session.start_repair_draft,
                        container.timezone_offset_hours(),
                    ),
                    keyboards::repairs::start_confirm(details.booking.id()),
                ),
            )
            .await
        }
        StartRepairStep::Confirm => {
            let details = match load_draft_booking(&container, &session).await {
                Ok(Some(details)) => details,
                Ok(None) => {
                    return render_missing_draft(&bot, &dialogue, msg.chat.id, session).await
                }
                Err(error) => {
                    return render_app_error(&bot, &dialogue, msg.chat.id, session, &error).await;
                }
            };

            render_screen(
                &bot,
                &dialogue,
                msg.chat.id,
                session.clone(),
                Screen::new(
                    messages::repairs::confirm_start(
                        &details,
                        &session.start_repair_draft,
                        container.timezone_offset_hours(),
                    ),
                    keyboards::repairs::start_confirm(details.booking.id()),
                ),
            )
            .await
        }
    }
}

pub async fn confirm_start(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    mut session: SessionData,
) -> HandlerResult {
    let details = match load_draft_booking(&container, &session).await {
        Ok(Some(details)) => details,
        Ok(None) => return render_missing_draft(bot, dialogue, chat_id, session).await,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };

    let input = match parse_start_draft(&session, &details) {
        Ok(input) => input,
        Err(error) => {
            return render_screen(
                bot,
                dialogue,
                chat_id,
                session,
                Screen::new(
                    crate::handlers::errors::app_error_message(&error),
                    keyboards::repairs::start_confirm(details.booking.id()),
                ),
            )
            .await;
        }
    };

    let repair = match container.repair_service().start_repair(input).await {
        Ok(repair) => repair,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };

    let details = match container
        .repair_query_service()
        .get_repair_details(repair.id())
        .await
    {
        Ok(details) => details,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };

    session.reset_dialog();

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            messages::repairs::repair_created_card(&details),
            keyboards::repairs::repair_card(&details.repair),
        ),
    )
    .await
}

pub async fn show_card(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    mut session: SessionData,
    repair_id: RepairId,
) -> HandlerResult {
    let details = match container
        .repair_query_service()
        .get_repair_details(repair_id)
        .await
    {
        Ok(details) => details,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };

    session.reset_dialog();

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            messages::repairs::repair_card(&details),
            keyboards::repairs::repair_card(&details.repair),
        ),
    )
    .await
}

pub async fn complete(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    session: SessionData,
    repair_id: RepairId,
) -> HandlerResult {
    let repair = match container
        .repair_service()
        .complete_repair(repair_id, Utc::now())
        .await
    {
        Ok(repair) => repair,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };

    show_card(bot, dialogue, chat_id, container, session, repair.id()).await
}

pub async fn cancel(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    session: SessionData,
    repair_id: RepairId,
) -> HandlerResult {
    let repair = match container
        .repair_service()
        .cancel_repair(repair_id, Utc::now())
        .await
    {
        Ok(repair) => repair,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };

    show_card(bot, dialogue, chat_id, container, session, repair.id()).await
}

fn parse_start_draft(
    session: &SessionData,
    details: &BookingDetails,
) -> Result<StartRepairCommand, AppError> {
    let Some(description) = session.start_repair_draft.description.as_deref() else {
        return Err(AppError::Repository {
            operation: "start repair draft",
            message: messages::repairs::missing_draft().to_string(),
        });
    };

    let description = RepairDescription::parse(description)?;
    let notes = match session.start_repair_draft.notes.as_deref() {
        Some(notes) => RepairNotes::parse(notes)?,
        None => None,
    };
    let zero = Money::zero(Currency::Byn);

    Ok(StartRepairCommand {
        client_id: details.booking.client_id(),
        car_id: details.booking.car_id(),
        booking_id: Some(details.booking.id()),
        description,
        labor_price: zero,
        parts_price: zero,
        parts_cost: zero,
        notes,
        now: Utc::now(),
    })
}

async fn load_draft_booking(
    container: &AppContainer,
    session: &SessionData,
) -> Result<Option<BookingDetails>, AppError> {
    let Some(booking_id) = session.start_repair_draft.booking_id else {
        return Ok(None);
    };

    container
        .booking_service()
        .get_booking_details(booking_id)
        .await
        .map(Some)
}

async fn render_missing_draft(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    mut session: SessionData,
) -> HandlerResult {
    session.reset_dialog();

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            messages::repairs::missing_draft(),
            keyboards::repairs::back_to_menu(),
        ),
    )
    .await
}

async fn render_app_error(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    session: SessionData,
    error: &AppError,
) -> HandlerResult {
    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            crate::handlers::errors::app_error_message(error),
            keyboards::repairs::menu(),
        ),
    )
    .await
}

fn optional_string(input: String) -> Option<String> {
    let value = input.trim();
    (!value.is_empty() && value != "-").then(|| value.to_string())
}

fn start_back_keyboard(booking_id: &Option<BookingId>) -> teloxide::types::InlineKeyboardMarkup {
    match booking_id {
        Some(booking_id) => keyboards::repairs::back_to_booking(*booking_id),
        None => keyboards::repairs::back_to_menu(),
    }
}
