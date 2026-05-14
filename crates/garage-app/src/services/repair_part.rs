//! Сценарии использования складских запчастей в ремонте.
//!
//! `RepairPartService` координирует три доменных факта:
//! - `Part` хранит текущий складской остаток;
//! - `RepairPart` фиксирует строку использованной запчасти в ремонте;
//! - `StockMovement` объясняет, почему изменился складской остаток.
//!
//! Сервис не пересчитывает цены ремонта и не сохраняет `Repair`: в этом
//! сценарии ремонт нужен только как контекст и защита от списания в закрытый
//! ремонт.

use chrono::{DateTime, Utc};
use garage_domain::{
    Money, Part, PartId, PartQuantity, RepairId, RepairPart, RepairPartId, StockMovement,
    StockMovementComment, StockMovementId, StockMovementReason, StockMovementType,
};

use crate::{
    AppError, AppResult, PartRepository, RepairPartRepository, RepairRepository,
    StockMovementRepository,
};

use super::common::{ensure_part_active, require_part, require_repair, require_repair_part};

/// Команда списания запчасти в ремонт.
///
/// `occurred_at` описывает фактическое время складского движения, а `now`
/// используется как момент изменения `Part` и создания новых записей.
pub struct UsePartInRepairCommand {
    pub repair_id: RepairId,
    pub part_id: PartId,
    pub quantity: PartQuantity,
    pub unit_cost: Money,
    pub unit_price: Money,
    pub comment: Option<StockMovementComment>,
    pub occurred_at: DateTime<Utc>,
    pub now: DateTime<Utc>,
}

/// Результат списания запчасти в ремонт.
///
/// Это application-level result model для UI: он возвращает созданные
/// исторические записи и состояние склада после списания.
pub struct UsePartInRepairResult {
    pub repair_part: RepairPart,
    pub stock_movement: StockMovement,
    pub part: Part,
    pub is_low_stock: bool,
    pub is_out_of_stock: bool,
}

/// Application service для запчастей, использованных в ремонте.
pub struct RepairPartService<Repairs, Parts, RepairParts, StockMovements> {
    repairs: Repairs,
    parts: Parts,
    repair_parts: RepairParts,
    stock_movements: StockMovements,
}

impl<Repairs, Parts, RepairParts, StockMovements>
    RepairPartService<Repairs, Parts, RepairParts, StockMovements>
where
    Repairs: RepairRepository,
    Parts: PartRepository,
    RepairParts: RepairPartRepository,
    StockMovements: StockMovementRepository,
{
    /// Создает сервис поверх нужных repository ports.
    pub fn new(
        repairs: Repairs,
        parts: Parts,
        repair_parts: RepairParts,
        stock_movements: StockMovements,
    ) -> Self {
        Self {
            repairs,
            parts,
            repair_parts,
            stock_movements,
        }
    }

    /// Фиксирует использование складской запчасти в ремонте.
    ///
    /// Порядок важен для атомарности на уровне app orchestration: `Part`
    /// сохраняется только после успешного списания в домене и успешного
    /// создания `RepairPart` и `StockMovement`.
    pub async fn use_part_in_repair(
        &self,
        command: UsePartInRepairCommand,
    ) -> AppResult<UsePartInRepairResult> {
        let mut repair = require_repair(&self.repairs, command.repair_id).await?;
        if !repair.is_in_progress() {
            return Err(AppError::CannotUsePartForClosedRepair {
                repair_id: command.repair_id,
            });
        }

        let mut part = require_part(&self.parts, command.part_id).await?;
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

        self.parts.save(&part).await?;
        self.repair_parts.save(&repair_part).await?;
        self.stock_movements.save(&movement).await?;
        self.repairs.save(&repair).await?;

        Ok(UsePartInRepairResult {
            repair_part,
            stock_movement: movement,
            is_low_stock: part.is_low_stock(),
            is_out_of_stock: part.is_out_of_stock(),
            part,
        })
    }

    /// Возвращает запчасти, использованные в ремонте.
    ///
    /// Сначала проверяем существование ремонта, чтобы ошибочный `repair_id` не
    /// выглядел как пустая история.
    pub async fn list_repair_parts(&self, repair_id: RepairId) -> AppResult<Vec<RepairPart>> {
        require_repair(&self.repairs, repair_id).await?;
        self.repair_parts.list_by_repair(repair_id).await
    }

    /// Возвращает строку использованной запчасти или `RepairPartNotFound`.
    pub async fn get_repair_part(&self, repair_part_id: RepairPartId) -> AppResult<RepairPart> {
        require_repair_part(&self.repair_parts, repair_part_id).await
    }
}
