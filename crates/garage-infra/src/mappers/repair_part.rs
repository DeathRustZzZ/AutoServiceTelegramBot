use garage_app::AppResult;
use garage_domain::{PartId, RepairId, RepairPart, RepairPartId};

use crate::mappers::{map_currency, map_money, map_quantity};
use crate::models::RepairPartRow;

pub fn to_domain(row: &RepairPartRow) -> AppResult<RepairPart> {
    let currency = map_currency("repair_part", &row.currency)?;

    RepairPart::restore(
        RepairPartId::from_uuid(row.id),
        RepairId::from_uuid(row.repair_id),
        PartId::from_uuid(row.part_id),
        map_quantity("repair_part", "quantity", row.quantity)?,
        map_money("repair_part", "unit_cost", row.unit_cost, currency)?,
        map_money("repair_part", "unit_price", row.unit_price, currency)?,
        row.created_at,
    )
    .map_err(Into::into)
}
