use teloxide::prelude::*;

use crate::keyboards;
use crate::messages;
use crate::state::{HandlerResult, SessionData, UserDialogue};
use crate::ui::render::{render_screen, Screen};

pub fn app_error_message(error: &garage_app::AppError) -> String {
    tracing::warn!(error = %error, "telegram handler app error");
    messages::errors::app_error(error)
}

pub async fn unknown_text(
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
            messages::errors::unknown_text(),
            keyboards::main::main_menu(),
        ),
    )
    .await
}
