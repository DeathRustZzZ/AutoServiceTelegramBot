use chrono::{DateTime, Utc};
use garage_domain::{Booking, BookingId, BookingNotes, BookingReason, CarId, ClientId};

use crate::{AppResult, BookingRepository, CarRepository, ClientRepository};

use super::common::{ensure_car_belongs_to_client, require_booking, require_car, require_client};

/// Use cases for bookings.
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

    pub async fn list_client_bookings(&self, client_id: ClientId) -> AppResult<Vec<Booking>> {
        require_client(&self.clients, client_id).await?;
        self.bookings.list_by_client(client_id).await
    }

    pub async fn list_car_bookings(&self, car_id: CarId) -> AppResult<Vec<Booking>> {
        require_car(&self.cars, car_id).await?;
        self.bookings.list_by_car(car_id).await
    }

    pub async fn list_bookings_between(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> AppResult<Vec<Booking>> {
        self.bookings.list_scheduled_between(from, to).await
    }
}
