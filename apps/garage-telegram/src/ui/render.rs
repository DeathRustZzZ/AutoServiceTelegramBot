use teloxide::prelude::*;
use teloxide::types::{ChatId, InlineKeyboardMarkup};

use crate::state::{HandlerResult, SessionData, UserDialogue};

pub struct Screen {
    pub text: String,
    pub keyboard: InlineKeyboardMarkup,
}

impl Screen {
    pub fn new(text: impl Into<String>, keyboard: InlineKeyboardMarkup) -> Self {
        Self {
            text: text.into(),
            keyboard,
        }
    }
}

pub async fn render_screen(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    mut session: SessionData,
    screen: Screen,
) -> HandlerResult {
    if let Some(message_id) = session.last_menu_msg_id {
        let edited = bot
            .edit_message_text(chat_id, message_id, screen.text.clone())
            .reply_markup(screen.keyboard.clone())
            .await;

        if edited.is_ok() {
            dialogue.update(session).await?;
            return Ok(());
        }
    }

    let message = send_menu_message(bot, chat_id, screen).await?;
    session.last_menu_msg_id = Some(message.id);
    dialogue.update(session).await?;

    Ok(())
}

pub async fn send_menu_message(
    bot: &Bot,
    chat_id: ChatId,
    screen: Screen,
) -> Result<Message, teloxide::RequestError> {
    bot.send_message(chat_id, screen.text)
        .reply_markup(screen.keyboard)
        .await
}
