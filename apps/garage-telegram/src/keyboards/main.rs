use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn main_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([
        [
            InlineKeyboardButton::callback("📅 Записи", "nav:bookings"),
            InlineKeyboardButton::callback("👥 Клиенты", "nav:clients"),
        ],
        [
            InlineKeyboardButton::callback("🚗 Авто", "nav:cars"),
            InlineKeyboardButton::callback("📦 Склад", "nav:stock"),
        ],
        [
            InlineKeyboardButton::callback("⚠️ Остатки", "nav:low_stock"),
            InlineKeyboardButton::callback("🔍 Поиск", "nav:search"),
        ],
    ])
}
