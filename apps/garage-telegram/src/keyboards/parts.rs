use garage_domain::Part;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn menu() -> InlineKeyboardMarkup {
    super::empty_inline_keyboard()
}

pub fn add_part_back_to_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([super::cancel_row()])
}

pub fn add_part_confirm() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "✅ Сохранить",
            "part:confirm",
        )],
        super::cancel_row(),
    ])
}

pub fn search_results(parts: &[Part]) -> InlineKeyboardMarkup {
    let mut rows = Vec::new();

    if !parts.is_empty() {
        rows.push(
            parts
                .iter()
                .enumerate()
                .map(|(index, part)| {
                    InlineKeyboardButton::callback(
                        (index + 1).to_string(),
                        format!("part:open:{}", part.id().as_uuid()),
                    )
                })
                .collect::<Vec<_>>(),
        );
    }

    InlineKeyboardMarkup::new(rows)
}

pub fn low_stock(parts: &[Part]) -> InlineKeyboardMarkup {
    let mut rows = Vec::new();

    if !parts.is_empty() {
        rows.push(
            parts
                .iter()
                .enumerate()
                .map(|(index, part)| {
                    InlineKeyboardButton::callback(
                        (index + 1).to_string(),
                        format!("part:open:{}", part.id().as_uuid()),
                    )
                })
                .collect::<Vec<_>>(),
        );
    }

    InlineKeyboardMarkup::new(rows)
}

pub fn part_card(part: &Part) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([
        [InlineKeyboardButton::callback(
            "🔢 Изменить остаток",
            format!("part:set_stock:{}", part.id().as_uuid()),
        )],
        [InlineKeyboardButton::callback(
            "🔍 Найти ещё",
            "part:search",
        )],
    ])
}

pub fn back_to_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([super::cancel_row()])
}
