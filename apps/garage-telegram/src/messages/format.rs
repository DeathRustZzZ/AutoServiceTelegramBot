use garage_domain::{Currency, Money};

use crate::ui::money_input::parse_byn_amount;

pub fn format_money(value: Money) -> String {
    format_minor(value.amount_minor(), value.currency())
}

pub fn format_byn_input(input: &str) -> Option<String> {
    parse_byn_amount(input).ok().map(format_money)
}

fn format_minor(value: i64, currency: Currency) -> String {
    let sign = if value < 0 { "-" } else { "" };
    let abs = value.abs();
    format!("{sign}{}.{:02} {currency}", abs / 100, abs % 100)
}
