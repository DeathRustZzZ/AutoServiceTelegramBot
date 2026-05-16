//! Сценарии чтения складских позиций.
//!
//! Query-сервис собирает карточку складской позиции вместе с историей движений
//! для UI и не мутирует доменные сущности.

use garage_domain::{Part, PartId, StockMovement};

use crate::{AppResult, PartRepository, StockMovementRepository};

use super::common::require_part;

/// Детальная карточка складской позиции для UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartDetails {
    pub part: Part,
    pub movements: Vec<StockMovement>,
}

/// Query-сервис для чтения детальных данных складской позиции.
pub struct PartQueryService<Parts, StockMovements> {
    parts: Parts,
    stock_movements: StockMovements,
}

impl<Parts, StockMovements> PartQueryService<Parts, StockMovements>
where
    Parts: PartRepository,
    StockMovements: StockMovementRepository,
{
    /// Создает query-сервис поверх repository ports.
    pub fn new(parts: Parts, stock_movements: StockMovements) -> Self {
        Self {
            parts,
            stock_movements,
        }
    }

    /// Возвращает складскую позицию вместе с историей движений.
    ///
    /// Архивная складская позиция здесь не запрещается: details используются для истории и
    /// отображения уже существующих данных.
    pub async fn get_part_details(&self, part_id: PartId) -> AppResult<PartDetails> {
        let part = require_part(&self.parts, part_id).await?;
        let movements = self.stock_movements.list_by_part(part_id).await?;

        Ok(PartDetails { part, movements })
    }
}
