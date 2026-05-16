//! Сценарии поставок запчастей.
//!
//! Поставка и складская позиция разделены намеренно: `PartSupply` описывает
//! ожидаемое/полученное пополнение, а `Part` хранит текущий остаток. Сервис
//! координирует оба агрегата, когда поставка фактически получена.

use chrono::{DateTime, Utc};
use garage_domain::{
    Part, PartId, PartQuantity, PartSupplier, PartSupply, PartSupplyId, PartSupplyNotes,
};

use crate::{AppResult, PartRepository, PartSupplyRepository};

use super::common::{require_part, require_supply};

/// Прикладной сервис для поставок.
pub struct PartSupplyService<Parts, Supplies> {
    parts: Parts,
    supplies: Supplies,
}

impl<Parts, Supplies> PartSupplyService<Parts, Supplies>
where
    Parts: PartRepository,
    Supplies: PartSupplyRepository,
{
    /// Создает сервис поставок.
    pub fn new(parts: Parts, supplies: Supplies) -> Self {
        Self { parts, supplies }
    }

    /// Создает ожидаемую поставку для существующей складской позиции.
    ///
    /// Проверка нулевого количества остается в `PartSupply::new`.
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

    /// Отмечает поставку полученной и увеличивает остаток склада.
    ///
    /// Это multi-aggregate сценарий:
    /// 1. Закрыть `PartSupply` как received.
    /// 2. Увеличить `Part.quantity`.
    /// 3. Сохранить оба агрегата.
    ///
    /// В PostgreSQL-реализации оба сохранения должны выполняться в одной
    /// транзакции.
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

    /// Отменяет ожидаемую поставку без изменения складского остатка.
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
