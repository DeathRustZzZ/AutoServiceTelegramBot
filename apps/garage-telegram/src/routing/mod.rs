//! Схема маршрутизации Telegram update'ов.
//!
//! Routing слой не выполняет бизнес-действия сам. Он разделяет входящие
//! update'ы на текстовые сообщения и callback query, подключает dialogue
//! storage и передает управление специализированным handler'ам.

use teloxide::dispatching::{dialogue, UpdateHandler};
use teloxide::prelude::*;

use crate::state::{HandlerResult, SessionData, Storage};

pub mod access;
pub mod callbacks;
pub mod text;

/// Собирает dptree-схему для всех поддерживаемых Telegram update'ов.
pub fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync>> {
    let messages = Update::filter_message()
        .enter_dialogue::<Message, Storage, SessionData>()
        .endpoint(text::handle);

    let callbacks = Update::filter_callback_query()
        .enter_dialogue::<CallbackQuery, Storage, SessionData>()
        .endpoint(callbacks::handle);

    dialogue::enter::<Update, Storage, SessionData, HandlerResult>()
        .branch(messages)
        .branch(callbacks)
}
