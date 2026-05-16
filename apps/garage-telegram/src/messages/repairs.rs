use chrono::{DateTime, Duration, Utc};
use garage_app::{BookingDetails, RepairDetails};
use garage_domain::{Car, Part, Repair};

use crate::state::{RecordPaymentDraft, StartRepairDraft, UseRepairPartDraft};

use super::format::{format_byn_input, format_money};

pub fn menu() -> &'static str {
    "🔧 Ремонты. Выберите действие."
}

pub fn active_list(items: &[RepairDetails]) -> String {
    let mut text = "🔧 Активные ремонты".to_string();

    for (index, item) in items.iter().enumerate() {
        text.push_str(&format!(
            "\n\n{}. {} — {} — {}",
            index + 1,
            item.client.name().as_str(),
            car_title(&item.car),
            item.repair.description().as_str()
        ));
    }

    text
}

pub fn active_empty() -> &'static str {
    "✅ Активных ремонтов нет."
}

pub fn ask_description() -> &'static str {
    "Введите описание работ:"
}

pub fn ask_notes() -> &'static str {
    "Введите заметку или отправьте -:"
}

pub fn confirm_start(
    details: &BookingDetails,
    draft: &StartRepairDraft,
    offset_hours: i32,
) -> String {
    format!(
        "Проверьте ремонт:\n\nКлиент: {}\nАвто: {}\nЗапись: {}\nОписание: {}\nЗаметка: {}",
        details.client.name().as_str(),
        car_title(&details.car),
        format_local_datetime(*details.booking.scheduled_at(), offset_hours),
        draft.description.as_deref().unwrap_or("не указано"),
        draft.notes.as_deref().unwrap_or("нет")
    )
}

pub fn repair_card(details: &RepairDetails) -> String {
    repair_card_with_title("🔧 Ремонт", details)
}

pub fn repair_created_card(details: &RepairDetails) -> String {
    repair_card_with_title("Ремонт создан\n\n🔧 Ремонт", details)
}

pub fn missing_draft() -> &'static str {
    "Данные ремонта устарели. Откройте запись и начните ремонт заново."
}

pub fn ask_payment_amount() -> &'static str {
    "Введите сумму оплаты в BYN. Например: 50 или 50.50"
}

pub fn ask_labor_price() -> &'static str {
    "Введите стоимость работ в BYN. Например: 100 или 100.50"
}

pub fn labor_price_updated_card(details: &RepairDetails) -> String {
    repair_card_with_title("Стоимость работ обновлена\n\n🔧 Ремонт", details)
}

pub fn ask_payment_method() -> &'static str {
    "Введите способ оплаты: cash, card, transfer, crypto или other."
}

pub fn invalid_payment_method() -> &'static str {
    "Неизвестный способ оплаты. Используйте: cash, card, transfer, crypto или other."
}

pub fn ask_payment_comment() -> &'static str {
    "Введите комментарий или отправьте -:"
}

pub fn confirm_payment(details: &RepairDetails, draft: &RecordPaymentDraft) -> String {
    format!(
        "Проверьте оплату:\n\nРемонт: {} — {}\nСумма: {}\nМетод: {}\nКомментарий: {}",
        car_title(&details.car),
        details.client.name().as_str(),
        draft
            .amount
            .as_deref()
            .and_then(format_byn_input)
            .unwrap_or_else(|| "не указана".to_string()),
        payment_method_title(draft.method.as_deref().unwrap_or("не указан")),
        draft.comment.as_deref().unwrap_or("нет")
    )
}

pub fn payment_recorded_card(details: &RepairDetails) -> String {
    repair_card_with_title("Оплата принята\n\n🔧 Ремонт", details)
}

pub fn ask_repair_part_query() -> &'static str {
    "Введите название или SKU запчасти:"
}

