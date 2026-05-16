//! Тексты записей на обслуживание.
//!
//! Booking хранится в UTC, а пользователю показывается локальное время
//! автосервиса через смещение из конфигурации. Модуль не вычисляет период
//! выборки: он только форматирует уже загруженные данные.

use chrono::{DateTime, Duration, Utc};
use garage_app::BookingDetails;
use garage_domain::{Booking, Car, Client};

use crate::state::BookingDraft;

/// Возвращает текст меню записей.
pub fn menu() -> &'static str {
    "📅 Записи. Выберите действие."
}

/// Возвращает текст для отсутствия записей сегодня.
pub fn empty_today() -> &'static str {
    "📅 На сегодня записей нет."
}

/// Возвращает текст для отсутствия записей завтра.
pub fn empty_tomorrow() -> &'static str {
    "📅 На завтра записей нет."
}

/// Форматирует список записей на сегодня.
pub fn list_today(items: &[BookingDetails], offset_hours: i32) -> String {
    list("📅 Записи на сегодня", items, offset_hours)
}

/// Форматирует список записей на завтра.
pub fn list_tomorrow(items: &[BookingDetails], offset_hours: i32) -> String {
    list("📅 Записи на завтра", items, offset_hours)
}

/// Возвращает prompt поиска клиента для новой записи.
pub fn ask_client_query() -> &'static str {
    "Введите имя или телефон клиента:"
}

/// Форматирует результаты поиска клиента для записи.
pub fn client_search_results(query: &str, clients: &[Client]) -> String {
    let mut text = format!("Выберите клиента для записи: {query}\n");

    for (index, client) in clients.iter().enumerate() {
        text.push_str(&format!(
            "\n{}. {}\n   📞 {}\n",
            index + 1,
            client.name().as_str(),
            client.phone().as_str()
        ));
    }

    text
}

/// Возвращает текст для пустого поиска клиента при создании записи.
pub fn no_client_results(query: &str) -> String {
    format!("По запросу `{query}` клиентов не найдено.")
}

/// Форматирует список автомобилей выбранного клиента.
pub fn select_car(client: &Client, cars: &[Car]) -> String {
    let mut text = format!("Выберите авто клиента {}\n", client.name().as_str());

    for (index, car) in cars.iter().enumerate() {
        text.push_str(&format!("\n{}. {}\n", index + 1, car_title(car)));
    }

    text
}

/// Возвращает текст, если у выбранного клиента нет автомобилей.
pub fn no_cars_for_client(client: &Client) -> String {
    format!(
        "У клиента {} нет автомобилей. Сначала добавьте авто в карточке клиента.",
        client.name().as_str()
    )
}

/// Возвращает prompt даты и времени визита.
pub fn ask_datetime() -> &'static str {
    "Введите дату и время визита в формате ДД.ММ.ГГГГ ЧЧ:ММ"
}

/// Возвращает текст ошибки формата даты и времени.
pub fn invalid_datetime() -> &'static str {
    "Неверный формат даты. Пример: 15.05.2026 14:30"
}

/// Возвращает текст ошибки записи в прошлом.
pub fn past_datetime() -> &'static str {
    "Нельзя создать запись в прошлом."
}

/// Возвращает prompt причины обращения.
pub fn ask_reason() -> &'static str {
    "Введите причину обращения:"
}

/// Возвращает prompt заметки к записи.
pub fn ask_notes() -> &'static str {
    "Введите заметку или отправьте -"
}

/// Форматирует экран подтверждения записи.
pub fn confirm(client: &Client, car: &Car, draft: &BookingDraft, _offset_hours: i32) -> String {
    format!(
        "Проверьте запись:\n\nКлиент: {}\nАвто: {}\nДата: {}\nПричина: {}\nЗаметка: {}",
        client.name().as_str(),
        car_title(car),
        draft.scheduled_at.as_deref().unwrap_or("не указана"),
        draft.reason.as_deref().unwrap_or("не указана"),
        draft.notes.as_deref().unwrap_or("нет")
    )
}

/// Форматирует карточку записи.
pub fn booking_card(booking: &Booking, client: &Client, car: &Car, offset_hours: i32) -> String {
    format!(
        "📅 Запись\n\nКлиент: {}\nТелефон: {}\nАвто: {}\nВремя: {}\nПричина: {}\nЗаметка: {}\nСтатус: {}",
        client.name().as_str(),
        client.phone().as_str(),
        car_title(car),
        format_local_datetime(*booking.scheduled_at(), offset_hours),
        booking.reason().as_str(),
        booking.notes().map(|notes| notes.as_str()).unwrap_or("нет"),
        booking.status()
    )
}

/// Форматирует карточку созданной записи.
pub fn created_card(booking: &Booking, client: &Client, car: &Car, offset_hours: i32) -> String {
    format!(
        "Запись создана\n\n{}",
        booking_card(booking, client, car, offset_hours)
    )
}

/// Форматирует общий список записей с заданным заголовком.
fn list(title: &str, items: &[BookingDetails], offset_hours: i32) -> String {
    let mut text = title.to_string();

    for item in items {
        text.push_str(&format!(
            "\n\n{} — {} — {} — {}",
            format_local_time(*item.booking.scheduled_at(), offset_hours),
            item.client.name().as_str(),
            car_title(&item.car),
            item.booking.reason().as_str()
        ));
    }

    text
}

/// Собирает короткое название автомобиля.
fn car_title(car: &Car) -> String {
    match car.year() {
        Some(year) => format!(
            "{} {} {}",
            car.make().as_str(),
            car.model().as_str(),
            year.value()
        ),
        None => format!("{} {}", car.make().as_str(), car.model().as_str()),
    }
}

/// Форматирует UTC-время как локальное время автосервиса.
fn format_local_time(value: DateTime<Utc>, offset_hours: i32) -> String {
    (value + Duration::hours(i64::from(offset_hours)))
        .format("%H:%M")
        .to_string()
}

/// Форматирует UTC дату-время как локальную дату-время автосервиса.
fn format_local_datetime(value: DateTime<Utc>, offset_hours: i32) -> String {
    (value + Duration::hours(i64::from(offset_hours)))
        .format("%d.%m.%Y %H:%M")
        .to_string()
}
