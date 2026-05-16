use teloxide::prelude::*;
use teloxide::types::KeyboardMarkup;

pub async fn set_reply_keyboard_silent(bot: &Bot, chat_id: ChatId, keyboard: KeyboardMarkup) {
    match bot
        .send_message(chat_id, "\u{2060}")
        .reply_markup(keyboard)
        .await
    {
        Ok(message) => {
            if let Err(error) = bot.delete_message(chat_id, message.id).await {
                tracing::debug!(
                    chat_id = chat_id.0,
                    message_id = message.id.0,
                    error = %error,
                    "failed to delete reply keyboard service message"
                );
            }
        }
        Err(error) => {
            tracing::warn!(
                chat_id = chat_id.0,
                error = %error,
                "failed to send reply keyboard service message"
            );
        }
    }
}
