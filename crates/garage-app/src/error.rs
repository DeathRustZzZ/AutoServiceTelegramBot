use garage_domain::{
    BookingError, BookingId, CarError, CarId, ClientError, ClientId, MoneyError, PartError, PartId,
    PartSupplyError, PartSupplyId, PhoneNumberError, RepairError, RepairId,
};
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("client not found: {0:?}")]
    ClientNotFound(ClientId),
    #[error("car not found: {0:?}")]
    CarNotFound(CarId),
    #[error("booking not found: {0:?}")]
    BookingNotFound(BookingId),
    #[error("part not found: {0:?}")]
    PartNotFound(PartId),
    #[error("part supply not found: {0:?}")]
    PartSupplyNotFound(PartSupplyId),
    #[error("repair not found: {0:?}")]
    RepairNotFound(RepairId),

    #[error("car {car_id:?} does not belong to client {client_id:?}")]
    CarDoesNotBelongToClient { car_id: CarId, client_id: ClientId },
    #[error("booking {booking_id:?} does not belong to client {client_id:?}")]
    BookingDoesNotBelongToClient {
        booking_id: BookingId,
        client_id: ClientId,
    },
    #[error("booking {booking_id:?} does not belong to car {car_id:?}")]
    BookingDoesNotBelongToCar {
        booking_id: BookingId,
        car_id: CarId,
    },

    #[error("repository error: {0}")]
    Repository(String),

    #[error(transparent)]
    Client(#[from] ClientError),
    #[error(transparent)]
    Car(#[from] CarError),
    #[error(transparent)]
    Booking(#[from] BookingError),
    #[error(transparent)]
    Part(#[from] PartError),
    #[error(transparent)]
    PartSupply(#[from] PartSupplyError),
    #[error(transparent)]
    Repair(#[from] RepairError),
    #[error(transparent)]
    Money(#[from] MoneyError),
    #[error(transparent)]
    PhoneNumber(#[from] PhoneNumberError),
}
