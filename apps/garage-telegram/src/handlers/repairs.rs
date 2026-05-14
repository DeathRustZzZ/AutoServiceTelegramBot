use chrono::Utc;
use garage_app::{
    AppError, BookingDetails, RecordPaymentCommand, StartRepairCommand, UsePartInRepairCommand,
    UsePartInRepairResult,
};
use garage_domain::{
    BookingId, Currency, Money, Part, PartId, PartQuantity, PaymentComment, PaymentMethod,
    RepairDescription, RepairId, RepairNotes, StockMovementComment,
};
use teloxide::prelude::*;

use crate::container::AppContainer;
use crate::keyboards;
use crate::messages;
use crate::state::{
    DialogState, HandlerResult, RecordPaymentStep, SessionData, SetRepairLaborStep,
    StartRepairStep, UseRepairPartStep, UserDialogue,
};
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

pub async fn begin_payment(
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

    if !details.repair.is_in_progress() {
        return render_screen(
            bot,
            dialogue,
            chat_id,
            session,
            Screen::new(
                "Оплату можно принять только по активному ремонту.",
                keyboards::repairs::back_to_repair(repair_id),
            ),
        )
        .await;
    }

    session.record_payment_draft.reset();
    session.record_payment_draft.repair_id = Some(repair_id);
    session.dialog = DialogState::RecordPayment(RecordPaymentStep::AwaitingAmount);

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            messages::repairs::ask_payment_amount(),
            keyboards::repairs::back_to_repair(repair_id),
        ),
    )
    .await
}

pub async fn begin_set_labor(
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

    if !details.repair.is_in_progress() {
        return render_screen(
            bot,
            dialogue,
            chat_id,
            session,
            Screen::new(
                "Стоимость работ можно менять только в активном ремонте.",
                keyboards::repairs::back_to_repair(repair_id),
            ),
        )
        .await;
    }

    session.set_repair_labor_draft.repair_id = Some(repair_id);
    session.dialog = DialogState::SetRepairLabor(SetRepairLaborStep::AwaitingAmount);

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            messages::repairs::ask_labor_price(),
            keyboards::repairs::back_to_repair(repair_id),
        ),
    )
    .await
}

pub async fn handle_set_labor_text(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    container: AppContainer,
    mut session: SessionData,
    _step: SetRepairLaborStep,
    text: String,
) -> HandlerResult {
    let Some(repair_id) = session.set_repair_labor_draft.repair_id else {
        return render_missing_draft(&bot, &dialogue, msg.chat.id, session).await;
    };

    let labor_price = match parse_money(&text) {
        Ok(value) => value,
        Err(_) => {
            return render_screen(
                &bot,
                &dialogue,
                msg.chat.id,
                session,
                Screen::new(
                    messages::repairs::invalid_money(),
                    keyboards::repairs::back_to_repair(repair_id),
                ),
            )
            .await;
        }
    };

    let repair = match container
        .repair_service()
        .set_labor_price(repair_id, labor_price, Utc::now())
        .await
    {
        Ok(repair) => repair,
        Err(error) => {
            return render_app_error(&bot, &dialogue, msg.chat.id, session, &error).await;
        }
    };

    let details = match container
        .repair_query_service()
        .get_repair_details(repair.id())
        .await
    {
        Ok(details) => details,
        Err(error) => {
            return render_app_error(&bot, &dialogue, msg.chat.id, session, &error).await;
        }
    };

    session.reset_dialog();

    render_screen(
        &bot,
        &dialogue,
        msg.chat.id,
        session,
        Screen::new(
            messages::repairs::labor_price_updated_card(&details),
            keyboards::repairs::repair_card(&details.repair),
        ),
    )
    .await
}

