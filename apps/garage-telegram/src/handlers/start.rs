use teloxide::prelude::*;

use crate::keyboards;
use crate::messages;
use crate::state::{HandlerResult, SessionData, UserDialogue};
use crate::ui::render::{render_screen, Screen};

pub async fn start(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    mut session: SessionData,
) -> HandlerResult {
    session.reset_dialog();
    session.last_menu_msg_id = None;

    bot.send_message(msg.chat.id, messages::main::welcome())
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
