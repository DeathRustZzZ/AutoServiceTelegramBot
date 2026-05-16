use teloxide::prelude::*;

use crate::keyboards;
use crate::messages;
use crate::state::{HandlerResult, SessionData, UserDialogue};
use crate::ui::render::{render_screen, Screen};
use crate::ui::reply_preset::send_reply_keyboard_notice;

pub async fn start(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    mut session: SessionData,
) -> HandlerResult {
    session.reset_dialog();

    send_reply_keyboard_notice(&bot, msg.chat.id, keyboards::reply::global_navigation()).await;

    render_screen(
        &bot,
        &dialogue,
        msg.chat.id,
        session,
        Screen::new(messages::main::main_menu(), keyboards::main::main_menu()),
    )
    .await
}
