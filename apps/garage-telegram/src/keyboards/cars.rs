//! Inline-клавиатуры раздела автомобилей.
//!
//! Все действия с автомобилем возвращают пользователя к карточке клиента или
//! списку его автомобилей, чтобы не терять контекст владельца.

use garage_domain::{Car, ClientId};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

/// Создает клавиатуру списка автомобилей клиента.
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

    InlineKeyboardMarkup::new(rows)
}

/// Создает клавиатуру для клиента без автомобилей.
pub fn empty_client_cars(client_id: ClientId) -> InlineKeyboardMarkup {
    client_cars(client_id, &[])
}

/// Создает клавиатуру возврата из формы добавления автомобиля.
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

/// Создает клавиатуру подтверждения добавления автомобиля.
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

/// Создает клавиатуру карточки автомобиля.
pub fn car_card(client_id: ClientId) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
        "⬅️ К авто клиента",
        format!("client:cars:{}", client_id.as_uuid()),
    )]])
}
