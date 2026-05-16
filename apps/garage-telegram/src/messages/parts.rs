//! Тексты складского раздела.
//!
//! Модуль форматирует складские позиции, результаты поиска и форму создания
//! запчасти. Денежный ввод показывается пользователю в BYN через общий
//! formatter, чтобы подтверждение совпадало с тем, что сохранит handler.

use garage_domain::Part;

use crate::state::PartDraft;

use super::format::{format_byn_input, format_money};

/// Возвращает текст меню склада.
pub fn menu() -> &'static str {
    "📦 Склад. Выберите действие."
}

/// Возвращает prompt названия запчасти.
pub fn ask_name() -> &'static str {
    "Введите название запчасти:"
}

/// Возвращает prompt SKU/артикула.
pub fn ask_sku() -> &'static str {
    "Введите SKU/артикул или отправьте -:"
}

/// Возвращает prompt количества.
pub fn ask_quantity() -> &'static str {
    "Введите количество:"
}

/// Возвращает prompt минимального остатка.
pub fn ask_min_quantity() -> &'static str {
    "Введите минимальный остаток:"
}

/// Возвращает prompt цены за единицу.
pub fn ask_unit_price() -> &'static str {
    "Введите цену за единицу в BYN. Например: 25 или 25.50"
}

/// Возвращает prompt заметки по запчасти.
pub fn ask_notes() -> &'static str {
    "Введите заметку или отправьте -:"
}

/// Форматирует экран подтверждения складской позиции.
pub fn confirm(draft: &PartDraft) -> String {
    format!(
        "Проверьте запчасть:\n\nНазвание: {}\nSKU: {}\nКоличество: {}\nМин. остаток: {}\nЦена: {}\nЗаметка: {}",
        draft.name.as_deref().unwrap_or("не указано"),
        draft.sku.as_deref().unwrap_or("нет"),
        draft.quantity.as_deref().unwrap_or("не указано"),
        draft.min_quantity.as_deref().unwrap_or("не указано"),
        draft
            .unit_price
            .as_deref()
            .and_then(format_byn_input)
            .unwrap_or_else(|| "не указана".to_string()),
        draft.notes.as_deref().unwrap_or("нет")
    )
}

/// Форматирует карточку созданной запчасти.
pub fn created_card(part: &Part) -> String {
    format!("Запчасть создана\n\n{}", part_card(part))
}

/// Возвращает prompt поиска запчасти.
pub fn ask_search_query() -> &'static str {
    "Введите название или SKU:"
}

/// Форматирует результаты поиска запчастей.
pub fn search_results(query: &str, parts: &[Part]) -> String {
    let mut text = format!("🔍 Результаты поиска: {query}");

    for (index, part) in parts.iter().enumerate() {
        text.push_str(&format!(
            "\n\n{}. {}\n   SKU: {}\n   Остаток: {} шт\n   Цена: {}",
            index + 1,
            part.name().as_str(),
            part.sku().map(|sku| sku.as_str()).unwrap_or("нет"),
            part.quantity().value(),
            format_money(part.unit_price())
        ));
    }

    text
}

/// Возвращает текст для пустого поиска запчастей.
pub fn empty_search_results(query: &str) -> String {
    format!("По запросу `{query}` запчасти не найдены.")
}

/// Форматирует список позиций с низким остатком.
pub fn low_stock(parts: &[Part]) -> String {
    let mut text = "⚠️ Низкий остаток".to_string();

    for (index, part) in parts.iter().enumerate() {
        text.push_str(&format!(
            "\n\n{}. {}\n   Остаток: {} шт\n   Минимум: {} шт",
            index + 1,
            part.name().as_str(),
            part.quantity().value(),
            part.min_quantity().value()
        ));
    }

    text
}

/// Возвращает текст, если низких остатков нет.
pub fn low_stock_empty() -> &'static str {
    "✅ Все запчасти в норме."
}

/// Форматирует карточку складской позиции.
pub fn part_card(part: &Part) -> String {
    format!(
        "📦 {}\nSKU: {}\nОстаток: {} шт\nМинимум: {} шт\nЦена: {}\nСтатус: {}",
        part.name().as_str(),
        part.sku().map(|sku| sku.as_str()).unwrap_or("нет"),
        part.quantity().value(),
        part.min_quantity().value(),
        format_money(part.unit_price()),
        if part.is_low_stock() {
            "⚠️ низкий остаток"
        } else {
            "✅ в норме"
        }
    )
}

/// Возвращает prompt нового фактического остатка.
pub fn ask_new_stock() -> &'static str {
    "Введите новый фактический остаток:"
}

/// Возвращает текст ошибки количества.
pub fn invalid_quantity() -> &'static str {
    "Введите количество числом, например 5."
}

/// Возвращает текст ошибки цены.
pub fn invalid_price() -> &'static str {
    "Введите цену в BYN. Например: 25 или 25.50"
}

/// Возвращает текст для неполного черновика запчасти.
pub fn missing_required_fields() -> &'static str {
    "Не хватает данных запчасти. Начните добавление заново."
}
