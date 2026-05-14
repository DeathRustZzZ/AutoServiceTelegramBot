use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn clients_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([
        [InlineKeyboardButton::callback(
            "➕ Добавить клиента",
            "client:add",
        )],
        [InlineKeyboardButton::callback(
            "⬅️ Главное меню",
            "nav:main",
        )],
    ])
}

pub fn add_client_back_to_clients() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
        "⬅️ К клиентам",
        "nav:clients",
    )]])
}

pub fn add_client_confirm() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([
        [InlineKeyboardButton::callback(
            "✅ Сохранить позже",
            "client:confirm",
        )],
        [InlineKeyboardButton::callback(
            "⬅️ К клиентам",
            "nav:clients",
        )],
    ])
}
