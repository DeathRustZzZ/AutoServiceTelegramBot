use garage_domain::{Car, ClientId};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn client_cars(client_id: ClientId, cars: &[Car]) -> InlineKeyboardMarkup {
    let mut rows = Vec::new();

    if !cars.is_empty() {
        rows.push(
            cars.iter()
                .enumerate()
                .map(|(index, car)| {
                    InlineKeyboardButton::callback(
                        (index + 1).to_string(),
                        format!("car:open:{}", car.id().as_uuid()),
                    )
                })
                .collect::<Vec<_>>(),
        );
    }

    rows.push(vec![InlineKeyboardButton::callback(
        "➕ Добавить авто",
        format!("car:add:{}", client_id.as_uuid()),
    )]);
    rows.push(vec![InlineKeyboardButton::callback(
        "⬅️ К клиенту",
        format!("client:open:{}", client_id.as_uuid()),
    )]);
    rows.push(vec![InlineKeyboardButton::callback(
        "🏠 Главное меню",
        "nav:main",
    )]);

    InlineKeyboardMarkup::new(rows)
}

pub fn empty_client_cars(client_id: ClientId) -> InlineKeyboardMarkup {
    client_cars(client_id, &[])
}

pub fn add_car_back_to_client(client_id: ClientId) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([
        [InlineKeyboardButton::callback(
            "⬅️ К клиенту",
            format!("client:open:{}", client_id.as_uuid()),
        )]
        .to_vec(),
        super::cancel_row(),
    ])
}

pub fn add_car_confirm(client_id: ClientId) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "✅ Сохранить",
            "car:confirm",
        )],
        vec![InlineKeyboardButton::callback(
            "⬅️ К клиенту",
            format!("client:open:{}", client_id.as_uuid()),
        )],
        super::cancel_row(),
    ])
}

pub fn car_card(client_id: ClientId) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([
        [InlineKeyboardButton::callback(
            "⬅️ К авто клиента",
            format!("client:cars:{}", client_id.as_uuid()),
        )],
        [InlineKeyboardButton::callback(
            "🏠 Главное меню",
            "nav:main",
        )],
    ])
}
