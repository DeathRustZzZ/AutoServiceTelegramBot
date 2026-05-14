use garage_domain::{Currency, Money};

pub fn format_money(value: Money) -> String {
    format_minor(value.amount_minor(), value.currency())
}

pub fn format_minor_byn(value: i64) -> String {
    format_minor(value, Currency::Byn)
}

fn format_minor(value: i64, currency: Currency) -> String {
    let sign = if value < 0 { "-" } else { "" };
    let abs = value.abs();
    format!("{sign}{}.{:02} {currency}", abs / 100, abs % 100)
}
