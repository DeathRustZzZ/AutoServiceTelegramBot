use chrono::{DateTime, Utc};
use garage_domain::{
    Booking, BookingId, BookingNotes, BookingReason, Car, CarId, CarMake, CarModel, CarNotes,
    CarYear, Client, ClientId, ClientName, ClientNotes, Currency, LicensePlate, Money, Part,
    PartId, PartName, PartNotes, PartQuantity, PartSku, PartSupplier, PartSupply, PartSupplyId,
    PartSupplyNotes, PhoneNumber, Repair, RepairDescription, RepairId, RepairNotes, SignedMoney,
    Vin,
};

use crate::{
    AppError, AppResult, BookingRepository, CarRepository, ClientRepository, PartRepository,
    PartSupplyRepository, RepairRepository,
};

pub struct ClientService<R> {
    clients: R,
}

impl<R> ClientService<R>
where
    R: ClientRepository,
{
    pub fn new(clients: R) -> Self {
        Self { clients }
    }

    pub async fn create_client(
        &self,
        name: ClientName,
        phone: PhoneNumber,
        notes: Option<ClientNotes>,
        now: DateTime<Utc>,
    ) -> AppResult<Client> {
        let client = Client::new(ClientId::new(), name, phone, notes, now);
        self.clients.save(&client).await?;
        Ok(client)
    }

    pub async fn rename_client(
        &self,
        client_id: ClientId,
        name: ClientName,
        now: DateTime<Utc>,
    ) -> AppResult<Client> {
        let mut client = require_client(&self.clients, client_id).await?;
        client.rename(name, now)?;
        self.clients.save(&client).await?;
        Ok(client)
    }

    pub async fn change_phone(
        &self,
        client_id: ClientId,
        phone: PhoneNumber,
        now: DateTime<Utc>,
    ) -> AppResult<Client> {
        let mut client = require_client(&self.clients, client_id).await?;
        client.change_phone(phone, now)?;
        self.clients.save(&client).await?;
        Ok(client)
    }

    pub async fn update_notes(
        &self,
        client_id: ClientId,
        notes: Option<ClientNotes>,
        now: DateTime<Utc>,
    ) -> AppResult<Client> {
        let mut client = require_client(&self.clients, client_id).await?;
        client.update_notes(notes, now)?;
        self.clients.save(&client).await?;
        Ok(client)
    }
}

pub struct CarService<Clients, Cars> {
    clients: Clients,
    cars: Cars,
}

