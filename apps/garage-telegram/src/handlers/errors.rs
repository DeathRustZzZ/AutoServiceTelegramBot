//! Общая обработка ошибок handler-слоя.
//!
//! Здесь технические `AppError` логируются с деталями, а пользователю
//! показывается безопасный текст из `messages::errors`.

use teloxide::prelude::*;

use crate::keyboards;
use crate::messages;
use crate::state::{HandlerResult, SessionData, UserDialogue};
use crate::ui::render::{render_screen, Screen};

/// Логирует прикладную ошибку и возвращает пользовательский текст.
pub fn app_error_message(error: &garage_app::AppError) -> String {
    tracing::warn!(error = %error, "telegram handler app error");
    messages::errors::app_error(error)
}

/// Показывает экран для текста, который не подходит ни под навигацию, ни под форму.
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
