use chrono::{DateTime, Utc};
use garage_domain::{
    BookingId, CarId, ClientId, Money, Repair, RepairDescription, RepairId, RepairNotes,
};

use crate::{AppResult, BookingRepository, CarRepository, ClientRepository, RepairRepository};

use super::common::{
    ensure_booking_belongs_to_client_and_car, ensure_car_belongs_to_client, require_booking,
    require_car, require_client, require_repair,
};

/// Use cases for repairs.
pub struct RepairService<Clients, Cars, Bookings, Repairs> {
    clients: Clients,
    cars: Cars,
    bookings: Bookings,
    repairs: Repairs,
}

impl<Clients, Cars, Bookings, Repairs> RepairService<Clients, Cars, Bookings, Repairs>
where
    Clients: ClientRepository,
    Cars: CarRepository,
    Bookings: BookingRepository,
    Repairs: RepairRepository,
{
    pub fn new(clients: Clients, cars: Cars, bookings: Bookings, repairs: Repairs) -> Self {
        Self {
            clients,
            cars,
            bookings,
            repairs,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn start_repair(
        &self,
        client_id: ClientId,
        car_id: CarId,
        booking_id: Option<BookingId>,
        description: RepairDescription,
        labor_price: Money,
        parts_price: Money,
        parts_cost: Money,
        notes: Option<RepairNotes>,
        now: DateTime<Utc>,
    ) -> AppResult<Repair> {
        require_client(&self.clients, client_id).await?;
        let car = require_car(&self.cars, car_id).await?;
        ensure_car_belongs_to_client(&car, client_id)?;

        if let Some(booking_id) = booking_id {
            let booking = require_booking(&self.bookings, booking_id).await?;
            ensure_booking_belongs_to_client_and_car(&booking, client_id, car_id)?;
        }

        let repair = Repair::new(
            RepairId::new(),
            client_id,
            car_id,
            booking_id,
            description,
            labor_price,
            parts_price,
            parts_cost,
            notes,
            now,
        )?;
        self.repairs.save(&repair).await?;
        Ok(repair)
    }

    pub async fn record_payment(
        &self,
        repair_id: RepairId,
        amount: Money,
        now: DateTime<Utc>,
    ) -> AppResult<Repair> {
        let mut repair = require_repair(&self.repairs, repair_id).await?;
        repair.record_payment(amount, now)?;
        self.repairs.save(&repair).await?;
        Ok(repair)
    }

    pub async fn complete_repair(
        &self,
        repair_id: RepairId,
        completed_at: DateTime<Utc>,
    ) -> AppResult<Repair> {
        let mut repair = require_repair(&self.repairs, repair_id).await?;
        repair.complete(completed_at)?;
        self.repairs.save(&repair).await?;
        Ok(repair)
    }

    pub async fn cancel_repair(
        &self,
        repair_id: RepairId,
        now: DateTime<Utc>,
    ) -> AppResult<Repair> {
        let mut repair = require_repair(&self.repairs, repair_id).await?;
        repair.cancel(now)?;
        self.repairs.save(&repair).await?;
        Ok(repair)
    }
}
