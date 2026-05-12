use garage_app::AppResult;
use garage_domain::{Booking, BookingId, BookingNotes, BookingReason, CarId, ClientId};

use crate::mappers::map_booking_status;
use crate::models::BookingRow;

pub fn to_domain(row: &BookingRow) -> AppResult<Booking> {
    Booking::restore(
        BookingId::from_uuid(row.id),
        ClientId::from_uuid(row.client_id),
        CarId::from_uuid(row.car_id),
        row.scheduled_at,
        map_booking_status(&row.status)?,
        BookingReason::parse(&row.reason)?,
        row.notes
            .as_deref()
            .map(BookingNotes::parse)
            .transpose()?
            .flatten(),
        row.closed_at,
        row.created_at,
        row.updated_at,
    )
    .map_err(Into::into)
}
