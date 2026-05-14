use garage_domain::Client;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

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
        [InlineKeyboardButton::callback(
            "🏠 Главное меню",
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
            "✅ Сохранить",
            "client:confirm",
        )],
        [InlineKeyboardButton::callback(
            "⬅️ К клиентам",
            "nav:clients",
        )],
    ])
}

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

    rows.push(vec![InlineKeyboardButton::callback(
        "🏠 Главное меню",
        "nav:main",
    )]);

    InlineKeyboardMarkup::new(rows)
}

pub fn empty_clients() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([
        [InlineKeyboardButton::callback(
            "➕ Добавить клиента",
            "client:add",
        )],
        [InlineKeyboardButton::callback(
            "🏠 Главное меню",
            "nav:main",
        )],
    ])
}

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

    rows.push(vec![InlineKeyboardButton::callback(
        "⬅️ К клиентам",
        "nav:clients",
    )]);
    rows.push(vec![InlineKeyboardButton::callback(
        "🏠 Главное меню",
        "nav:main",
    )]);

    InlineKeyboardMarkup::new(rows)
}

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
        [InlineKeyboardButton::callback(
            "⬅️ К клиентам",
            "nav:clients",
        )],
        [InlineKeyboardButton::callback(
            "🏠 Главное меню",
            "nav:main",
        )],
    ])
}
