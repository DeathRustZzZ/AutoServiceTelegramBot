use garage_domain::{Money, Part};

use crate::state::PartDraft;

pub fn menu() -> &'static str {
    "📦 Склад"
}

pub fn ask_name() -> &'static str {
    "Введите название запчасти:"
}

pub fn ask_sku() -> &'static str {
    "Введите SKU/артикул или отправьте -:"
}

pub fn ask_quantity() -> &'static str {
    "Введите количество:"
}

pub fn ask_min_quantity() -> &'static str {
    "Введите минимальный остаток:"
}

pub fn ask_unit_price() -> &'static str {
    "Введите цену за единицу в копейках BYN:"
}

pub fn ask_notes() -> &'static str {
    "Введите заметку или отправьте -:"
}

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
            .and_then(|value| value.parse::<i64>().ok())
            .map(format_minor_byn)
            .unwrap_or_else(|| "не указана".to_string()),
        draft.notes.as_deref().unwrap_or("нет")
    )
}

pub fn created_card(part: &Part) -> String {
    format!("Запчасть создана\n\n{}", part_card(part))
}

pub fn ask_search_query() -> &'static str {
    "Введите название или SKU:"
}

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

pub fn empty_search_results(query: &str) -> String {
    format!("По запросу `{query}` запчасти не найдены.")
}

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

pub fn low_stock_empty() -> &'static str {
    "✅ Все запчасти в норме."
}

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

pub fn ask_new_stock() -> &'static str {
    "Введите новый фактический остаток:"
}

pub fn invalid_quantity() -> &'static str {
    "Введите количество числом, например 5."
}

pub fn invalid_price() -> &'static str {
    "Цена должна быть указана в копейках, например 2500 для 25.00 BYN."
}

pub fn missing_required_fields() -> &'static str {
    "Не хватает данных запчасти. Начните добавление заново."
}

fn format_money(value: Money) -> String {
    format_minor_byn(value.amount_minor())
}

fn format_minor_byn(value: i64) -> String {
    format!("{}.{:02} BYN", value / 100, value % 100)
}
