use garage_app::RepairDetails;
use garage_domain::{BookingId, Part, Repair, RepairId};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([
        [InlineKeyboardButton::callback(
            "🔥 Активные ремонты",
            "repair:active",
        )],
        [InlineKeyboardButton::callback(
            "🏠 Главное меню",
            "nav:main",
        )],
    ])
}

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

    rows.push(vec![InlineKeyboardButton::callback(
        "🏠 Главное меню",
        "nav:main",
    )]);

    InlineKeyboardMarkup::new(rows)
}

pub fn active_empty() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
        "🏠 Главное меню",
        "nav:main",
    )]])
}

pub fn start_confirm(booking_id: BookingId) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([
        [InlineKeyboardButton::callback(
            "✅ Начать ремонт",
            "repair:confirm_start",
        )],
        [InlineKeyboardButton::callback(
            "⬅️ К записи",
            format!("booking:open:{}", booking_id.as_uuid()),
        )],
    ])
}

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

    rows.push(vec![InlineKeyboardButton::callback(
        "🔧 Ремонты",
        "nav:repairs",
    )]);
    rows.push(vec![InlineKeyboardButton::callback(
        "🏠 Главное меню",
        "nav:main",
    )]);

    InlineKeyboardMarkup::new(rows)
}

pub fn back_to_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
        "⬅️ К ремонтам",
        "nav:repairs",
    )]])
}

pub fn back_to_booking(booking_id: BookingId) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
        "⬅️ К записи",
        format!("booking:open:{}", booking_id.as_uuid()),
    )]])
}

pub fn payment_confirm(repair_id: RepairId) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([
        [InlineKeyboardButton::callback(
            "✅ Принять оплату",
            "repair:confirm_payment",
        )],
        [InlineKeyboardButton::callback(
            "⬅️ К ремонту",
            format!("repair:open:{}", repair_id.as_uuid()),
        )],
    ])
}

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

    InlineKeyboardMarkup::new(rows)
}

pub fn repair_part_confirm(repair_id: RepairId) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([
        [InlineKeyboardButton::callback(
            "✅ Добавить в ремонт",
            "repair:confirm_part",
        )],
        [InlineKeyboardButton::callback(
            "⬅️ К ремонту",
            format!("repair:open:{}", repair_id.as_uuid()),
        )],
    ])
}

pub fn back_to_repair(repair_id: RepairId) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
        "⬅️ К ремонту",
        format!("repair:open:{}", repair_id.as_uuid()),
    )]])
}
