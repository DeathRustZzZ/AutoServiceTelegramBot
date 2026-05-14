use garage_domain::Money;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoneyInputError {
    Empty,
    Negative,
    InvalidFormat,
    TooManyFractionDigits,
    Domain,
}

pub fn parse_byn_amount(input: &str) -> Result<Money, MoneyInputError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(MoneyInputError::Empty);
    }
    if input.starts_with('-') {
        return Err(MoneyInputError::Negative);
    }

    let normalized = input.replace(',', ".");
    let parts = normalized.split('.').collect::<Vec<_>>();
    if parts.len() > 2 {
        return Err(MoneyInputError::InvalidFormat);
    }

    let major = parts[0];
    if major.is_empty() || !major.chars().all(|value| value.is_ascii_digit()) {
        return Err(MoneyInputError::InvalidFormat);
    }

    let minor = match parts.get(1).copied() {
        None | Some("") => 0,
        Some(value) if value.len() > 2 => return Err(MoneyInputError::TooManyFractionDigits),
        Some(value) if value.chars().all(|value| value.is_ascii_digit()) => {
            let parsed = value
                .parse::<i64>()
                .map_err(|_| MoneyInputError::InvalidFormat)?;
            if value.len() == 1 {
                parsed * 10
            } else {
                parsed
            }
        }
        Some(_) => return Err(MoneyInputError::InvalidFormat),
    };

    let major = major
        .parse::<i64>()
        .map_err(|_| MoneyInputError::InvalidFormat)?;
    let total_minor = major
        .checked_mul(100)
        .and_then(|value| value.checked_add(minor))
        .ok_or(MoneyInputError::Domain)?;

    Money::byn_minor(total_minor).map_err(|_| MoneyInputError::Domain)
}

pub fn ensure_positive_money(money: Money) -> Result<Money, MoneyInputError> {
    if money.amount_minor() > 0 {
        Ok(money)
    } else {
        Err(MoneyInputError::Domain)
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_byn_amount, MoneyInputError};

    #[test]
    fn parse_byn_amount_accepts_human_byn_values() {
        let cases = [
            ("25", 2500),
            ("25.5", 2550),
            ("25.50", 2550),
            ("25,50", 2550),
            ("0", 0),
            ("0.00", 0),
            (" 25.50 ", 2550),
            ("25.", 2500),
        ];

        for (input, expected_minor) in cases {
            let money = parse_byn_amount(input).unwrap();
            assert_eq!(money.amount_minor(), expected_minor, "{input}");
        }
    }

    #[test]
    fn parse_byn_amount_rejects_invalid_values() {
        let cases = [
            ("", MoneyInputError::Empty),
            ("abc", MoneyInputError::InvalidFormat),
            ("-1", MoneyInputError::Negative),
            ("25.005", MoneyInputError::TooManyFractionDigits),
            ("25,005", MoneyInputError::TooManyFractionDigits),
            ("12.3.4", MoneyInputError::InvalidFormat),
            (".50", MoneyInputError::InvalidFormat),
        ];

        for (input, expected_error) in cases {
            assert_eq!(parse_byn_amount(input), Err(expected_error), "{input}");
        }
    }
}
