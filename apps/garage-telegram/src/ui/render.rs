//! Отрисовка основного Telegram-экрана.
//!
//! Бот старается работать как одноэкранное приложение: вместо отправки новой
//! карточки на каждый шаг он редактирует последнее меню. Если Telegram не дает
//! отредактировать сообщение, модуль отправляет новый экран и сохраняет его id.

use teloxide::prelude::*;
use teloxide::types::{ChatId, InlineKeyboardMarkup};

use crate::state::{HandlerResult, SessionData, UserDialogue};

/// Описание экрана, который должен увидеть пользователь.
pub struct Screen {
    /// Markdown не используется: текст передается как обычная Telegram-строка.
    pub text: String,
    /// Inline-клавиатура с действиями текущего экрана.
    pub keyboard: InlineKeyboardMarkup,
}

impl Screen {
    /// Создает экран из текста и inline-клавиатуры.
    pub fn new(text: impl Into<String>, keyboard: InlineKeyboardMarkup) -> Self {
        Self {
            text: text.into(),
            keyboard,
        }
    }
}

/// Редактирует последний экран или отправляет новый, если редактирование невозможно.
///
/// Метод также сохраняет обновленную `SessionData`, поэтому handler'ы должны
/// вызывать его в конце перехода состояния.
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

        match edited {
            Ok(_) => {
                dialogue.update(session).await?;
                return Ok(());
            }
            Err(error) => {
                tracing::warn!(
                    chat_id = chat_id.0,
                    message_id = message_id.0,
                    error = %error,
                    "failed to edit menu message; falling back to send_message"
                );
            }
        }
    }

    let message = send_menu_message(bot, chat_id, screen).await?;
    session.last_menu_msg_id = Some(message.id);
    dialogue.update(session).await?;

    Ok(())
}

/// Отправляет новое экранное сообщение без обновления session state.
pub async fn send_menu_message(
    bot: &Bot,
    chat_id: ChatId,
    screen: Screen,
) -> Result<Message, teloxide::RequestError> {
    bot.send_message(chat_id, screen.text)
        .reply_markup(screen.keyboard)
        .await
}