pub fn repair_part_search_results(query: &str, parts: &[Part]) -> String {
    let mut text = format!("Выберите запчасть для ремонта: {query}");

    for (index, part) in parts.iter().enumerate() {
        text.push_str(&format!(
            "\n\n{}. {} — SKU {} — остаток {} шт — цена {}",
            index + 1,
            part.name().as_str(),
            part.sku().map(|sku| sku.as_str()).unwrap_or("нет"),
            part.quantity().value(),
            format_money(part.unit_price())
        ));
    }

    text
}

pub fn no_repair_part_results(query: &str) -> String {
    format!("По запросу `{query}` запчасти не найдены.")
}

pub fn ask_repair_part_quantity() -> &'static str {
    "Введите количество:"
}

pub fn ask_repair_part_unit_price() -> &'static str {
    "Введите цену продажи за единицу в BYN. Например: 25 или 25.50"
}

pub fn ask_repair_part_comment() -> &'static str {
    "Введите комментарий или отправьте -:"
}

pub fn confirm_repair_part(
    details: &RepairDetails,
    part: &Part,
    draft: &UseRepairPartDraft,
) -> String {
    format!(
        "Проверьте запчасть для ремонта:\n\nРемонт: {} — {}\nЗапчасть: {}\nКоличество: {} шт\nОстаток сейчас: {} шт\nЦена продажи: {}\nКомментарий: {}",
        car_title(&details.car),
        details.client.name().as_str(),
        part.name().as_str(),
        draft.quantity.as_deref().unwrap_or("не указано"),
        part.quantity().value(),
        draft
            .unit_price
            .as_deref()
            .and_then(format_byn_input)
            .unwrap_or_else(|| "не указана".to_string()),
        draft.comment.as_deref().unwrap_or("нет")
    )
}

pub fn repair_part_added_card(details: &RepairDetails, result_message: Option<&str>) -> String {
    match result_message {
        Some(message) => format!("{message}\n\n{}", repair_card(details)),
        None => repair_card_with_title("Запчасть добавлена\n\n🔧 Ремонт", details),
    }
}

pub fn invalid_money() -> &'static str {
    "Введите сумму в BYN. Например: 50 или 50.50"
}

pub fn money_must_be_positive() -> &'static str {
    "Сумма должна быть больше 0."
}

pub fn invalid_quantity() -> &'static str {
    "Введите количество числом, например 1."
}

fn repair_card_with_title(title: &str, details: &RepairDetails) -> String {
    let repair = &details.repair;
    format!(
        "{title}\n\nСтатус: {}\nКлиент: {}\nАвто: {}\nОписание: {}\nЗаметка: {}\nРаботы: {}\nЗапчасти: {}\nИтого: {}\nОплачено: {}\nОстаток: {}",
        repair.status(),
        details.client.name().as_str(),
        car_title(&details.car),
        repair.description().as_str(),
        repair.notes().map(|notes| notes.as_str()).unwrap_or("нет"),
        format_money(repair.labor_price()),
        format_money(repair.parts_price()),
        format_repair_money(repair, RepairMoneyKind::Total),
        format_money(repair.paid_amount()),
        format_repair_money(repair, RepairMoneyKind::Remaining)
    )
}

enum RepairMoneyKind {
    Total,
    Remaining,
}

fn format_repair_money(repair: &Repair, kind: RepairMoneyKind) -> String {
    let result = match kind {
        RepairMoneyKind::Total => repair.total_price(),
        RepairMoneyKind::Remaining => repair.remaining_amount(),
    };

    result
        .map(format_money)
        .unwrap_or_else(|_| "не удалось рассчитать".to_string())
}

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

fn payment_method_title(value: &str) -> &'static str {
    match value {
        "cash" => "Наличные",
        "card" => "Карта",
        "transfer" | "bank_transfer" => "Перевод",
        "crypto" => "Crypto",
        "other" => "Другое",
        _ => "не указан",
    }
}

fn format_local_datetime(value: DateTime<Utc>, offset_hours: i32) -> String {
    (value + Duration::hours(i64::from(offset_hours)))
        .format("%d.%m.%Y %H:%M")
        .to_string()
}
