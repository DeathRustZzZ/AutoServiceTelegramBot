use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::{Money, PartId};

const MAX_PART_NAME_LEN: usize = 150;
const MAX_PART_SKU_LEN: usize = 100;
const MAX_PART_NOTES_LEN: usize = 1000;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PartName(String);

impl PartName {
    pub fn parse(input: &str) -> Result<Self, PartError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Err(PartError::EmptyName);
        }

        let actual = trimmed.chars().count();

        if actual > MAX_PART_NAME_LEN {
            return Err(PartError::NameTooLong {
                max: MAX_PART_NAME_LEN,
                actual,
            });
        }

        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PartName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PartSku(String);

impl PartSku {
    pub fn parse(input: &str) -> Result<Option<Self>, PartError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(None);
        }

        let normalized = trimmed.to_uppercase();
        let actual = normalized.chars().count();

        if actual > MAX_PART_SKU_LEN {
            return Err(PartError::SkuTooLong {
                max: MAX_PART_SKU_LEN,
                actual,
            });
        }

        Ok(Some(Self(normalized)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PartSku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PartQuantity(u32);

impl PartQuantity {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn zero() -> Self {
        Self(0)
    }

    pub fn value(&self) -> u32 {
        self.0
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    pub fn checked_add(self, other: Self) -> Result<Self, PartError> {
        let value = self
            .0
            .checked_add(other.0)
            .ok_or(PartError::QuantityOverflow)?;

        Ok(Self(value))
    }

    pub fn checked_sub(self, other: Self) -> Result<Self, PartError> {
        let value = self
            .0
            .checked_sub(other.0)
            .ok_or(PartError::InsufficientStock {
                available: self.0,
                requested: other.0,
            })?;

        Ok(Self(value))
    }
}

impl std::fmt::Display for PartQuantity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PartNotes(String);

impl PartNotes {
    pub fn parse(input: &str) -> Result<Option<Self>, PartError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(None);
        }

        let actual = trimmed.chars().count();

        if actual > MAX_PART_NOTES_LEN {
            return Err(PartError::NotesTooLong {
                max: MAX_PART_NOTES_LEN,
                actual,
            });
        }

        Ok(Some(Self(trimmed.to_string())))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PartNotes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    id: PartId,
    name: PartName,
    sku: Option<PartSku>,
    quantity: PartQuantity,
    min_quantity: PartQuantity,
    unit_price: Money,
    notes: Option<PartNotes>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Part {
    pub fn new(
        id: PartId,
        name: PartName,
        sku: Option<PartSku>,
        quantity: PartQuantity,
        min_quantity: PartQuantity,
        unit_price: Money,
        notes: Option<PartNotes>,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            sku,
            quantity,
            min_quantity,
            unit_price,
            notes,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn restore(
        id: PartId,
        name: PartName,
        sku: Option<PartSku>,
        quantity: PartQuantity,
        min_quantity: PartQuantity,
        unit_price: Money,
        notes: Option<PartNotes>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, PartError> {
        if updated_at < created_at {
            return Err(PartError::UpdatedAtBeforeCreatedAt);
        }

        Ok(Self {
            id,
            name,
            sku,
            quantity,
            min_quantity,
            unit_price,
            notes,
            created_at,
            updated_at,
        })
    }

    pub fn id(&self) -> PartId {
        self.id
    }

    pub fn name(&self) -> &PartName {
        &self.name
    }

    pub fn sku(&self) -> Option<&PartSku> {
        self.sku.as_ref()
    }

    pub fn quantity(&self) -> PartQuantity {
        self.quantity
    }

    pub fn min_quantity(&self) -> PartQuantity {
        self.min_quantity
    }

    pub fn unit_price(&self) -> Money {
        self.unit_price
    }

    pub fn notes(&self) -> Option<&PartNotes> {
        self.notes.as_ref()
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    pub fn is_low_stock(&self) -> bool {
        self.quantity.value() <= self.min_quantity.value()
    }

    pub fn is_out_of_stock(&self) -> bool {
        self.quantity.is_zero()
    }

    pub fn update_name(&mut self, name: PartName, now: DateTime<Utc>) -> Result<(), PartError> {
        self.touch(now)?;
        self.name = name;
        Ok(())
    }

    pub fn update_sku(
        &mut self,
        sku: Option<PartSku>,
        now: DateTime<Utc>,
    ) -> Result<(), PartError> {
        self.touch(now)?;
        self.sku = sku;
        Ok(())
    }

    pub fn clear_sku(&mut self, now: DateTime<Utc>) -> Result<(), PartError> {
        self.touch(now)?;
        self.sku = None;
        Ok(())
    }

    pub fn update_min_quantity(
        &mut self,
        min_quantity: PartQuantity,
        now: DateTime<Utc>,
    ) -> Result<(), PartError> {
        self.touch(now)?;
        self.min_quantity = min_quantity;
        Ok(())
    }

    pub fn update_unit_price(
        &mut self,
        unit_price: Money,
        now: DateTime<Utc>,
    ) -> Result<(), PartError> {
        self.touch(now)?;
        self.unit_price = unit_price;
        Ok(())
    }

    pub fn update_notes(
        &mut self,
        notes: Option<PartNotes>,
        now: DateTime<Utc>,
    ) -> Result<(), PartError> {
        self.touch(now)?;
        self.notes = notes;
        Ok(())
    }

    pub fn clear_notes(&mut self, now: DateTime<Utc>) -> Result<(), PartError> {
        self.touch(now)?;
        self.notes = None;
        Ok(())
    }

    pub fn increase_stock(
        &mut self,
        amount: PartQuantity,
        now: DateTime<Utc>,
    ) -> Result<(), PartError> {
        let quantity = self.quantity.checked_add(amount)?;
        self.touch(now)?;
        self.quantity = quantity;
        Ok(())
    }

    pub fn decrease_stock(
        &mut self,
        amount: PartQuantity,
        now: DateTime<Utc>,
    ) -> Result<(), PartError> {
        let quantity = self.quantity.checked_sub(amount)?;
        self.touch(now)?;
        self.quantity = quantity;
        Ok(())
    }

    pub fn set_stock(
        &mut self,
        quantity: PartQuantity,
        now: DateTime<Utc>,
    ) -> Result<(), PartError> {
        self.touch(now)?;
        self.quantity = quantity;
        Ok(())
    }

    fn touch(&mut self, now: DateTime<Utc>) -> Result<(), PartError> {
        if now < self.created_at {
            return Err(PartError::UpdatedAtBeforeCreatedAt);
        }

        self.updated_at = now;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PartError {
    #[error("part name is empty")]
    EmptyName,

    #[error("part name is too long: max={max}, actual={actual}")]
    NameTooLong { max: usize, actual: usize },

    #[error("part sku is too long: max={max}, actual={actual}")]
    SkuTooLong { max: usize, actual: usize },

    #[error("part notes are too long: max={max}, actual={actual}")]
    NotesTooLong { max: usize, actual: usize },

    #[error("part quantity overflow")]
    QuantityOverflow,

    #[error("insufficient stock: available={available}, requested={requested}")]
    InsufficientStock { available: u32, requested: u32 },

    #[error("part updated_at cannot be earlier than created_at")]
    UpdatedAtBeforeCreatedAt,
}