pub async fn handle_payment_text(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    container: AppContainer,
    mut session: SessionData,
    step: RecordPaymentStep,
    text: String,
) -> HandlerResult {
    match step {
        RecordPaymentStep::AwaitingAmount => {
            if parse_money(&text).is_err() {
                let keyboard = payment_back_keyboard(&session.record_payment_draft.repair_id);
                return render_screen(
                    &bot,
                    &dialogue,
                    msg.chat.id,
                    session,
                    Screen::new(messages::repairs::invalid_money(), keyboard),
                )
                .await;
            }

            session.record_payment_draft.amount = Some(text);
            session.dialog = DialogState::RecordPayment(RecordPaymentStep::AwaitingMethod);
            let keyboard = payment_back_keyboard(&session.record_payment_draft.repair_id);

            render_screen(
                &bot,
                &dialogue,
                msg.chat.id,
                session,
                Screen::new(messages::repairs::ask_payment_method(), keyboard),
            )
            .await
        }
        RecordPaymentStep::AwaitingMethod => {
            if parse_payment_method(&text).is_err() {
                let keyboard = payment_back_keyboard(&session.record_payment_draft.repair_id);
                return render_screen(
                    &bot,
                    &dialogue,
                    msg.chat.id,
                    session,
                    Screen::new(messages::repairs::ask_payment_method(), keyboard),
                )
                .await;
            }

            session.record_payment_draft.method = Some(text.trim().to_string());
            session.dialog = DialogState::RecordPayment(RecordPaymentStep::AwaitingComment);
            let keyboard = payment_back_keyboard(&session.record_payment_draft.repair_id);

            render_screen(
                &bot,
                &dialogue,
                msg.chat.id,
                session,
                Screen::new(messages::repairs::ask_payment_comment(), keyboard),
            )
            .await
        }
        RecordPaymentStep::AwaitingComment => {
            session.record_payment_draft.comment = optional_string(text);
            session.dialog = DialogState::RecordPayment(RecordPaymentStep::Confirm);

            let details = match load_payment_repair(&container, &session).await {
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
                    messages::repairs::confirm_payment(&details, &session.record_payment_draft),
                    keyboards::repairs::payment_confirm(details.repair.id()),
                ),
            )
            .await
        }
        RecordPaymentStep::Confirm => {
            let details = match load_payment_repair(&container, &session).await {
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
                    messages::repairs::confirm_payment(&details, &session.record_payment_draft),
                    keyboards::repairs::payment_confirm(details.repair.id()),
                ),
            )
            .await
        }
    }
}

pub async fn confirm_payment(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    mut session: SessionData,
) -> HandlerResult {
    let command = match parse_payment_draft(&session) {
        Ok(command) => command,
        Err(error) => {
            return render_screen(
                bot,
                dialogue,
                chat_id,
                session,
                Screen::new(
                    crate::handlers::errors::app_error_message(&error),
                    keyboards::repairs::back_to_menu(),
                ),
            )
            .await;
        }
    };
    let repair_id = command.repair_id;

    if let Err(error) = container.record_payment(command).await {
        return render_app_error(bot, dialogue, chat_id, session, &error).await;
    }

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
            messages::repairs::payment_recorded_card(&details),
            keyboards::repairs::repair_card(&details.repair),
        ),
    )
    .await
}

pub async fn begin_add_part(
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

    if !details.repair.is_in_progress() {
        return render_screen(
            bot,
            dialogue,
            chat_id,
            session,
            Screen::new(
                "Запчасти можно добавлять только в активный ремонт.",
                keyboards::repairs::back_to_repair(repair_id),
            ),
        )
        .await;
    }

    session.use_repair_part_draft.reset();
    session.use_repair_part_draft.repair_id = Some(repair_id);
    session.dialog = DialogState::UseRepairPart(UseRepairPartStep::AwaitingPartSearch);

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            messages::repairs::ask_repair_part_query(),
            keyboards::repairs::back_to_repair(repair_id),
        ),
    )
    .await
}

