//! Inline-клавиатуры раздела ремонтов.
//!
//! Набор действий зависит от статуса ремонта: активный ремонт можно изменять,
//! оплачивать, пополнять запчастями и закрывать, а финальный ремонт остается
//! read-only экраном.

use garage_app::RepairDetails;
use garage_domain::{BookingId, Part, Repair, RepairId};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

/// Создает меню ремонтного раздела.
pub fn menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
        "🔥 Активные ремонты",
        "repair:active",
    )]])
}

/// Создает клавиатуру списка активных ремонтов.
pub fn active_list(items: &[RepairDetails]) -> InlineKeyboardMarkup {
    let mut rows = Vec::new();

    if !items.is_empty() {
        rows.push(
            items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    InlineKeyboardButton::callback(
                        (index + 1).to_string(),
                        format!("repair:open:{}", item.repair.id().as_uuid()),
                    )
                })
                .collect::<Vec<_>>(),
        );
    }

    InlineKeyboardMarkup::new(rows)
}

/// Создает клавиатуру пустого списка активных ремонтов.
pub fn active_empty() -> InlineKeyboardMarkup {
    super::empty_inline_keyboard()
}

/// Создает клавиатуру подтверждения запуска ремонта из записи.
pub fn start_confirm(booking_id: BookingId) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "✅ Начать ремонт",
            "repair:confirm_start",
        )],
        vec![InlineKeyboardButton::callback(
            "⬅️ К записи",
            format!("booking:open:{}", booking_id.as_uuid()),
        )],
        super::cancel_row(),
    ])
}

/// Создает клавиатуру карточки ремонта.
pub fn repair_card(repair: &Repair) -> InlineKeyboardMarkup {
    let mut rows = Vec::new();

    if repair.is_in_progress() {
        rows.push(vec![InlineKeyboardButton::callback(
            "💵 Изменить работы",
            format!("repair:set_labor:{}", repair.id().as_uuid()),
        )]);
        rows.push(vec![InlineKeyboardButton::callback(
            "💰 Принять оплату",
            format!("repair:payment:{}", repair.id().as_uuid()),
        )]);
        rows.push(vec![InlineKeyboardButton::callback(
            "📦 Добавить запчасть",
            format!("repair:add_part:{}", repair.id().as_uuid()),
        )]);
        rows.push(vec![InlineKeyboardButton::callback(
            "✅ Завершить",
            format!("repair:complete:{}", repair.id().as_uuid()),
        )]);
        rows.push(vec![InlineKeyboardButton::callback(
            "❌ Отменить",
            format!("repair:cancel:{}", repair.id().as_uuid()),
        )]);
    }

    InlineKeyboardMarkup::new(rows)
}

/// Создает клавиатуру возврата/отмены к меню ремонтов.
pub fn back_to_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([super::cancel_row()])
}

/// Создает клавиатуру возврата к записи.
pub fn back_to_booking(booking_id: BookingId) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([
        [InlineKeyboardButton::callback(
            "⬅️ К записи",
            format!("booking:open:{}", booking_id.as_uuid()),
        )]
        .to_vec(),
        super::cancel_row(),
    ])
}

/// Создает клавиатуру подтверждения оплаты.
pub fn payment_confirm(repair_id: RepairId) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "✅ Принять оплату",
            "repair:confirm_payment",
        )],
        vec![InlineKeyboardButton::callback(
            "⬅️ К ремонту",
            format!("repair:open:{}", repair_id.as_uuid()),
        )],
        super::cancel_row(),
    ])
}

/// Создает клавиатуру выбора запчасти для ремонта.
pub fn repair_part_search_results(parts: &[Part], repair_id: RepairId) -> InlineKeyboardMarkup {
    let mut rows = Vec::new();

    if !parts.is_empty() {
        rows.push(
            parts
                .iter()
                .enumerate()
                .map(|(index, part)| {
                    InlineKeyboardButton::callback(
                        (index + 1).to_string(),
                        format!("repair:part_select:{}", part.id().as_uuid()),
                    )
                })
                .collect::<Vec<_>>(),
        );
    }

    rows.push(vec![InlineKeyboardButton::callback(
        "🔍 Найти снова",
        format!("repair:add_part:{}", repair_id.as_uuid()),
    )]);
    rows.push(vec![InlineKeyboardButton::callback(
        "⬅️ К ремонту",
        format!("repair:open:{}", repair_id.as_uuid()),
    )]);
    rows.push(super::cancel_row());

    InlineKeyboardMarkup::new(rows)
}

/// Создает клавиатуру подтверждения списания запчасти в ремонт.
pub fn repair_part_confirm(repair_id: RepairId) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "✅ Добавить в ремонт",
            "repair:confirm_part",
        )],
        vec![InlineKeyboardButton::callback(
            "⬅️ К ремонту",
            format!("repair:open:{}", repair_id.as_uuid()),
        )],
        super::cancel_row(),
    ])
}

/// Создает клавиатуру возврата к ремонту.
pub fn back_to_repair(repair_id: RepairId) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([
        [InlineKeyboardButton::callback(
            "⬅️ К ремонту",
            format!("repair:open:{}", repair_id.as_uuid()),
        )]
        .to_vec(),
        super::cancel_row(),
    ])
}
