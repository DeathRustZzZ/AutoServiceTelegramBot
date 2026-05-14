//! Transactional variant of using a stock part in a repair.
//!
//! This service mirrors `RepairPartService`, but writes through a Unit of Work
//! so the changed stock, repair part line and stock movement can later be
//! committed atomically by infrastructure.

use garage_domain::{
    RepairPart, RepairPartId, StockMovement, StockMovementId, StockMovementReason,
    StockMovementType,
};

use crate::{
    AppError, AppResult, PartRepository, RepairPartRepository, RepairPartUnitOfWork,
    RepairRepository, StockMovementRepository,
};

use super::{
    common::{ensure_part_active, require_part, require_repair},
    UsePartInRepairCommand, UsePartInRepairResult,
};

/// Application service for transactional part usage.
pub struct RepairPartTransactionalService<Uow> {
    uow: Uow,
}

impl<Uow> RepairPartTransactionalService<Uow>
where
    Uow: RepairPartUnitOfWork,
{
    /// Creates a service over a transactional repository bundle.
    pub fn new(uow: Uow) -> Self {
        Self { uow }
    }

    /// Uses a stock part in a repair and commits the transaction boundary.
    pub async fn use_part_in_repair(
        &self,
        command: UsePartInRepairCommand,
    ) -> AppResult<UsePartInRepairResult> {
        let mut repair = require_repair(self.uow.repairs(), command.repair_id).await?;
        if repair.is_cancelled() {
            return Err(AppError::CannotUsePartForCancelledRepair {
                repair_id: command.repair_id,
            });
        }

        let mut part = require_part(self.uow.parts(), command.part_id).await?;
        ensure_part_active(&part)?;
        part.decrease_stock(command.quantity, command.now)?;

        let repair_part = RepairPart::new(
            RepairPartId::new(),
            command.repair_id,
            command.part_id,
            command.quantity,
            command.unit_cost,
            command.unit_price,
            command.now,
        )?;

        let parts_price = repair.parts_price().checked_add(
            command
                .unit_price
                .checked_mul_u32(command.quantity.value())?,
        )?;
        let parts_cost = repair.parts_cost().checked_add(
            command
                .unit_cost
                .checked_mul_u32(command.quantity.value())?,
        )?;
        repair.update_prices(repair.labor_price(), parts_price, parts_cost, command.now)?;

        let movement = StockMovement::new(
            StockMovementId::new(),
            command.part_id,
            StockMovementType::Out,
            command.quantity,
            StockMovementReason::RepairUsage,
            command.comment,
            command.occurred_at,
            command.now,
        )?;

        if let Err(error) = self.uow.parts().save(&part).await {
            self.uow.rollback().await.ok();
            return Err(error);
        }

        if let Err(error) = self.uow.repair_parts().save(&repair_part).await {
            self.uow.rollback().await.ok();
            return Err(error);
        }

        if let Err(error) = self.uow.stock_movements().save(&movement).await {
            self.uow.rollback().await.ok();
            return Err(error);
        }

        if let Err(error) = self.uow.repairs().save(&repair).await {
            self.uow.rollback().await.ok();
            return Err(error);
        }

        if let Err(error) = self.uow.commit().await {
            self.uow.rollback().await.ok();
            return Err(error);
        }

        Ok(UsePartInRepairResult {
            repair_part,
            stock_movement: movement,
            is_low_stock: part.is_low_stock(),
            is_out_of_stock: part.is_out_of_stock(),
            part,
        })
    }
}
