use teloxide::prelude::*;
use teloxide::types::{CallbackQuery, ChatId, UserId};

use crate::container::AppContainer;

const ACCESS_DENIED: &str = "Доступ запрещён.";
type AccessResult = Result<bool, Box<dyn std::error::Error + Send + Sync>>;

pub async fn ensure_message_access(
    bot: &Bot,
    msg: &Message,
    container: &AppContainer,
) -> AccessResult {
    if is_allowed(
        container,
        msg.chat.id,
        msg.from.as_ref().map(|user| user.id),
    ) {
        return Ok(true);
    }

    tracing::warn!(chat_id = msg.chat.id.0, "denied access to message update");
    bot.send_message(msg.chat.id, ACCESS_DENIED).await?;
    Ok(false)
}

pub async fn ensure_callback_access(
    bot: &Bot,
    query: &CallbackQuery,
    container: &AppContainer,
) -> AccessResult {
    let chat_id = query.message.as_ref().map(|message| message.chat().id);
    let allowed = match chat_id {
        Some(chat_id) => is_allowed(container, chat_id, Some(query.from.id)),
        None => is_user_allowed(container, query.from.id),
    };

    if allowed {
        return Ok(true);
    }

    if let Some(chat_id) = chat_id {
        tracing::warn!(chat_id = chat_id.0, "denied access to callback update");
    } else {
        tracing::warn!(
            user_id = query.from.id.0,
            "denied access to callback without message"
        );
    }

    bot.answer_callback_query(query.id.clone())
        .text(ACCESS_DENIED)
        .await?;
    Ok(false)
}

fn is_allowed(container: &AppContainer, chat_id: ChatId, user_id: Option<UserId>) -> bool {
    let Some(owner_id) = container.owner_chat_id() else {
        return true;
    };

    chat_id.0 == owner_id
        || user_id.is_some_and(|user_id| user_id_to_i64(user_id) == Some(owner_id))
}

fn is_user_allowed(container: &AppContainer, user_id: UserId) -> bool {
    let Some(owner_id) = container.owner_chat_id() else {
        return true;
    };

    user_id_to_i64(user_id) == Some(owner_id)
}

fn user_id_to_i64(user_id: UserId) -> Option<i64> {
    i64::try_from(user_id.0).ok()
}
