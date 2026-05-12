//! PostgreSQL-реализации repository ports из application layer.

pub mod booking;
pub mod car;
pub mod client;
pub mod part;
pub mod payment;
pub mod repair;

use garage_app::AppError;
use garage_domain::{Currency, PartQuantity};

pub(super) fn repository_error(operation: &'static str, error: impl ToString) -> AppError {
    AppError::Repository {
        operation,
        message: error.to_string(),
    }
}

pub(super) fn currency_code(currency: Currency) -> &'static str {
    match currency {
        Currency::Byn => "BYN",
        Currency::Usd => "USD",
    }
}

pub(super) fn quantity_to_i32(
    operation: &'static str,
    field: &'static str,
    quantity: PartQuantity,
) -> Result<i32, AppError> {
    i32::try_from(quantity.value())
        .map_err(|_| repository_error(operation, format!("{field} does not fit into i32")))
}
