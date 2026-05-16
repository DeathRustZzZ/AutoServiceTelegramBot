//! Inline-клавиатура главного экрана.
//!
//! Основная навигация вынесена в reply-клавиатуру, поэтому главный экран пока
//! не содержит inline-действий.

use teloxide::types::InlineKeyboardMarkup;

/// Возвращает inline-клавиатуру главного экрана.
pub fn main_menu() -> InlineKeyboardMarkup {
    super::empty_inline_keyboard()
}
