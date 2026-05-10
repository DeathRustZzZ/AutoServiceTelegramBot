use chrono::{DateTime, Utc};
use garage_domain::{Money, Part, PartId, PartName, PartNotes, PartQuantity, PartSku};

use crate::{AppResult, PartRepository};

use super::common::require_part;

/// Use cases for warehouse parts.
pub struct PartService<R> {
    parts: R,
}

impl<R> PartService<R>
where
    R: PartRepository,
{
    pub fn new(parts: R) -> Self {
        Self { parts }
    }

    pub async fn create_part(
        &self,
        name: PartName,
        sku: Option<PartSku>,
        quantity: PartQuantity,
        min_quantity: PartQuantity,
        unit_price: Money,
        notes: Option<PartNotes>,
        now: DateTime<Utc>,
    ) -> AppResult<Part> {
        let part = Part::new(
            PartId::new(),
            name,
            sku,
            quantity,
            min_quantity,
            unit_price,
            notes,
            now,
        );
        self.parts.save(&part).await?;
        Ok(part)
    }

    pub async fn set_stock(
        &self,
        part_id: PartId,
        quantity: PartQuantity,
        now: DateTime<Utc>,
    ) -> AppResult<Part> {
        let mut part = require_part(&self.parts, part_id).await?;
        part.set_stock(quantity, now)?;
        self.parts.save(&part).await?;
        Ok(part)
    }

    pub async fn search_parts(&self, query: &str) -> AppResult<Vec<Part>> {
        self.parts.search(query).await
    }

    pub async fn list_low_stock(&self) -> AppResult<Vec<Part>> {
        self.parts.list_low_stock().await
    }
}
