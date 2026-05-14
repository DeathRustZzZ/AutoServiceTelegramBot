use teloxide::prelude::*;

use crate::messages;
use crate::state::HandlerResult;

pub async fn unknown_text(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, messages::errors::unknown_text())
        .await?;
    Ok(())
}