pub async fn handle_repair_part_text(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    container: AppContainer,
    mut session: SessionData,
    step: UseRepairPartStep,
    text: String,
) -> HandlerResult {
    match step {
        UseRepairPartStep::AwaitingPartSearch => {
            let Some(repair_id) = session.use_repair_part_draft.repair_id else {
                return render_missing_draft(&bot, &dialogue, msg.chat.id, session).await;
            };
            let query = text.trim().to_string();
            if query.is_empty() {
                return render_screen(
                    &bot,
                    &dialogue,
                    msg.chat.id,
                    session,
                    Screen::new(
                        messages::repairs::ask_repair_part_query(),
                        keyboards::repairs::back_to_repair(repair_id),
                    ),
                )
                .await;
            }

            let parts = match container.part_service().search_parts(&query).await {
                Ok(parts) => parts,
                Err(error) => {
                    return render_app_error(&bot, &dialogue, msg.chat.id, session, &error).await;
                }
            };

            let text = if parts.is_empty() {
                messages::repairs::no_repair_part_results(&query)
            } else {
                messages::repairs::repair_part_search_results(&query, &parts)
            };
            session.dialog = DialogState::UseRepairPart(UseRepairPartStep::AwaitingPartSelection);

            render_screen(
                &bot,
                &dialogue,
                msg.chat.id,
                session,
                Screen::new(
                    text,
                    keyboards::repairs::repair_part_search_results(&parts, repair_id),
                ),
            )
            .await
        }
        UseRepairPartStep::AwaitingPartSelection => {
            let keyboard = repair_part_back_keyboard(&session.use_repair_part_draft.repair_id);
            render_screen(
                &bot,
                &dialogue,
                msg.chat.id,
                session,
                Screen::new("Выберите запчасть кнопкой.", keyboard),
            )
            .await
        }
        UseRepairPartStep::AwaitingQuantity => {
            if parse_positive_quantity(&text).is_err() {
                let keyboard = repair_part_back_keyboard(&session.use_repair_part_draft.repair_id);
                return render_screen(
                    &bot,
                    &dialogue,
                    msg.chat.id,
                    session,
                    Screen::new(messages::repairs::invalid_quantity(), keyboard),
                )
                .await;
            }

            session.use_repair_part_draft.quantity = Some(text);
            session.dialog = DialogState::UseRepairPart(UseRepairPartStep::AwaitingUnitPrice);
            let keyboard = repair_part_back_keyboard(&session.use_repair_part_draft.repair_id);

            render_screen(
                &bot,
                &dialogue,
                msg.chat.id,
                session,
                Screen::new(messages::repairs::ask_repair_part_unit_price(), keyboard),
            )
            .await
        }
        UseRepairPartStep::AwaitingUnitPrice => {
            if parse_money(&text).is_err() {
                let keyboard = repair_part_back_keyboard(&session.use_repair_part_draft.repair_id);
                return render_screen(
                    &bot,
                    &dialogue,
                    msg.chat.id,
                    session,
                    Screen::new(messages::repairs::invalid_money(), keyboard),
                )
                .await;
            }

            session.use_repair_part_draft.unit_price = Some(text);
            session.dialog = DialogState::UseRepairPart(UseRepairPartStep::AwaitingComment);
            let keyboard = repair_part_back_keyboard(&session.use_repair_part_draft.repair_id);

            render_screen(
                &bot,
                &dialogue,
                msg.chat.id,
                session,
                Screen::new(messages::repairs::ask_repair_part_comment(), keyboard),
            )
            .await
        }
        UseRepairPartStep::AwaitingComment => {
            session.use_repair_part_draft.comment = optional_string(text);
            session.dialog = DialogState::UseRepairPart(UseRepairPartStep::Confirm);

            let (details, part) = match load_repair_part_draft(&container, &session).await {
                Ok(Some(value)) => value,
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
                    messages::repairs::confirm_repair_part(
                        &details,
                        &part,
                        &session.use_repair_part_draft,
                    ),
                    keyboards::repairs::repair_part_confirm(details.repair.id()),
                ),
            )
            .await
        }
        UseRepairPartStep::Confirm => {
            let (details, part) = match load_repair_part_draft(&container, &session).await {
                Ok(Some(value)) => value,
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
                    messages::repairs::confirm_repair_part(
                        &details,
                        &part,
                        &session.use_repair_part_draft,
                    ),
                    keyboards::repairs::repair_part_confirm(details.repair.id()),
                ),
            )
            .await
        }
    }
}

