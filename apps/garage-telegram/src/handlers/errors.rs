use teloxide::prelude::*;

use crate::messages;
use crate::state::HandlerResult;

pub fn app_error_message(error: &garage_app::AppError) -> String {
    tracing::warn!(error = %error, "telegram handler app error");
    messages::errors::app_error(error)
}

pub async fn unknown_text(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, messages::errors::unknown_text())
        .await?;
    Ok(())
}
