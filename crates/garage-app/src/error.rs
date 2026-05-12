//! Ошибки прикладного слоя.
//!
//! В domain-ошибках описаны нарушения инвариантов внутри конкретной сущности:
//! невалидный VIN, платеж больше суммы ремонта, попытка изменить финальный
//! booking. В application layer появляются другие классы ошибок:
//! - агрегат не найден в репозитории;
//! - связь между агрегатами не совпала;
//! - инфраструктурная реализация репозитория не смогла выполнить операцию.
//!
//! Мы не прячем domain errors за строками. Они пробрасываются как transparent
//! variants, чтобы обработчик Telegram или будущий API мог отличать
//! пользовательскую ошибку ввода от ошибки хранения.

use garage_domain::{
    BookingError, BookingId, CarError, CarId, ClientError, ClientId, MoneyError, PartError, PartId,
    PartSupplyError, PartSupplyId, PaymentError, PaymentId, PhoneNumberError, RepairError,
    RepairId, RepairPartError, RepairPartId, StockMovementError, StockMovementId,
};
use thiserror::Error;

/// Единый результат прикладного слоя.
///
/// Такой alias делает сигнатуры сервисов и портов короче, но не скрывает тип
/// ошибки: вся публичная поверхность app-layer возвращает `AppError`.
pub type AppResult<T> = Result<T, AppError>;

/// Ошибка use case или repository port.
///
/// Этот enum намеренно живет в `garage-app`, а не в `garage-domain`:
/// отсутствие записи в БД, несовпадение связей между агрегатами и сбои
/// репозитория не являются свойствами одной доменной сущности.
#[derive(Debug, Error)]
pub enum AppError {
    /// Клиент не найден по переданному идентификатору.
    #[error("client not found: {0:?}")]
    ClientNotFound(ClientId),
    /// Автомобиль не найден по переданному идентификатору.
    #[error("car not found: {0:?}")]
    CarNotFound(CarId),
    /// Запись на обслуживание не найдена.
    #[error("booking not found: {0:?}")]
    BookingNotFound(BookingId),
    /// Складская позиция не найдена.
    #[error("part not found: {0:?}")]
    PartNotFound(PartId),
    /// Поставка запчасти не найдена.
    #[error("part supply not found: {0:?}")]
    PartSupplyNotFound(PartSupplyId),
    /// Ремонт не найден.
    #[error("repair not found: {0:?}")]
    RepairNotFound(RepairId),
    /// Оплата не найдена.
    #[error("payment not found: {0:?}")]
    PaymentNotFound(PaymentId),
    /// Строка использованной в ремонте запчасти не найдена.
    #[error("repair part not found: {0:?}")]
    RepairPartNotFound(RepairPartId),
    /// Движение склада не найдено.
    #[error("stock movement not found: {0:?}")]
    StockMovementNotFound(StockMovementId),

    /// Автомобиль существует, но принадлежит другому клиенту.
    ///
    /// Это app-layer инвариант: `Car` знает только свой `client_id`, но сценарий
    /// должен убедиться, что пользователь не создает booking или repair для
    /// чужой связки `client_id + car_id`.
    #[error("car {car_id:?} does not belong to client {client_id:?}")]
    CarDoesNotBelongToClient { car_id: CarId, client_id: ClientId },
    /// Booking существует, но относится к другому клиенту.
    #[error("booking {booking_id:?} does not belong to client {client_id:?}")]
    BookingDoesNotBelongToClient {
        booking_id: BookingId,
        client_id: ClientId,
    },
    /// Booking существует, но относится к другому автомобилю.
    #[error("booking {booking_id:?} does not belong to car {car_id:?}")]
    BookingDoesNotBelongToCar {
        booking_id: BookingId,
        car_id: CarId,
    },
    /// Строка использованной запчасти относится к другому ремонту.
    #[error("repair part {repair_part_id:?} does not belong to repair {repair_id:?}")]
    RepairPartDoesNotBelongToRepair {
        repair_part_id: RepairPartId,
        repair_id: RepairId,
    },
    /// Движение склада относится к другой складской позиции.
    #[error("stock movement {stock_movement_id:?} does not belong to part {part_id:?}")]
    StockMovementDoesNotBelongToPart {
        stock_movement_id: StockMovementId,
        part_id: PartId,
    },
    /// Нельзя списывать запчасти в отмененный ремонт.
    #[error("cannot use part for cancelled repair {repair_id:?}")]
    CannotUsePartForCancelledRepair { repair_id: RepairId },

    /// Ошибка реализации репозитория.
    ///
    /// Конкретная инфраструктура позже сможет завернуть сюда SQLx/PostgreSQL
    /// ошибку, нарушение уникальности или проблему соединения. App-layer не
    /// должен зависеть от этих типов напрямую.
    #[error("repository error during {operation}: {message}")]
    Repository {
        operation: &'static str,
        message: String,
    },

    /// Ошибка домена клиента.
    #[error(transparent)]
    Client(#[from] ClientError),
    /// Ошибка домена автомобиля.
    #[error(transparent)]
    Car(#[from] CarError),
    /// Ошибка домена записи.
    #[error(transparent)]
    Booking(#[from] BookingError),
    /// Ошибка домена складской позиции.
    #[error(transparent)]
    Part(#[from] PartError),
    /// Ошибка домена поставки.
    #[error(transparent)]
    PartSupply(#[from] PartSupplyError),
    /// Ошибка домена ремонта.
    #[error(transparent)]
    Repair(#[from] RepairError),
    /// Ошибка домена оплаты.
    #[error(transparent)]
    Payment(#[from] PaymentError),
    /// Ошибка домена использованной в ремонте запчасти.
    #[error(transparent)]
    RepairPart(#[from] RepairPartError),
    /// Ошибка домена движения склада.
    #[error(transparent)]
    StockMovement(#[from] StockMovementError),
    /// Ошибка денежных вычислений.
    #[error(transparent)]
    Money(#[from] MoneyError),
    /// Ошибка нормализации телефона.
    #[error(transparent)]
    PhoneNumber(#[from] PhoneNumberError),
}
