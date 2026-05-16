//! Общие guard-функции прикладного слоя.
//!
//! Эти функции не являются доменной логикой. Они превращают `Option<T>` из
//! repository port в точные `AppError::*NotFound` и проверяют связи между
//! агрегатами. Домен не может делать такие проверки сам, потому что у сущностей
//! нет доступа к репозиториям.

use garage_domain::{
    Booking, BookingId, Car, CarId, Client, ClientId, Part, PartId, PartSupply, PartSupplyId,
    Payment, PaymentId, Repair, RepairId, RepairPart, RepairPartId, StockMovement, StockMovementId,
};

use crate::{
    AppError, AppResult, BookingRepository, CarRepository, ClientRepository, PartRepository,
    PartSupplyRepository, PaymentRepository, RepairPartRepository, RepairRepository,
    StockMovementRepository,
};

/// Загружает клиента или возвращает `ClientNotFound`.
pub(crate) async fn require_client<R>(clients: &R, client_id: ClientId) -> AppResult<Client>
where
    R: ClientRepository,
{
    clients
        .get(client_id)
        .await?
        .ok_or(AppError::ClientNotFound(client_id))
}

/// Загружает автомобиль или возвращает `CarNotFound`.
pub(crate) async fn require_car<R>(cars: &R, car_id: CarId) -> AppResult<Car>
where
    R: CarRepository,
{
    cars.get(car_id).await?.ok_or(AppError::CarNotFound(car_id))
}

/// Загружает запись или возвращает `BookingNotFound`.
pub(crate) async fn require_booking<R>(bookings: &R, booking_id: BookingId) -> AppResult<Booking>
where
    R: BookingRepository,
{
    bookings
        .get(booking_id)
        .await?
        .ok_or(AppError::BookingNotFound(booking_id))
}

/// Загружает складскую позицию или возвращает `PartNotFound`.
pub(crate) async fn require_part<R>(parts: &R, part_id: PartId) -> AppResult<Part>
where
    R: PartRepository,
{
    parts
        .get(part_id)
        .await?
        .ok_or(AppError::PartNotFound(part_id))
}

/// Загружает поставку или возвращает `PartSupplyNotFound`.
pub(crate) async fn require_supply<R>(
    supplies: &R,
    supply_id: PartSupplyId,
) -> AppResult<PartSupply>
where
    R: PartSupplyRepository,
{
    supplies
        .get(supply_id)
        .await?
        .ok_or(AppError::PartSupplyNotFound(supply_id))
}

/// Загружает ремонт или возвращает `RepairNotFound`.
pub(crate) async fn require_repair<R>(repairs: &R, repair_id: RepairId) -> AppResult<Repair>
where
    R: RepairRepository,
{
    repairs
        .get(repair_id)
        .await?
        .ok_or(AppError::RepairNotFound(repair_id))
}

/// Загружает оплату или возвращает `PaymentNotFound`.
#[allow(dead_code)]
pub(crate) async fn require_payment<R>(payments: &R, payment_id: PaymentId) -> AppResult<Payment>
where
    R: PaymentRepository,
{
    payments
        .get(payment_id)
        .await?
        .ok_or(AppError::PaymentNotFound(payment_id))
}

/// Загружает строку использованной запчасти или возвращает `RepairPartNotFound`.
#[allow(dead_code)]
pub(crate) async fn require_repair_part<R>(
    repair_parts: &R,
    repair_part_id: RepairPartId,
) -> AppResult<RepairPart>
where
    R: RepairPartRepository,
{
    repair_parts
        .get(repair_part_id)
        .await?
        .ok_or(AppError::RepairPartNotFound(repair_part_id))
}

/// Загружает движение склада или возвращает `StockMovementNotFound`.
#[allow(dead_code)]
pub(crate) async fn require_stock_movement<R>(
    stock_movements: &R,
    stock_movement_id: StockMovementId,
) -> AppResult<StockMovement>
where
    R: StockMovementRepository,
{
    stock_movements
        .get(stock_movement_id)
        .await?
        .ok_or(AppError::StockMovementNotFound(stock_movement_id))
}

/// Проверяет, что автомобиль принадлежит указанному клиенту.
///
/// Это защита от операций вида "изменить машину клиента A, передав car_id
/// машины клиента B". Проверка должна выполняться до любой мутации `Car`.
pub(crate) fn ensure_car_belongs_to_client(car: &Car, client_id: ClientId) -> AppResult<()> {
    if car.client_id() != client_id {
        return Err(AppError::CarDoesNotBelongToClient {
            car_id: car.id(),
            client_id,
        });
    }

    Ok(())
}

/// Проверяет, что клиент доступен для новых активных сценариев.
pub(crate) fn ensure_client_active(client: &Client) -> AppResult<()> {
    if client.is_archived() {
        return Err(AppError::ClientArchived(client.id()));
    }

    Ok(())
}

/// Проверяет, что автомобиль доступен для новых активных сценариев.
pub(crate) fn ensure_car_active(car: &Car) -> AppResult<()> {
    if car.is_archived() {
        return Err(AppError::CarArchived(car.id()));
    }

    Ok(())
}

/// Проверяет, что складская позиция доступна для новых активных сценариев.
pub(crate) fn ensure_part_active(part: &Part) -> AppResult<()> {
    if part.is_archived() {
        return Err(AppError::PartArchived(part.id()));
    }

    Ok(())
}

/// Проверяет, что строка использованной запчасти относится к указанному ремонту.
#[allow(dead_code)]
pub(crate) fn ensure_repair_part_belongs_to_repair(
    repair_part: &RepairPart,
    repair_id: RepairId,
) -> AppResult<()> {
    if repair_part.repair_id() != repair_id {
        return Err(AppError::RepairPartDoesNotBelongToRepair {
            repair_part_id: repair_part.id(),
            repair_id,
        });
    }

    Ok(())
}

/// Проверяет, что движение склада относится к указанной складской позиции.
#[allow(dead_code)]
pub(crate) fn ensure_stock_movement_belongs_to_part(
    movement: &StockMovement,
    part_id: PartId,
) -> AppResult<()> {
    if movement.part_id() != part_id {
        return Err(AppError::StockMovementDoesNotBelongToPart {
            stock_movement_id: movement.id(),
            part_id,
        });
    }

    Ok(())
}

/// Проверяет, что booking относится к той же паре клиента и автомобиля.
///
/// Нужна при старте ремонта из записи и при сборке read model'ей. Без этой
/// проверки поврежденные данные хранилища могли бы связать ремонт или карточку
/// записи с чужим автомобилем.
pub(crate) fn ensure_booking_belongs_to_client_and_car(
    booking: &Booking,
    client_id: ClientId,
    car_id: CarId,
) -> AppResult<()> {
    if booking.client_id() != client_id {
        return Err(AppError::BookingDoesNotBelongToClient {
            booking_id: booking.id(),
            client_id,
        });
    }

    if booking.car_id() != car_id {
        return Err(AppError::BookingDoesNotBelongToCar {
            booking_id: booking.id(),
            car_id,
        });
    }

    Ok(())
}
