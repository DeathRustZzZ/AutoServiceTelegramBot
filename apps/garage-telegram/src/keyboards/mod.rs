//! Inline-клавиатуры Telegram UI.
//!
//! Callback data из этих клавиатур является внутренним UI-протоколом и должна
//! оставаться согласованной с `routing::callbacks`. Доменные id передаются как
//! UUID-строки, а handler повторно загружает сущности перед любой мутацией.

pub mod bookings;
pub mod cars;
pub mod clients;
pub mod main;
pub mod parts;
pub mod repairs;
pub mod reply;

use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

/// Пустая inline-клавиатура для экранов без действий.
pub fn empty_inline_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(Vec::<Vec<InlineKeyboardButton>>::new())
}

/// Стандартная строка отмены текущего диалога.
pub fn cancel_row() -> Vec<InlineKeyboardButton> {
    vec![InlineKeyboardButton::callback("❌ Отмена", "nav:cancel")]
}
