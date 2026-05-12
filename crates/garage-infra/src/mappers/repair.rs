use garage_app::AppResult;
use garage_domain::{BookingId, CarId, ClientId, Repair, RepairDescription, RepairId, RepairNotes};

use crate::mappers::{map_currency, map_money, map_repair_status};
use crate::models::RepairRow;

pub fn to_domain(row: &RepairRow) -> AppResult<Repair> {
    let currency = map_currency("repair", &row.currency)?;

    Repair::restore(
        RepairId::from_uuid(row.id),
        ClientId::from_uuid(row.client_id),
        CarId::from_uuid(row.car_id),
        row.booking_id.map(BookingId::from_uuid),
        map_repair_status(&row.status)?,
        RepairDescription::parse(&row.description)?,
        map_money("repair", "labor_price", row.labor_price, currency)?,
        map_money("repair", "parts_price", row.parts_price, currency)?,
        map_money("repair", "parts_cost", row.parts_cost, currency)?,
        map_money("repair", "paid_amount", row.paid_amount, currency)?,
        row.notes
            .as_deref()
            .map(RepairNotes::parse)
            .transpose()?
            .flatten(),
        row.started_at,
        row.completed_at,
        row.created_at,
        row.updated_at,
    )
    .map_err(Into::into)
}
