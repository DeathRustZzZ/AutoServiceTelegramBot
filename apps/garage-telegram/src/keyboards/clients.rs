//! Inline-клавиатуры клиентского раздела.
//!
//! Кнопки списков используют позиционные номера в тексте и UUID в callback
//! data, поэтому пользователь видит короткие кнопки, а handler получает
//! устойчивый идентификатор клиента.

use garage_domain::Client;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

/// Создает меню клиентского раздела.
pub fn clients_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([
        [InlineKeyboardButton::callback(
            "➕ Добавить клиента",
            "client:add",
        )],
        [InlineKeyboardButton::callback(
            "📋 Список клиентов",
            "client:list:0",
        )],
        [InlineKeyboardButton::callback(
            "🔍 Найти клиента",
            "client:search",
        )],
    ])
}

/// Создает клавиатуру возврата/отмены из формы клиента.
pub fn add_client_back_to_clients() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([super::cancel_row()])
}

/// Создает клавиатуру подтверждения создания клиента.
pub fn add_client_confirm() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "✅ Сохранить",
            "client:confirm",
        )],
        super::cancel_row(),
    ])
}

/// Создает клавиатуру страницы клиентов с pagination.
pub fn clients_list(clients: &[Client], page: usize, has_next: bool) -> InlineKeyboardMarkup {
    let mut rows = Vec::new();

    if !clients.is_empty() {
        rows.push(
            clients
                .iter()
                .enumerate()
                .map(|(index, client)| {
                    InlineKeyboardButton::callback(
                        (index + 1).to_string(),
                        format!("client:open:{}", client.id().as_uuid()),
                    )
                })
                .collect::<Vec<_>>(),
        );
    }

    let mut pagination = Vec::new();
    if page > 0 {
        pagination.push(InlineKeyboardButton::callback(
            "← Назад",
            format!("client:list:{}", page - 1),
        ));
    }
    if has_next {
        pagination.push(InlineKeyboardButton::callback(
            "Далее →",
            format!("client:list:{}", page + 1),
        ));
    }
    if !pagination.is_empty() {
        rows.push(pagination);
    }

    InlineKeyboardMarkup::new(rows)
}

/// Создает клавиатуру пустого списка клиентов.
pub fn empty_clients() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
        "➕ Добавить клиента",
        "client:add",
    )]])
}

/// Создает клавиатуру результатов поиска клиентов.
pub fn search_results(clients: &[Client]) -> InlineKeyboardMarkup {
    let mut rows = Vec::new();

    if !clients.is_empty() {
        rows.push(
            clients
                .iter()
                .enumerate()
                .map(|(index, client)| {
                    InlineKeyboardButton::callback(
                        (index + 1).to_string(),
                        format!("client:open:{}", client.id().as_uuid()),
                    )
                })
                .collect::<Vec<_>>(),
        );
    }

    InlineKeyboardMarkup::new(rows)
}

/// Создает клавиатуру карточки клиента.
pub fn client_card(client: &Client) -> InlineKeyboardMarkup {
    let client_id = client.id().as_uuid();

    InlineKeyboardMarkup::new([
        [InlineKeyboardButton::callback(
            "🚗 Авто клиента",
            format!("client:cars:{client_id}"),
        )],
        [InlineKeyboardButton::callback(
            "➕ Добавить авто",
            format!("car:add:{client_id}"),
        )],
    ])
}