pub async fn select_part_for_repair(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    mut session: SessionData,
    part_id: PartId,
) -> HandlerResult {
    if session.use_repair_part_draft.repair_id.is_none() {
        return render_missing_draft(bot, dialogue, chat_id, session).await;
    }

    session.use_repair_part_draft.part_id = Some(part_id);
    session.dialog = DialogState::UseRepairPart(UseRepairPartStep::AwaitingQuantity);
    let keyboard = repair_part_back_keyboard(&session.use_repair_part_draft.repair_id);

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(messages::repairs::ask_repair_part_quantity(), keyboard),
    )
    .await
}

pub async fn confirm_repair_part(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    mut session: SessionData,
) -> HandlerResult {
    let command = match parse_repair_part_draft(&container, &session).await {
        Ok(command) => command,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };
    let repair_id = command.repair_id;

    let result = match container.use_part_in_repair(command).await {
        Ok(result) => result,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };

    let details = match container
        .repair_query_service()
        .get_repair_details(repair_id)
        .await
    {
        Ok(details) => details,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };

    session.reset_dialog();
    let warning = stock_warning(&result);

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            messages::repairs::repair_part_added_card(&details, warning.as_deref()),
            keyboards::repairs::repair_card(&details.repair),
        ),
    )
    .await
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

fn parse_payment_draft(session: &SessionData) -> Result<RecordPaymentCommand, AppError> {
    let Some(repair_id) = session.record_payment_draft.repair_id else {
        return Err(draft_error());
    };
    let Some(amount) = session.record_payment_draft.amount.as_deref() else {
        return Err(draft_error());
    };
    let Some(method) = session.record_payment_draft.method.as_deref() else {
        return Err(draft_error());
    };

    let amount = parse_money(amount).map_err(|_| money_input_error())?;
    let method = parse_payment_method(method).map_err(|_| money_input_error())?;
    let comment = match session.record_payment_draft.comment.as_deref() {
        Some(comment) => PaymentComment::parse(comment)?,
        None => None,
    };
    let now = Utc::now();

    Ok(RecordPaymentCommand {
        repair_id,
        amount,
        method,
        comment,
        paid_at: now,
        now,
    })
}

async fn parse_repair_part_draft(
    container: &AppContainer,
    session: &SessionData,
) -> Result<UsePartInRepairCommand, AppError> {
    let Some(repair_id) = session.use_repair_part_draft.repair_id else {
        return Err(draft_error());
    };
    let Some(part_id) = session.use_repair_part_draft.part_id else {
        return Err(draft_error());
    };
    let Some(quantity) = session.use_repair_part_draft.quantity.as_deref() else {
        return Err(draft_error());
    };
    let Some(unit_price) = session.use_repair_part_draft.unit_price.as_deref() else {
        return Err(draft_error());
    };

    let part = container.part_service().get_part(part_id).await?;
    let quantity = parse_positive_quantity(quantity).map_err(|_| quantity_input_error())?;
    let unit_price = parse_money(unit_price).map_err(|_| money_input_error())?;
    let comment = match session.use_repair_part_draft.comment.as_deref() {
        Some(comment) => StockMovementComment::parse(comment)?,
        None => None,
    };
    let now = Utc::now();

    Ok(UsePartInRepairCommand {
        repair_id,
        part_id,
        quantity,
        unit_cost: part.unit_price(),
        unit_price,
        comment,
        occurred_at: now,
        now,
    })
}

