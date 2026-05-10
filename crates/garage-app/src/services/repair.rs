use chrono::{DateTime, Utc};
use garage_domain::{
    BookingId, CarId, ClientId, Money, Repair, RepairDescription, RepairId, RepairNotes,
};

use crate::{AppResult, BookingRepository, CarRepository, ClientRepository, RepairRepository};

use super::common::{
    ensure_booking_belongs_to_client_and_car, ensure_car_belongs_to_client, require_booking,
    require_car, require_client, require_repair,
};

/// Command for starting a repair.
///
/// `RepairService` still accepts already validated domain value objects. The
/// command only groups use-case input; it is not an infrastructure DTO.
pub struct StartRepairCommand {
    pub client_id: ClientId,
    pub car_id: CarId,
    pub booking_id: Option<BookingId>,
    pub description: RepairDescription,
    pub labor_price: Money,
    pub parts_price: Money,
    pub parts_cost: Money,
    pub notes: Option<RepairNotes>,
    pub now: DateTime<Utc>,
}

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

    pub async fn start_repair(&self, command: StartRepairCommand) -> AppResult<Repair> {
        require_client(&self.clients, command.client_id).await?;
        let car = require_car(&self.cars, command.car_id).await?;
        ensure_car_belongs_to_client(&car, command.client_id)?;

        if let Some(booking_id) = command.booking_id {
            let booking = require_booking(&self.bookings, booking_id).await?;
            ensure_booking_belongs_to_client_and_car(&booking, command.client_id, command.car_id)?;
        }

        let repair = Repair::new(
            RepairId::new(),
            command.client_id,
            command.car_id,
            command.booking_id,
            command.description,
            command.labor_price,
            command.parts_price,
            command.parts_cost,
            command.notes,
            command.now,
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
