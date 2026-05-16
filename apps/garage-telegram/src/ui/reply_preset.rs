use teloxide::prelude::*;
use teloxide::types::KeyboardMarkup;

pub async fn send_reply_keyboard_notice(bot: &Bot, chat_id: ChatId, keyboard: KeyboardMarkup) {
    if let Err(error) = bot
        .send_message(chat_id, "Панель обновлена.")
        .reply_markup(keyboard)
        .await
    {
        tracing::warn!(
            chat_id = chat_id.0,
            error = %error,
            "failed to send reply keyboard preset"
        );
    }
}
