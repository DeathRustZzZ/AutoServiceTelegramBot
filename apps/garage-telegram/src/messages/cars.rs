//! Тексты раздела автомобилей.
//!
//! Здесь собираются человекочитаемые карточки автомобилей и prompts формы
//! добавления. Проверка VIN, года и номера остается в `garage-domain`.

use garage_domain::{Car, Client};

use crate::state::CarDraft;

/// Форматирует список автомобилей клиента.
pub fn list_client_cars(client: &Client, cars: &[Car]) -> String {
    let mut text = format!("🚗 Авто клиента\n👤 {}\n", client.name().as_str());

    for (index, car) in cars.iter().enumerate() {
        text.push_str(&format!(
            "\n{}. {}\n   Госномер: {}\n   VIN: {}\n",
            index + 1,
            car_title(car),
            car.license_plate()
                .map(|plate| plate.as_str())
                .unwrap_or("не указан"),
            car.vin().map(|vin| vin.as_str()).unwrap_or("не указан")
        ));
    }

    text
}

/// Возвращает текст для клиента без автомобилей.
pub fn empty_client_cars(client: &Client) -> String {
    format!(
        "🚗 У клиента {} пока нет автомобилей.",
        client.name().as_str()
    )
}

/// Возвращает prompt марки автомобиля.
pub fn ask_make() -> &'static str {
    "Введите марку автомобиля:"
}

/// Возвращает prompt модели автомобиля.
pub fn ask_model() -> &'static str {
    "Введите модель автомобиля:"
}

/// Возвращает prompt года выпуска автомобиля.
pub fn ask_year() -> &'static str {
    "Введите год выпуска или отправьте -, если не нужно:"
}

/// Возвращает prompt государственного номера.
pub fn ask_license_plate() -> &'static str {
    "Введите госномер или отправьте -, если не нужно:"
}

/// Возвращает prompt VIN.
pub fn ask_vin() -> &'static str {
    "Введите VIN или отправьте -, если не нужно:"
}

/// Форматирует экран подтверждения перед созданием автомобиля.
pub fn confirm(client: &Client, draft: &CarDraft) -> String {
    format!(
        "Проверьте данные автомобиля:\n\nКлиент: {}\nМарка: {}\nМодель: {}\nГод: {}\nГосномер: {}\nVIN: {}",
        client.name().as_str(),
        draft.make.as_deref().unwrap_or("не указана"),
        draft.model.as_deref().unwrap_or("не указана"),
        optional_field(draft.year.as_deref()),
        optional_field(draft.license_plate.as_deref()),
        optional_field(draft.vin.as_deref())
    )
}

/// Форматирует карточку только что созданного автомобиля.
pub fn created_card(car: &Car, client: &Client) -> String {
    car_card(car, client)
}

/// Форматирует карточку автомобиля.
pub fn car_card(car: &Car, client: &Client) -> String {
    format!(
        "🚗 {}\n👤 Клиент: {}\nГосномер: {}\nVIN: {}",
        car_title(car),
        client.name().as_str(),
        car.license_plate()
            .map(|plate| plate.as_str())
            .unwrap_or("не указан"),
        car.vin().map(|vin| vin.as_str()).unwrap_or("не указан")
    )
}

/// Возвращает текст для устаревшего или неполного черновика автомобиля.
pub fn missing_draft() -> &'static str {
    "Данные автомобиля устарели. Откройте карточку клиента и начните добавление заново."
}

/// Возвращает текст ошибки года выпуска.
pub fn invalid_year() -> &'static str {
    "Проверьте год выпуска. Укажите год числом или отправьте -."
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

/// Форматирует опциональное поле автомобиля для экрана подтверждения.
fn optional_field(value: Option<&str>) -> &str {
    match value.map(str::trim) {
        Some(value) if !value.is_empty() && value != "-" => value,
        _ => "не указан",
    }
}
