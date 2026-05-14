use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn main_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("📅 Записи", "nav:bookings"),
            InlineKeyboardButton::callback("👥 Клиенты", "nav:clients"),
        ],
        vec![
            InlineKeyboardButton::callback("🚗 Авто", "nav:cars"),
            InlineKeyboardButton::callback("📦 Склад", "nav:stock"),
        ],
        vec![
            InlineKeyboardButton::callback("⚠️ Остатки", "nav:low_stock"),
            InlineKeyboardButton::callback("🔧 Ремонты", "nav:repairs"),
        ],
        vec![InlineKeyboardButton::callback("🔍 Поиск", "nav:search")],
    ])
}
