//! Преобразование SQL row-моделей в доменные сущности.

pub mod booking;
pub mod car;
pub mod client;
pub mod part;
pub mod part_supply;
pub mod payment;
pub mod repair;
pub mod repair_part;
pub mod stock_movement;

use garage_app::AppError;
use garage_domain::{
    BookingStatus, ClientStatus, Currency, Money, PartQuantity, PartStatus, PartSupplyStatus,
    PaymentMethod, RepairStatus, StockMovementReason, StockMovementType,
};

pub(super) fn invalid_row_error(
    entity: &'static str,
    field: &'static str,
    value: impl ToString,
) -> AppError {
    AppError::Repository {
        operation: "map row to domain",
        message: format!("invalid {entity}.{field}: {}", value.to_string()),
    }
}

pub(super) fn map_client_status(value: &str) -> Result<ClientStatus, AppError> {
    match value {
        "active" => Ok(ClientStatus::Active),
        "archived" => Ok(ClientStatus::Archived),
        _ => Err(invalid_row_error("client", "status", value)),
    }
}

pub(super) fn map_car_status(value: &str) -> Result<garage_domain::CarStatus, AppError> {
    match value {
        "active" => Ok(garage_domain::CarStatus::Active),
        "archived" => Ok(garage_domain::CarStatus::Archived),
        _ => Err(invalid_row_error("car", "status", value)),
    }
}

pub(super) fn map_booking_status(value: &str) -> Result<BookingStatus, AppError> {
    match value {
        "scheduled" => Ok(BookingStatus::Scheduled),
        "completed" => Ok(BookingStatus::Completed),
        "cancelled" => Ok(BookingStatus::Cancelled),
        "no_show" => Ok(BookingStatus::NoShow),
        _ => Err(invalid_row_error("booking", "status", value)),
    }
}

pub(super) fn map_part_status(value: &str) -> Result<PartStatus, AppError> {
    match value {
        "active" => Ok(PartStatus::Active),
        "archived" => Ok(PartStatus::Archived),
        _ => Err(invalid_row_error("part", "status", value)),
    }
}

pub(super) fn map_part_supply_status(value: &str) -> Result<PartSupplyStatus, AppError> {
    match value {
        "expected" => Ok(PartSupplyStatus::Expected),
        "received" => Ok(PartSupplyStatus::Received),
        "cancelled" => Ok(PartSupplyStatus::Cancelled),
        _ => Err(invalid_row_error("part_supply", "status", value)),
    }
}

pub(super) fn map_repair_status(value: &str) -> Result<RepairStatus, AppError> {
    match value {
        "in_progress" => Ok(RepairStatus::InProgress),
        "completed" => Ok(RepairStatus::Completed),
        "cancelled" => Ok(RepairStatus::Cancelled),
        _ => Err(invalid_row_error("repair", "status", value)),
    }
}

pub(super) fn map_payment_method(value: &str) -> Result<PaymentMethod, AppError> {
    match value {
        "cash" => Ok(PaymentMethod::Cash),
        "card" => Ok(PaymentMethod::Card),
        "bank_transfer" => Ok(PaymentMethod::BankTransfer),
        "crypto" => Ok(PaymentMethod::Crypto),
        "other" => Ok(PaymentMethod::Other),
        _ => Err(invalid_row_error("payment", "method", value)),
    }
}

pub(super) fn map_stock_movement_type(value: &str) -> Result<StockMovementType, AppError> {
    match value {
        "in" => Ok(StockMovementType::In),
        "out" => Ok(StockMovementType::Out),
        "adjustment" => Ok(StockMovementType::Adjustment),
        _ => Err(invalid_row_error("stock_movement", "movement_type", value)),
    }
}

pub(super) fn map_stock_movement_reason(value: &str) -> Result<StockMovementReason, AppError> {
    match value {
        "supply" => Ok(StockMovementReason::Supply),
        "repair_usage" => Ok(StockMovementReason::RepairUsage),
        "return_from_repair" => Ok(StockMovementReason::ReturnFromRepair),
        "inventory_correction" => Ok(StockMovementReason::InventoryCorrection),
        "manual_correction" => Ok(StockMovementReason::ManualCorrection),
        "other" => Ok(StockMovementReason::Other),
        _ => Err(invalid_row_error("stock_movement", "reason", value)),
    }
}

pub(super) fn map_currency(entity: &'static str, value: &str) -> Result<Currency, AppError> {
    match value {
        "BYN" => Ok(Currency::Byn),
        "USD" => Ok(Currency::Usd),
        _ => Err(invalid_row_error(entity, "currency", value)),
    }
}

pub(super) fn map_money(
    entity: &'static str,
    field: &'static str,
    amount_minor: i64,
    currency: Currency,
) -> Result<Money, AppError> {
    Money::new(amount_minor, currency)
        .map_err(|error| invalid_row_error(entity, field, error.to_string()))
}

pub(super) fn map_quantity(
    entity: &'static str,
    field: &'static str,
    value: i32,
) -> Result<PartQuantity, AppError> {
    let value = u32::try_from(value).map_err(|_| invalid_row_error(entity, field, value))?;
    Ok(PartQuantity::new(value))
}
