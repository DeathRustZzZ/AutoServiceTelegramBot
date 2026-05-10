use chrono::{DateTime, Utc};
use garage_domain::{
    Part, PartId, PartQuantity, PartSupplier, PartSupply, PartSupplyId, PartSupplyNotes,
};

use crate::{AppResult, PartRepository, PartSupplyRepository};

use super::common::{require_part, require_supply};

/// Use cases for part supplies.
pub struct PartSupplyService<Parts, Supplies> {
    parts: Parts,
    supplies: Supplies,
}

impl<Parts, Supplies> PartSupplyService<Parts, Supplies>
where
    Parts: PartRepository,
    Supplies: PartSupplyRepository,
{
    pub fn new(parts: Parts, supplies: Supplies) -> Self {
        Self { parts, supplies }
    }

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
