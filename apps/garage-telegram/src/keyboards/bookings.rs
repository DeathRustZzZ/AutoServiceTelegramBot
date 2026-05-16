use garage_app::BookingDetails;
use garage_domain::{Booking, Car, Client, ClientId};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn menu() -> InlineKeyboardMarkup {
    super::empty_inline_keyboard()
}

pub fn list(items: &[BookingDetails]) -> InlineKeyboardMarkup {
    let mut rows = Vec::new();

    if !items.is_empty() {
        rows.push(
            items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    InlineKeyboardButton::callback(
                        (index + 1).to_string(),
                        format!("booking:open:{}", item.booking.id().as_uuid()),
                    )
                })
                .collect::<Vec<_>>(),
        );
    }

    InlineKeyboardMarkup::new(rows)
}

pub fn empty_list() -> InlineKeyboardMarkup {
    super::empty_inline_keyboard()
}

pub fn client_search_results(clients: &[Client]) -> InlineKeyboardMarkup {
    let mut rows = Vec::new();

    if !clients.is_empty() {
        rows.push(
            clients
                .iter()
                .enumerate()
                .map(|(index, client)| {
                    InlineKeyboardButton::callback(
                        (index + 1).to_string(),
                        format!("booking:client:{}", client.id().as_uuid()),
                    )
                })
                .collect::<Vec<_>>(),
        );
    }

    rows.push(super::cancel_row());

    InlineKeyboardMarkup::new(rows)
}

pub fn select_car(cars: &[Car]) -> InlineKeyboardMarkup {
    let mut rows = Vec::new();

    if !cars.is_empty() {
        rows.push(
            cars.iter()
                .enumerate()
                .map(|(index, car)| {
                    InlineKeyboardButton::callback(
                        (index + 1).to_string(),
                        format!("booking:car:{}", car.id().as_uuid()),
                    )
                })
                .collect::<Vec<_>>(),
        );
    }

    rows.push(super::cancel_row());

    InlineKeyboardMarkup::new(rows)
}

pub fn no_cars(client_id: ClientId) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "👤 Открыть клиента",
            format!("client:open:{}", client_id.as_uuid()),
        )],
        super::cancel_row(),
    ])
}

pub fn confirm() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "✅ Сохранить",
            "booking:confirm",
        )],
        super::cancel_row(),
    ])
}

pub fn booking_card(booking: &Booking) -> InlineKeyboardMarkup {
    let mut rows = Vec::new();

    if booking.is_scheduled() {
        rows.push(vec![InlineKeyboardButton::callback(
            "🔧 Начать ремонт",
            format!("booking:start_repair:{}", booking.id().as_uuid()),
        )]);
        rows.push(vec![InlineKeyboardButton::callback(
            "✅ Выполнена",
            format!("booking:complete:{}", booking.id().as_uuid()),
        )]);
        rows.push(vec![InlineKeyboardButton::callback(
            "❌ Отменить",
            format!("booking:cancel:{}", booking.id().as_uuid()),
        )]);
    }

    InlineKeyboardMarkup::new(rows)
}

pub fn back_to_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([super::cancel_row()])
}
