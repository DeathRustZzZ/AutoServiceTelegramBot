use garage_app::AppResult;
use garage_domain::{PartId, PartSupplier, PartSupply, PartSupplyId, PartSupplyNotes};

use crate::mappers::{map_part_supply_status, map_quantity};
use crate::models::PartSupplyRow;

pub fn to_domain(row: &PartSupplyRow) -> AppResult<PartSupply> {
    PartSupply::restore(
        PartSupplyId::from_uuid(row.id),
        PartId::from_uuid(row.part_id),
        map_quantity("part_supply", "quantity", row.quantity)?,
        row.expected_at,
        map_part_supply_status(&row.status)?,
        row.supplier
            .as_deref()
            .map(PartSupplier::parse)
            .transpose()?
            .flatten(),
        row.notes
            .as_deref()
            .map(PartSupplyNotes::parse)
            .transpose()?
            .flatten(),
        row.created_at,
        row.updated_at,
    )
    .map_err(Into::into)
}
