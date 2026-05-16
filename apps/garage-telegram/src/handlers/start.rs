//! Handler команды `/start`.
//!
//! Команда обновляет нижнюю reply-клавиатуру и возвращает пользователя на
//! главный экран, не создавая доменных данных.

use teloxide::prelude::*;

use crate::keyboards;
use crate::messages;
use crate::state::{HandlerResult, SessionData, UserDialogue};
use crate::ui::render::{render_screen, Screen};

/// Обрабатывает `/start` и сбрасывает активный диалог.
pub async fn start(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    mut session: SessionData,
) -> HandlerResult {
    session.reset_dialog();

    bot.send_message(msg.chat.id, "Клавиатура обновлена.")
        .reply_markup(keyboards::reply::global_navigation())
        .await?;

    render_screen(
        &bot,
        &dialogue,
        msg.chat.id,
        session,
        Screen::new(messages::main::main_menu(), keyboards::main::main_menu()),
    )
    .await
}
