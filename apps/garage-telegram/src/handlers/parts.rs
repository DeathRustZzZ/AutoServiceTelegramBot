use chrono::Utc;
use garage_app::AppError;
use garage_domain::{Money, Part, PartId, PartName, PartNotes, PartQuantity, PartSku};
use teloxide::prelude::*;

use crate::container::AppContainer;
use crate::keyboards;
use crate::messages;
use crate::state::{
    AddPartStep, DialogState, HandlerResult, SessionData, SetPartStockStep, UserDialogue,
};
use crate::ui::money_input::parse_byn_amount;
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
        Screen::new(messages::parts::menu(), keyboards::parts::menu()),
    )
    .await
}

pub async fn begin_add(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    mut session: SessionData,
) -> HandlerResult {
    session.part_draft.reset();
    session.dialog = DialogState::AddPart(AddPartStep::AwaitingName);

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            messages::parts::ask_name(),
            keyboards::parts::add_part_back_to_menu(),
        ),
    )
    .await
}

pub async fn handle_add_text(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    mut session: SessionData,
    step: AddPartStep,
    text: String,
) -> HandlerResult {
    let screen = match step {
        AddPartStep::AwaitingName => {
            session.part_draft.name = Some(text);
            session.dialog = DialogState::AddPart(AddPartStep::AwaitingSku);

            Screen::new(
                messages::parts::ask_sku(),
                keyboards::parts::add_part_back_to_menu(),
            )
        }
        AddPartStep::AwaitingSku => {
            session.part_draft.sku = optional_string(text);
            session.dialog = DialogState::AddPart(AddPartStep::AwaitingQuantity);

            Screen::new(
                messages::parts::ask_quantity(),
                keyboards::parts::add_part_back_to_menu(),
            )
        }
        AddPartStep::AwaitingQuantity => {
            if parse_quantity(&text).is_err() {
                return render_screen(
                    &bot,
                    &dialogue,
                    msg.chat.id,
                    session,
                    Screen::new(
                        messages::parts::invalid_quantity(),
                        keyboards::parts::add_part_back_to_menu(),
                    ),
                )
                .await;
            }

            session.part_draft.quantity = Some(text);
            session.dialog = DialogState::AddPart(AddPartStep::AwaitingMinQuantity);

            Screen::new(
                messages::parts::ask_min_quantity(),
                keyboards::parts::add_part_back_to_menu(),
            )
        }
        AddPartStep::AwaitingMinQuantity => {
            if parse_quantity(&text).is_err() {
                return render_screen(
                    &bot,
                    &dialogue,
                    msg.chat.id,
                    session,
                    Screen::new(
                        messages::parts::invalid_quantity(),
                        keyboards::parts::add_part_back_to_menu(),
                    ),
                )
                .await;
            }

            session.part_draft.min_quantity = Some(text);
            session.dialog = DialogState::AddPart(AddPartStep::AwaitingUnitPrice);

            Screen::new(
                messages::parts::ask_unit_price(),
                keyboards::parts::add_part_back_to_menu(),
            )
        }
        AddPartStep::AwaitingUnitPrice => {
            if parse_price(&text).is_err() {
                return render_screen(
                    &bot,
                    &dialogue,
                    msg.chat.id,
                    session,
                    Screen::new(
                        messages::parts::invalid_price(),
                        keyboards::parts::add_part_back_to_menu(),
                    ),
                )
                .await;
            }

            session.part_draft.unit_price = Some(text);
            session.dialog = DialogState::AddPart(AddPartStep::AwaitingNotes);

            Screen::new(
                messages::parts::ask_notes(),
                keyboards::parts::add_part_back_to_menu(),
            )
        }
        AddPartStep::AwaitingNotes => {
            session.part_draft.notes = optional_string(text);
            session.dialog = DialogState::AddPart(AddPartStep::Confirm);

            Screen::new(
                messages::parts::confirm(&session.part_draft),
                keyboards::parts::add_part_confirm(),
            )
        }
        AddPartStep::Confirm => Screen::new(
            messages::parts::confirm(&session.part_draft),
            keyboards::parts::add_part_confirm(),
        ),
    };

    render_screen(&bot, &dialogue, msg.chat.id, session, screen).await
}

pub async fn confirm(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    mut session: SessionData,
) -> HandlerResult {
    let input = match parse_part_draft(&session) {
        Ok(input) => input,
        Err(message) => {
            return render_screen(
                bot,
                dialogue,
                chat_id,
                session,
                Screen::new(message, keyboards::parts::add_part_back_to_menu()),
            )
            .await;
        }
    };

    let part = match container
        .part_service()
        .create_part(
            input.name,
            input.sku,
            input.quantity,
            input.min_quantity,
            input.unit_price,
            input.notes,
            Utc::now(),
        )
        .await
    {
        Ok(part) => part,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };

    session.reset_dialog();

    render_part_card(
        bot,
        dialogue,
        chat_id,
        session,
        messages::parts::created_card(&part),
        &part,
    )
    .await
}

pub async fn begin_search(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    mut session: SessionData,
) -> HandlerResult {
    session.dialog = DialogState::SearchPart;

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            messages::parts::ask_search_query(),
            keyboards::parts::back_to_menu(),
        ),
    )
    .await
}

