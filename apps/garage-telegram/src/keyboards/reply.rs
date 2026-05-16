use teloxide::types::{KeyboardButton, KeyboardMarkup};

pub const NAV_BOOKINGS: &str = "📅 Записи";
pub const NAV_CLIENTS: &str = "👥 Клиенты";
pub const NAV_CARS: &str = "🚗 Авто";
pub const NAV_STOCK: &str = "📦 Склад";
pub const NAV_LOW_STOCK: &str = "⚠️ Остатки";
pub const NAV_REPAIRS: &str = "🔧 Ремонты";
pub const NAV_SEARCH: &str = "🔍 Поиск";

pub const NAV_BACK: &str = "⬅️ Назад";
pub const NAV_CANCEL: &str = "❌ Отмена";

pub const CLIENT_ADD: &str = "➕ Добавить клиента";
pub const CLIENT_LIST: &str = "📋 Список клиентов";
pub const CLIENT_SEARCH: &str = "🔍 Найти клиента";

pub const BOOKING_TODAY: &str = "📋 Сегодня";
pub const BOOKING_TOMORROW: &str = "📋 Завтра";
pub const BOOKING_ADD: &str = "➕ Создать запись";

pub const PART_ADD: &str = "➕ Добавить запчасть";
pub const PART_SEARCH: &str = "🔍 Найти запчасть";
pub const PART_LOW_STOCK: &str = "⚠️ Низкий остаток";

pub const REPAIR_ACTIVE: &str = "🔥 Активные ремонты";

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

pub fn clients_navigation() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![KeyboardButton::new(CLIENT_ADD)],
        vec![KeyboardButton::new(CLIENT_LIST)],
        vec![KeyboardButton::new(CLIENT_SEARCH)],
        vec![KeyboardButton::new(NAV_BACK)],
    ])
    .resize_keyboard()
    .persistent()
}

pub fn bookings_navigation() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![KeyboardButton::new(BOOKING_TODAY)],
        vec![KeyboardButton::new(BOOKING_TOMORROW)],
        vec![KeyboardButton::new(BOOKING_ADD)],
        vec![KeyboardButton::new(NAV_BACK)],
    ])
    .resize_keyboard()
    .persistent()
}

pub fn parts_navigation() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![KeyboardButton::new(PART_ADD)],
        vec![KeyboardButton::new(PART_SEARCH)],
        vec![KeyboardButton::new(PART_LOW_STOCK)],
        vec![KeyboardButton::new(NAV_BACK)],
    ])
    .resize_keyboard()
    .persistent()
}

pub fn repairs_navigation() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![KeyboardButton::new(REPAIR_ACTIVE)],
        vec![KeyboardButton::new(NAV_BACK)],
    ])
    .resize_keyboard()
    .persistent()
}

pub fn dialog_navigation() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![vec![KeyboardButton::new(NAV_CANCEL)]])
        .resize_keyboard()
        .persistent()
}
