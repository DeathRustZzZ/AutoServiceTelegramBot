use garage_app::AppResult;
use garage_domain::{Part, PartId, PartName, PartNotes, PartSku};

use crate::mappers::{map_currency, map_money, map_part_status, map_quantity};
use crate::models::PartRow;

pub fn to_domain(row: &PartRow) -> AppResult<Part> {
    let currency = map_currency("part", &row.currency)?;

    Part::restore(
        PartId::from_uuid(row.id),
        PartName::parse(&row.name)?,
        row.sku
            .as_deref()
            .map(PartSku::parse)
            .transpose()?
            .flatten(),
        map_quantity("part", "quantity", row.quantity)?,
        map_quantity("part", "min_quantity", row.min_quantity)?,
        map_money("part", "unit_price", row.unit_price, currency)?,
        row.notes
            .as_deref()
            .map(PartNotes::parse)
            .transpose()?
            .flatten(),
        map_part_status(&row.status)?,
        row.created_at,
        row.updated_at,
    )
    .map_err(Into::into)
}
