pub mod bookings;
pub mod cars;
pub mod clients;
pub mod main;
pub mod parts;
pub mod repairs;
pub mod reply;

use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn empty_inline_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(Vec::<Vec<InlineKeyboardButton>>::new())
}

pub fn cancel_row() -> Vec<InlineKeyboardButton> {
    vec![InlineKeyboardButton::callback("❌ Отмена", "nav:cancel")]
}
