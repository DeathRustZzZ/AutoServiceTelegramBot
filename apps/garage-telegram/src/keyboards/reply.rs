//! Нижняя reply-клавиатура глобальной навигации.
//!
//! Эти кнопки приходят как обычный текст, поэтому их значения являются частью
//! routing-контракта `routing::text`.

use teloxide::types::{KeyboardButton, KeyboardMarkup};

pub const NAV_BOOKINGS: &str = "📅 Записи";
pub const NAV_CLIENTS: &str = "👥 Клиенты";
pub const NAV_CARS: &str = "🚗 Авто";
pub const NAV_STOCK: &str = "📦 Склад";
pub const NAV_LOW_STOCK: &str = "⚠️ Остатки";
pub const NAV_REPAIRS: &str = "🔧 Ремонты";
pub const NAV_SEARCH: &str = "🔍 Поиск";

/// Создает persistent клавиатуру с основными разделами автосервиса.
pub fn global_navigation() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![
            KeyboardButton::new(NAV_BOOKINGS),
            KeyboardButton::new(NAV_CLIENTS),
        ],
        vec![
            KeyboardButton::new(NAV_CARS),
            KeyboardButton::new(NAV_STOCK),
        ],
        vec![
            KeyboardButton::new(NAV_LOW_STOCK),
            KeyboardButton::new(NAV_REPAIRS),
        ],
        vec![KeyboardButton::new(NAV_SEARCH)],
    ])
    .resize_keyboard()
    .persistent()
}
