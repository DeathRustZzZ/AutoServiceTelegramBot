use teloxide::prelude::*;

pub async fn delete_user_message_silent(bot: &Bot, msg: &Message) {
    if let Err(error) = bot.delete_message(msg.chat.id, msg.id).await {
        tracing::debug!(
            chat_id = msg.chat.id.0,
            message_id = msg.id.0,
            error = %error,
            "failed to delete user message"
        );
    }
}
