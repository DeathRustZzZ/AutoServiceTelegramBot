use teloxide::prelude::*;

use crate::keyboards;
use crate::messages;
use crate::state::{AddClientStep, DialogState, HandlerResult, SessionData, UserDialogue};
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

pub async fn confirm_placeholder(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    mut session: SessionData,
) -> HandlerResult {
    // Здесь будет вызов ClientService::create_client после подключения application container.
    session.reset_dialog();

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            messages::clients::saved_placeholder(),
            keyboards::clients::clients_menu(),
        ),
    )
    .await
}