impl<Clients, Cars> CarService<Clients, Cars>
where
    Clients: ClientRepository,
    Cars: CarRepository,
{
    pub fn new(clients: Clients, cars: Cars) -> Self {
        Self { clients, cars }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_car(
        &self,
        client_id: ClientId,
        make: CarMake,
        model: CarModel,
        year: Option<CarYear>,
        license_plate: Option<LicensePlate>,
        vin: Option<Vin>,
        notes: Option<CarNotes>,
        now: DateTime<Utc>,
    ) -> AppResult<Car> {
        require_client(&self.clients, client_id).await?;
        let car = Car::new(
            CarId::new(),
            client_id,
            make,
            model,
            year,
            license_plate,
            vin,
            notes,
            now,
        );
        self.cars.save(&car).await?;
        Ok(car)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_identity(
        &self,
        car_id: CarId,
        make: CarMake,
        model: CarModel,
        year: Option<CarYear>,
        license_plate: Option<LicensePlate>,
        vin: Option<Vin>,
        now: DateTime<Utc>,
    ) -> AppResult<Car> {
        let mut car = require_car(&self.cars, car_id).await?;
        car.update_identity(make, model, year, now)?;
        car.update_license_plate(license_plate, now)?;
        car.update_vin(vin, now)?;
        self.cars.save(&car).await?;
        Ok(car)
    }
}

pub struct BookingService<Clients, Cars, Bookings> {
    clients: Clients,
    cars: Cars,
    bookings: Bookings,
}

impl<Clients, Cars, Bookings> BookingService<Clients, Cars, Bookings>
where
    Clients: ClientRepository,
    Cars: CarRepository,
    Bookings: BookingRepository,
{
    pub fn new(clients: Clients, cars: Cars, bookings: Bookings) -> Self {
        Self {
            clients,
            cars,
            bookings,
        }
    }

    pub async fn schedule_booking(
        &self,
        client_id: ClientId,
        car_id: CarId,
        scheduled_at: DateTime<Utc>,
        reason: BookingReason,
        notes: Option<BookingNotes>,
        now: DateTime<Utc>,
    ) -> AppResult<Booking> {
        require_client(&self.clients, client_id).await?;
        let car = require_car(&self.cars, car_id).await?;
        ensure_car_belongs_to_client(&car, client_id)?;

        let booking = Booking::new(
            BookingId::new(),
            client_id,
            car_id,
            scheduled_at,
            reason,
            notes,
            now,
        );
        self.bookings.save(&booking).await?;
        Ok(booking)
    }

    pub async fn reschedule_booking(
        &self,
        booking_id: BookingId,
        scheduled_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> AppResult<Booking> {
        let mut booking = require_booking(&self.bookings, booking_id).await?;
        booking.reschedule(scheduled_at, now)?;
        self.bookings.save(&booking).await?;
        Ok(booking)
    }

    pub async fn complete_booking(
        &self,
        booking_id: BookingId,
        now: DateTime<Utc>,
    ) -> AppResult<Booking> {
        let mut booking = require_booking(&self.bookings, booking_id).await?;
        booking.complete(now)?;
        self.bookings.save(&booking).await?;
        Ok(booking)
    }

    pub async fn cancel_booking(
        &self,
        booking_id: BookingId,
        now: DateTime<Utc>,
    ) -> AppResult<Booking> {
        let mut booking = require_booking(&self.bookings, booking_id).await?;
        booking.cancel(now)?;
        self.bookings.save(&booking).await?;
        Ok(booking)
    }

    pub async fn mark_no_show(
        &self,
        booking_id: BookingId,
        now: DateTime<Utc>,
    ) -> AppResult<Booking> {
        let mut booking = require_booking(&self.bookings, booking_id).await?;
        booking.mark_no_show(now)?;
        self.bookings.save(&booking).await?;
        Ok(booking)
    }
}

pub struct PartService<R> {
    parts: R,
}

impl<R> PartService<R>
where
    R: PartRepository,
{
    pub fn new(parts: R) -> Self {
        Self { parts }
    }

    pub async fn create_part(
        &self,
        name: PartName,
        sku: Option<PartSku>,
        quantity: PartQuantity,
        min_quantity: PartQuantity,
        unit_price: Money,
        notes: Option<PartNotes>,
        now: DateTime<Utc>,
    ) -> AppResult<Part> {
        let part = Part::new(
            PartId::new(),
            name,
            sku,
            quantity,
            min_quantity,
            unit_price,
            notes,
            now,
        );
        self.parts.save(&part).await?;
        Ok(part)
    }

    pub async fn set_stock(
        &self,
        part_id: PartId,
        quantity: PartQuantity,
        now: DateTime<Utc>,
    ) -> AppResult<Part> {
        let mut part = require_part(&self.parts, part_id).await?;
        part.set_stock(quantity, now)?;
        self.parts.save(&part).await?;
        Ok(part)
    }
}

pub struct PartSupplyService<Parts, Supplies> {
    parts: Parts,
    supplies: Supplies,
}

impl<Parts, Supplies> PartSupplyService<Parts, Supplies>
where
    Parts: PartRepository,
    Supplies: PartSupplyRepository,
{
    pub fn new(parts: Parts, supplies: Supplies) -> Self {
        Self { parts, supplies }
    }

    pub async fn create_supply(
        &self,
        part_id: PartId,
        quantity: PartQuantity,
        expected_at: DateTime<Utc>,
        supplier: Option<PartSupplier>,
        notes: Option<PartSupplyNotes>,
        now: DateTime<Utc>,
    ) -> AppResult<PartSupply> {
        require_part(&self.parts, part_id).await?;
        let supply = PartSupply::new(
            PartSupplyId::new(),
            part_id,
            quantity,
            expected_at,
            supplier,
            notes,
            now,
        )?;
        self.supplies.save(&supply).await?;
        Ok(supply)
    }

    pub async fn receive_supply(
        &self,
        supply_id: PartSupplyId,
        now: DateTime<Utc>,
    ) -> AppResult<(PartSupply, Part)> {
        let mut supply = require_supply(&self.supplies, supply_id).await?;
        let mut part = require_part(&self.parts, supply.part_id()).await?;

        supply.mark_received(now)?;
        part.increase_stock(supply.quantity(), now)?;

        self.supplies.save(&supply).await?;
        self.parts.save(&part).await?;
        Ok((supply, part))
    }

    pub async fn cancel_supply(
        &self,
        supply_id: PartSupplyId,
        now: DateTime<Utc>,
    ) -> AppResult<PartSupply> {
        let mut supply = require_supply(&self.supplies, supply_id).await?;
        supply.cancel(now)?;
        self.supplies.save(&supply).await?;
        Ok(supply)
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfitSummary {
    pub currency: Currency,
    pub completed_repairs: usize,
    pub revenue: Money,
    pub parts_cost: Money,
    pub expected_profit: SignedMoney,
    pub actual_profit: SignedMoney,
}

pub struct StatisticsService<R> {
    repairs: R,
}

impl<R> StatisticsService<R>
where
    R: RepairRepository,
{
    pub fn new(repairs: R) -> Self {
        Self { repairs }
    }

    pub async fn profit_summary(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        currency: Currency,
    ) -> AppResult<ProfitSummary> {
        let repairs = self.repairs.list_completed_between(from, to).await?;
        let mut completed_repairs = 0;
        let mut revenue = Money::zero(currency);
        let mut parts_cost = Money::zero(currency);
        let mut expected_profit = SignedMoney::zero(currency);
        let mut actual_profit = SignedMoney::zero(currency);

        for repair in repairs
            .into_iter()
            .filter(|repair| repair.currency() == currency)
        {
            completed_repairs += 1;
            revenue = revenue.checked_add(repair.total_price()?)?;
            parts_cost = parts_cost.checked_add(repair.parts_cost())?;
            expected_profit = expected_profit.checked_add(repair.expected_profit()?)?;
            actual_profit = actual_profit.checked_add(repair.actual_profit()?)?;
        }

        Ok(ProfitSummary {
            currency,
            completed_repairs,
            revenue,
            parts_cost,
            expected_profit,
            actual_profit,
        })
    }
}

async fn require_client<R>(clients: &R, client_id: ClientId) -> AppResult<Client>
where
    R: ClientRepository,
{
    clients
        .get(client_id)
        .await?
        .ok_or(AppError::ClientNotFound(client_id))
}

async fn require_car<R>(cars: &R, car_id: CarId) -> AppResult<Car>
where
    R: CarRepository,
{
    cars.get(car_id).await?.ok_or(AppError::CarNotFound(car_id))
}

async fn require_booking<R>(bookings: &R, booking_id: BookingId) -> AppResult<Booking>
where
    R: BookingRepository,
{
    bookings
        .get(booking_id)
        .await?
        .ok_or(AppError::BookingNotFound(booking_id))
}

async fn require_part<R>(parts: &R, part_id: PartId) -> AppResult<Part>
where
    R: PartRepository,
{
    parts
        .get(part_id)
        .await?
        .ok_or(AppError::PartNotFound(part_id))
}

async fn require_supply<R>(supplies: &R, supply_id: PartSupplyId) -> AppResult<PartSupply>
where
    R: PartSupplyRepository,
{
    supplies
        .get(supply_id)
        .await?
        .ok_or(AppError::PartSupplyNotFound(supply_id))
}

async fn require_repair<R>(repairs: &R, repair_id: RepairId) -> AppResult<Repair>
where
    R: RepairRepository,
{
    repairs
        .get(repair_id)
        .await?
        .ok_or(AppError::RepairNotFound(repair_id))
}

fn ensure_car_belongs_to_client(car: &Car, client_id: ClientId) -> AppResult<()> {
    if car.client_id() != client_id {
        return Err(AppError::CarDoesNotBelongToClient {
            car_id: car.id(),
            client_id,
        });
    }

    Ok(())
}

fn ensure_booking_belongs_to_client_and_car(
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
