use teloxide::dispatching::{dialogue, UpdateHandler};
use teloxide::prelude::*;

use crate::state::{HandlerResult, SessionData, Storage};

pub mod access;
pub mod callbacks;
pub mod text;

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