async fn load_payment_repair(
    container: &AppContainer,
    session: &SessionData,
) -> Result<Option<garage_app::RepairDetails>, AppError> {
    let Some(repair_id) = session.record_payment_draft.repair_id else {
        return Ok(None);
    };

    container
        .repair_query_service()
        .get_repair_details(repair_id)
        .await
        .map(Some)
}

async fn load_repair_part_draft(
    container: &AppContainer,
    session: &SessionData,
) -> Result<Option<(garage_app::RepairDetails, Part)>, AppError> {
    let Some(repair_id) = session.use_repair_part_draft.repair_id else {
        return Ok(None);
    };
    let Some(part_id) = session.use_repair_part_draft.part_id else {
        return Ok(None);
    };

    let details = container
        .repair_query_service()
        .get_repair_details(repair_id)
        .await?;
    let part = container.part_service().get_part(part_id).await?;

    Ok(Some((details, part)))
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

fn parse_money(input: &str) -> Result<Money, ()> {
    let value = input.trim().parse::<i64>().map_err(|_| ())?;
    Money::byn_minor(value).map_err(|_| ())
}

fn parse_positive_quantity(input: &str) -> Result<PartQuantity, ()> {
    let value = input.trim().parse::<u32>().map_err(|_| ())?;
    if value == 0 {
        return Err(());
    }
    Ok(PartQuantity::new(value))
}

fn parse_payment_method(input: &str) -> Result<PaymentMethod, ()> {
    match input.trim().to_lowercase().as_str() {
        "cash" | "наличные" => Ok(PaymentMethod::Cash),
        "card" | "карта" => Ok(PaymentMethod::Card),
        "transfer" | "bank_transfer" | "перевод" => Ok(PaymentMethod::BankTransfer),
        "crypto" => Ok(PaymentMethod::Crypto),
        "other" | "другое" => Ok(PaymentMethod::Other),
        _ => Err(()),
    }
}

fn stock_warning(result: &UsePartInRepairResult) -> Option<String> {
    if result.is_out_of_stock {
        Some(format!(
            "⚠️ После списания запчасть закончилась: {} шт.",
            result.part.quantity().value()
        ))
    } else if result.is_low_stock {
        Some(format!(
            "⚠️ После списания низкий остаток: {} шт.",
            result.part.quantity().value()
        ))
    } else {
        None
    }
}

fn payment_back_keyboard(repair_id: &Option<RepairId>) -> teloxide::types::InlineKeyboardMarkup {
    match repair_id {
        Some(repair_id) => keyboards::repairs::back_to_repair(*repair_id),
        None => keyboards::repairs::back_to_menu(),
    }
}

fn repair_part_back_keyboard(
    repair_id: &Option<RepairId>,
) -> teloxide::types::InlineKeyboardMarkup {
    match repair_id {
        Some(repair_id) => keyboards::repairs::back_to_repair(*repair_id),
        None => keyboards::repairs::back_to_menu(),
    }
}

fn start_back_keyboard(booking_id: &Option<BookingId>) -> teloxide::types::InlineKeyboardMarkup {
    match booking_id {
        Some(booking_id) => keyboards::repairs::back_to_booking(*booking_id),
        None => keyboards::repairs::back_to_menu(),
    }
}

fn draft_error() -> AppError {
    AppError::Repository {
        operation: "repair telegram draft",
        message: messages::repairs::missing_draft().to_string(),
    }
}

fn money_input_error() -> AppError {
    AppError::Repository {
        operation: "repair money input",
        message: messages::repairs::invalid_money().to_string(),
    }
}

fn quantity_input_error() -> AppError {
    AppError::Repository {
        operation: "repair quantity input",
        message: messages::repairs::invalid_quantity().to_string(),
    }
}
