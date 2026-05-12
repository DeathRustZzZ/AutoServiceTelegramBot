use garage_app::AppResult;
use garage_domain::{PartId, StockMovement, StockMovementComment, StockMovementId};

use crate::mappers::{map_quantity, map_stock_movement_reason, map_stock_movement_type};
use crate::models::StockMovementRow;

pub fn to_domain(row: &StockMovementRow) -> AppResult<StockMovement> {
    StockMovement::restore(
        StockMovementId::from_uuid(row.id),
        PartId::from_uuid(row.part_id),
        map_stock_movement_type(&row.movement_type)?,
        map_quantity("stock_movement", "quantity", row.quantity)?,
        map_stock_movement_reason(&row.reason)?,
        row.comment
            .as_deref()
            .map(StockMovementComment::parse)
            .transpose()?
            .flatten(),
        row.occurred_at,
        row.created_at,
    )
    .map_err(Into::into)
}
