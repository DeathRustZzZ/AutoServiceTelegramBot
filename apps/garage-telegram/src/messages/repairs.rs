use chrono::{DateTime, Duration, Utc};
use garage_app::{BookingDetails, RepairDetails};
use garage_domain::{Car, Money, Repair};

use crate::state::StartRepairDraft;

pub fn menu() -> &'static str {
    "🔧 Ремонты"
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

fn format_money(value: Money) -> String {
    format!(
        "{}.{:02} {}",
        value.amount_minor() / 100,
        value.amount_minor() % 100,
        value.currency()
    )
}

fn format_local_datetime(value: DateTime<Utc>, offset_hours: i32) -> String {
    (value + Duration::hours(i64::from(offset_hours)))
        .format("%d.%m.%Y %H:%M")
        .to_string()
}
