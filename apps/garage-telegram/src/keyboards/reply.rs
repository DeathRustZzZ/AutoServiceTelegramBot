use teloxide::types::{KeyboardButton, KeyboardMarkup};

pub const NAV_BOOKINGS: &str = "📅 Записи";
pub const NAV_CLIENTS: &str = "👥 Клиенты";
pub const NAV_CARS: &str = "🚗 Авто";
pub const NAV_STOCK: &str = "📦 Склад";
pub const NAV_LOW_STOCK: &str = "⚠️ Остатки";
pub const NAV_SEARCH: &str = "🔍 Поиск";

pub fn global_navigation() -> KeyboardMarkup {
    KeyboardMarkup::new([
        [
            KeyboardButton::new(NAV_BOOKINGS),
            KeyboardButton::new(NAV_CLIENTS),
        ],
        [
            KeyboardButton::new(NAV_CARS),
            KeyboardButton::new(NAV_STOCK),
        ],
        [
            KeyboardButton::new(NAV_LOW_STOCK),
            KeyboardButton::new(NAV_SEARCH),
        ],
    ])
    .resize_keyboard()
    .persistent()
}
