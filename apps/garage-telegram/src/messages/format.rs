//! Общие formatter'ы значений для сообщений.
//!
//! Форматирование денег централизовано, чтобы карточки ремонтов, склада и
//! подтверждения форм не расходились по виду BYN-сумм.

use garage_domain::{Currency, Money};

use crate::ui::money_input::parse_byn_amount;

/// Форматирует доменную сумму как `major.minor CURRENCY`.
pub fn format_money(value: Money) -> String {
    format_minor(value.amount_minor(), value.currency())
}

/// Форматирует пользовательский ввод суммы, если он успешно разбирается как BYN.
pub fn format_byn_input(input: &str) -> Option<String> {
    parse_byn_amount(input).ok().map(format_money)
}

/// Форматирует minor units с указанной валютой.
fn format_minor(value: i64, currency: Currency) -> String {
    let sign = if value < 0 { "-" } else { "" };
    let abs = value.abs();
    format!("{sign}{}.{:02} {currency}", abs / 100, abs % 100)
}
