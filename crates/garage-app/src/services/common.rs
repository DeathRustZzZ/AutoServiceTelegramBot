use garage_domain::{
    Booking, BookingId, Car, CarId, Client, ClientId, Part, PartId, PartSupply, PartSupplyId,
    Repair, RepairId,
};

use crate::{
    AppError, AppResult, BookingRepository, CarRepository, ClientRepository, PartRepository,
    PartSupplyRepository, RepairRepository,
};

pub(crate) async fn require_client<R>(clients: &R, client_id: ClientId) -> AppResult<Client>
where
    R: ClientRepository,
{
    clients
        .get(client_id)
        .await?
        .ok_or(AppError::ClientNotFound(client_id))
}

pub(crate) async fn require_car<R>(cars: &R, car_id: CarId) -> AppResult<Car>
where
    R: CarRepository,
{
    cars.get(car_id).await?.ok_or(AppError::CarNotFound(car_id))
}

pub(crate) async fn require_booking<R>(bookings: &R, booking_id: BookingId) -> AppResult<Booking>
where
    R: BookingRepository,
{
    bookings
        .get(booking_id)
        .await?
        .ok_or(AppError::BookingNotFound(booking_id))
}

pub(crate) async fn require_part<R>(parts: &R, part_id: PartId) -> AppResult<Part>
where
    R: PartRepository,
{
    parts
        .get(part_id)
        .await?
        .ok_or(AppError::PartNotFound(part_id))
}

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

pub(crate) async fn require_repair<R>(repairs: &R, repair_id: RepairId) -> AppResult<Repair>
where
    R: RepairRepository,
{
    repairs
        .get(repair_id)
        .await?
        .ok_or(AppError::RepairNotFound(repair_id))
}

pub(crate) fn ensure_car_belongs_to_client(car: &Car, client_id: ClientId) -> AppResult<()> {
    if car.client_id() != client_id {
        return Err(AppError::CarDoesNotBelongToClient {
            car_id: car.id(),
            client_id,
        });
    }

    Ok(())
}

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
