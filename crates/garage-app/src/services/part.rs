//! Сценарии складских позиций.
//!
//! `Part` хранит текущее состояние позиции на складе: название, SKU, остаток,
//! минимальный остаток и цену. История поступлений вынесена в `PartSupply`.

use chrono::{DateTime, Utc};
use garage_domain::{Money, Part, PartId, PartName, PartNotes, PartQuantity, PartSku};

use crate::{AppResult, PartRepository};

use super::common::require_part;

/// Прикладной сервис для складских позиций.
pub struct PartService<R> {
    parts: R,
}

impl<R> PartService<R>
where
    R: PartRepository,
{
    /// Создает сервис поверх repository port складских позиций.
    pub fn new(parts: R) -> Self {
        Self { parts }
    }

    /// Создает новую складскую позицию.
    ///
    /// Начальный остаток может быть нулевым: позицию можно завести в каталог до
    /// первой поставки. Все ограничения названия, SKU и денег уже проверены
    /// соответствующими value objects.
    #[allow(clippy::too_many_arguments)]
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

    /// Устанавливает фактический остаток при инвентаризации.
    ///
    /// Это не сценарий поставки. Получение поставки должно идти через
    /// `PartSupplyService::receive_supply`, чтобы не потерять историю.
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

    /// Возвращает активную складскую позицию по идентификатору.
    pub async fn get_part(&self, part_id: PartId) -> AppResult<Part> {
        require_part(&self.parts, part_id).await
    }

    /// Ищет запчасти по пользовательскому запросу.
    ///
    /// App-layer не решает, искать ли по названию, SKU или индексу БД. Это
    /// ответственность реализации `PartRepository`.
    pub async fn search_parts(&self, query: &str) -> AppResult<Vec<Part>> {
        self.parts.search(query).await
    }

    /// Возвращает позиции с низким остатком.
    pub async fn list_low_stock(&self) -> AppResult<Vec<Part>> {
        self.parts.list_low_stock().await
    }

    /// Архивирует складскую позицию без физического удаления.
    pub async fn archive_part(&self, part_id: PartId, now: DateTime<Utc>) -> AppResult<Part> {
        let mut part = require_part(&self.parts, part_id).await?;
        part.archive(now)?;
        self.parts.save(&part).await?;
        Ok(part)
    }

    /// Возвращает складскую позицию из архива.
    pub async fn restore_part_from_archive(
        &self,
        part_id: PartId,
        now: DateTime<Utc>,
    ) -> AppResult<Part> {
        let mut part = require_part(&self.parts, part_id).await?;
        part.restore_from_archive(now)?;
        self.parts.save(&part).await?;
        Ok(part)
    }
}
