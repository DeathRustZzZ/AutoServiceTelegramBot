use chrono::Utc;
use garage_app::AppError;
use garage_domain::{ClientId, ClientName, ClientNotes, PhoneNumber};
use teloxide::prelude::*;

use crate::container::AppContainer;
use crate::keyboards;
use crate::messages;
use crate::state::{AddClientStep, DialogState, HandlerResult, SessionData, UserDialogue};
use crate::ui::render::{render_screen, Screen};

const PAGE_SIZE: u32 = 5;

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
        Screen::new(
            messages::clients::menu(),
            keyboards::clients::clients_menu(),
        ),
    )
    .await
}

pub async fn begin_add(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    mut session: SessionData,
) -> HandlerResult {
    session.client_draft.reset();
    session.dialog = DialogState::AddClient(AddClientStep::AwaitingName);

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            messages::clients::ask_name(),
            keyboards::clients::add_client_back_to_clients(),
        ),
    )
    .await
}

pub async fn show_list(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    session: SessionData,
    page: usize,
) -> HandlerResult {
    let offset = page.saturating_mul(PAGE_SIZE as usize) as u32;
    let clients = match container
        .client_service()
        .list_clients(PAGE_SIZE, offset)
        .await
    {
        Ok(clients) => clients,
        Err(error) => {
            return render_app_error(bot, dialogue, chat_id, session, &error).await;
        }
    };

    let screen = if clients.is_empty() && page == 0 {
        Screen::new(
            messages::clients::empty_list(),
            keyboards::clients::empty_clients(),
        )
    } else {
        Screen::new(
            messages::clients::list_page(&clients, page),
            keyboards::clients::clients_list(&clients, page, clients.len() == PAGE_SIZE as usize),
        )
    };

    render_screen(bot, dialogue, chat_id, session, screen).await
}

pub async fn begin_search(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    mut session: SessionData,
) -> HandlerResult {
    session.dialog = DialogState::SearchClient;

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            messages::clients::ask_search_query(),
            keyboards::clients::add_client_back_to_clients(),
        ),
    )
    .await
}

pub async fn handle_add_text(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    mut session: SessionData,
    step: AddClientStep,
    text: String,
) -> HandlerResult {
    let screen = match step {
        AddClientStep::AwaitingName => {
            session.client_draft.name = Some(text);
            session.dialog = DialogState::AddClient(AddClientStep::AwaitingPhone);

            Screen::new(
                messages::clients::ask_phone(),
                keyboards::clients::add_client_back_to_clients(),
            )
        }
        AddClientStep::AwaitingPhone => {
            session.client_draft.phone = Some(text);
            session.dialog = DialogState::AddClient(AddClientStep::AwaitingNotes);

            Screen::new(
                messages::clients::ask_notes(),
                keyboards::clients::add_client_back_to_clients(),
            )
        }
        AddClientStep::AwaitingNotes => {
            let notes = text.trim();
            session.client_draft.notes =
                (!notes.is_empty() && notes != "-").then(|| notes.to_string());
            session.dialog = DialogState::AddClient(AddClientStep::Confirm);

            Screen::new(
                messages::clients::confirm(&session.client_draft),
                keyboards::clients::add_client_confirm(),
            )
        }
        AddClientStep::Confirm => Screen::new(
            messages::clients::confirm(&session.client_draft),
            keyboards::clients::add_client_confirm(),
        ),
    };

    render_screen(&bot, &dialogue, msg.chat.id, session, screen).await
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
                messages::clients::ask_search_query(),
                keyboards::clients::add_client_back_to_clients(),
            ),
        )
        .await;
    }

    let clients = match container
        .client_service()
        .search_clients(&query, PAGE_SIZE, 0)
        .await
    {
        Ok(clients) => clients,
        Err(error) => {
            return render_app_error(&bot, &dialogue, msg.chat.id, session, &error).await;
        }
    };

    session.reset_dialog();

    let text = if clients.is_empty() {
        messages::clients::empty_search_results(&query)
    } else {
        messages::clients::search_results(&query, &clients)
    };

    render_screen(
        &bot,
        &dialogue,
        msg.chat.id,
        session,
        Screen::new(text, keyboards::clients::search_results(&clients)),
    )
    .await
}

pub async fn show_card(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    mut session: SessionData,
    client_id: ClientId,
) -> HandlerResult {
    let client = match container.client_service().get_client(client_id).await {
        Ok(client) => client,
        Err(error) => {
            return render_app_error(bot, dialogue, chat_id, session, &error).await;
        }
    };

    session.reset_dialog();

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            messages::clients::client_card(&client, "Клиент"),
            keyboards::clients::client_card(&client),
        ),
    )
    .await
}

pub async fn confirm(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    mut session: SessionData,
) -> HandlerResult {
    let Some(name) = session.client_draft.name.as_deref() else {
        return render_screen(
            bot,
            dialogue,
            chat_id,
            session,
            Screen::new(
                messages::errors::missing_client_name(),
                keyboards::clients::add_client_back_to_clients(),
            ),
        )
        .await;
    };
    let Some(phone) = session.client_draft.phone.as_deref() else {
        return render_screen(
            bot,
            dialogue,
            chat_id,
            session,
            Screen::new(
                messages::errors::missing_client_phone(),
                keyboards::clients::add_client_back_to_clients(),
            ),
        )
        .await;
    };

    let result = create_client(
        container,
        name,
        phone,
        session.client_draft.notes.as_deref(),
    )
    .await;

    let client = match result {
        Ok(client) => client,
        Err(error) => {
            return render_screen(
                bot,
                dialogue,
                chat_id,
                session,
                Screen::new(
                    crate::handlers::errors::app_error_message(&error),
                    keyboards::clients::add_client_back_to_clients(),
                ),
            )
            .await;
        }
    };

    session.reset_dialog();

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            messages::clients::created_card(&client),
            keyboards::clients::client_card(&client),
        ),
    )
    .await
}

async fn create_client(
    container: AppContainer,
    name: &str,
    phone: &str,
    notes: Option<&str>,
) -> Result<garage_domain::Client, AppError> {
    let name = ClientName::parse(name)?;
    let phone = PhoneNumber::parse(phone)?;
    let notes = match notes {
        Some(notes) => ClientNotes::parse(notes)?,
        None => None,
    };

    container
        .client_service()
        .create_client(name, phone, notes, Utc::now())
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
            keyboards::clients::clients_menu(),
        ),
    )
    .await
}