pub async fn handle_search_text(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    container: AppContainer,
    mut session: SessionData,
    query: String,
) -> HandlerResult {
    let query = query.trim().to_string();

    if query.is_empty() {
        return render_screen(
            &bot,
            &dialogue,
            msg.chat.id,
            session,
            Screen::new(
                messages::parts::ask_search_query(),
                keyboards::parts::back_to_menu(),
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

    session.reset_dialog();

    let text = if parts.is_empty() {
        messages::parts::empty_search_results(&query)
    } else {
        messages::parts::search_results(&query, &parts)
    };

    render_screen(
        &bot,
        &dialogue,
        msg.chat.id,
        session,
        Screen::new(text, keyboards::parts::search_results(&parts)),
    )
    .await
}

pub async fn show_low_stock(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    mut session: SessionData,
) -> HandlerResult {
    let parts = match container.part_service().list_low_stock().await {
        Ok(parts) => parts,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };

    session.reset_dialog();

    let text = if parts.is_empty() {
        messages::parts::low_stock_empty().to_string()
    } else {
        messages::parts::low_stock(&parts)
    };

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(text, keyboards::parts::low_stock(&parts)),
    )
    .await
}

pub async fn show_card(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    mut session: SessionData,
    part_id: PartId,
) -> HandlerResult {
    let part = match container.part_service().get_part(part_id).await {
        Ok(part) => part,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };

    session.reset_dialog();

    render_part_card(
        bot,
        dialogue,
        chat_id,
        session,
        messages::parts::part_card(&part),
        &part,
    )
    .await
}

pub async fn begin_set_stock(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    mut session: SessionData,
    part_id: PartId,
) -> HandlerResult {
    session.set_part_stock_draft.part_id = Some(part_id);
    session.dialog = DialogState::SetPartStock(SetPartStockStep::AwaitingQuantity);

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            messages::parts::ask_new_stock(),
            keyboards::parts::back_to_menu(),
        ),
    )
    .await
}

pub async fn handle_set_stock_text(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    container: AppContainer,
    mut session: SessionData,
    _step: SetPartStockStep,
    text: String,
) -> HandlerResult {
    let Some(part_id) = session.set_part_stock_draft.part_id else {
        session.reset_dialog();
        return render_screen(
            &bot,
            &dialogue,
            msg.chat.id,
            session,
            Screen::new(
                messages::errors::invalid_callback(),
                keyboards::parts::back_to_menu(),
            ),
        )
        .await;
    };

    let quantity = match parse_quantity(&text) {
        Ok(quantity) => quantity,
        Err(_) => {
            return render_screen(
                &bot,
                &dialogue,
                msg.chat.id,
                session,
                Screen::new(
                    messages::parts::invalid_quantity(),
                    keyboards::parts::back_to_menu(),
                ),
            )
            .await;
        }
    };

    let part = match container
        .part_service()
        .set_stock(part_id, quantity, Utc::now())
        .await
    {
        Ok(part) => part,
        Err(error) => {
            return render_app_error(&bot, &dialogue, msg.chat.id, session, &error).await;
        }
    };

    session.reset_dialog();

    render_part_card(
        &bot,
        &dialogue,
        msg.chat.id,
        session,
        messages::parts::part_card(&part),
        &part,
    )
    .await
}

struct ParsedPart {
    name: PartName,
    sku: Option<PartSku>,
    quantity: PartQuantity,
    min_quantity: PartQuantity,
    unit_price: Money,
    notes: Option<PartNotes>,
}

fn parse_part_draft(session: &SessionData) -> Result<ParsedPart, String> {
    let Some(name) = session.part_draft.name.as_deref() else {
        return Err(messages::parts::missing_required_fields().to_string());
    };
    let Some(quantity) = session.part_draft.quantity.as_deref() else {
        return Err(messages::parts::missing_required_fields().to_string());
    };
    let Some(min_quantity) = session.part_draft.min_quantity.as_deref() else {
        return Err(messages::parts::missing_required_fields().to_string());
    };
    let Some(unit_price) = session.part_draft.unit_price.as_deref() else {
        return Err(messages::parts::missing_required_fields().to_string());
    };

    let name = PartName::parse(name).map_err(|error| part_error_message(error.into()))?;
    let sku = PartSku::parse(session.part_draft.sku.as_deref().unwrap_or(""))
        .map_err(|error| part_error_message(error.into()))?;
    let quantity =
        parse_quantity(quantity).map_err(|_| messages::parts::invalid_quantity().to_string())?;
    let min_quantity = parse_quantity(min_quantity)
        .map_err(|_| messages::parts::invalid_quantity().to_string())?;
    let unit_price =
        parse_price(unit_price).map_err(|_| messages::parts::invalid_price().to_string())?;
    let notes = match session.part_draft.notes.as_deref() {
        Some(notes) => PartNotes::parse(notes).map_err(|error| part_error_message(error.into()))?,
        None => None,
    };

    Ok(ParsedPart {
        name,
        sku,
        quantity,
        min_quantity,
        unit_price,
        notes,
    })
}

fn parse_quantity(input: &str) -> Result<PartQuantity, ()> {
    input
        .trim()
        .parse::<u32>()
        .map(PartQuantity::new)
        .map_err(|_| ())
}

fn parse_price(input: &str) -> Result<Money, ()> {
    parse_byn_amount(input).map_err(|_| ())
}

fn optional_string(input: String) -> Option<String> {
    let value = input.trim();
    (!value.is_empty() && value != "-").then(|| value.to_string())
}

fn part_error_message(error: AppError) -> String {
    crate::handlers::errors::app_error_message(&error)
}

async fn render_part_card(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    session: SessionData,
    text: String,
    part: &Part,
) -> HandlerResult {
    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(text, keyboards::parts::part_card(part)),
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
            keyboards::parts::menu(),
        ),
    )
    .await
}
