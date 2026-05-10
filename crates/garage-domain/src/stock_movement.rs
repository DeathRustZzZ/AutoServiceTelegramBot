//! История движений складского остатка.
//!
//! `Part` отвечает за текущее состояние склада: сколько единиц сейчас есть,
//! является ли позиция низкоостаточной и можно ли списать нужное количество.
//! Но текущего остатка недостаточно для аудита. Если количество изменилось,
//! сервис должен уметь объяснить, почему это произошло.
//!
//! `StockMovement` фиксирует один исторический факт изменения остатка
//! конкретной складской позиции. Сущность намеренно не меняет `Part.quantity`:
//! изменение остатка и создание движения должны быть скоординированы будущим
//! application-layer use case-ом, а инфраструктура позже сможет сохранить их в
//! одной транзакции.

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::{PartId, PartQuantity, StockMovementId};

/// Максимальная длина комментария к движению склада в Unicode-символах.
const MAX_STOCK_MOVEMENT_COMMENT_LEN: usize = 500;

/// Направление движения складского остатка.
///
/// `Adjustment` выделен отдельно, потому что ручная корректировка или
/// инвентаризация может технически увеличить или уменьшить остаток, но
/// бизнес-смысл у нее отличается от обычного прихода или списания.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StockMovementType {
    /// Остаток увеличился.
    In,
    /// Остаток уменьшился.
    Out,
    /// Ручная корректировка или инвентаризация.
    Adjustment,
}

/// Стабильное строковое представление направления движения.
impl std::fmt::Display for StockMovementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StockMovementType::In => write!(f, "in"),
            StockMovementType::Out => write!(f, "out"),
            StockMovementType::Adjustment => write!(f, "adjustment"),
        }
    }
}

/// Бизнес-причина движения склада.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StockMovementReason {
    /// Приход от поставщика.
    Supply,
    /// Списание запчасти в ремонт.
    RepairUsage,
    /// Возврат неиспользованной запчасти из ремонта.
    ReturnFromRepair,
    /// Корректировка после инвентаризации.
    InventoryCorrection,
    /// Ручное исправление остатка.
    ManualCorrection,
    /// Другая причина.
    Other,
}

/// Стабильное строковое представление причины движения.
impl std::fmt::Display for StockMovementReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StockMovementReason::Supply => write!(f, "supply"),
            StockMovementReason::RepairUsage => write!(f, "repair_usage"),
            StockMovementReason::ReturnFromRepair => write!(f, "return_from_repair"),
            StockMovementReason::InventoryCorrection => write!(f, "inventory_correction"),
            StockMovementReason::ManualCorrection => write!(f, "manual_correction"),
            StockMovementReason::Other => write!(f, "other"),
        }
    }
}

/// Проверенный комментарий к движению склада.
///
/// Комментарий необязателен. Пустой пользовательский ввод становится `None`, а
/// непустой текст хранится в trimmed-виде.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StockMovementComment(String);

impl StockMovementComment {
    /// Нормализует пользовательский комментарий.
    ///
    /// Алгоритм:
    /// 1. Убираем пробелы по краям.
    /// 2. Пустой результат возвращаем как `Ok(None)`.
    /// 3. Длину считаем через `chars().count()`, чтобы Unicode-текст
    ///    ограничивался в пользовательских символах, а не байтах UTF-8.
    /// 4. Сохраняем уже trimmed-строку.
    pub fn parse(input: &str) -> Result<Option<Self>, StockMovementError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(None);
        }

        let actual = trimmed.chars().count();

        if actual > MAX_STOCK_MOVEMENT_COMMENT_LEN {
            return Err(StockMovementError::CommentTooLong {
                max: MAX_STOCK_MOVEMENT_COMMENT_LEN,
                actual,
            });
        }

        Ok(Some(Self(trimmed.to_string())))
    }

    /// Возвращает комментарий без копирования строки.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Печатает комментарий в сохраненном виде.
impl std::fmt::Display for StockMovementComment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Исторический факт изменения остатка конкретной запчасти.
///
/// Сущность immutable для MVP. Исправление ошибочного движения лучше делать
/// отдельным корректирующим движением, чтобы складская история оставалась
/// проверяемой и не переписывалась без следа.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StockMovement {
    /// Стабильный идентификатор движения.
    id: StockMovementId,
    /// Складская позиция, по которой произошло движение.
    part_id: PartId,
    /// Направление изменения остатка.
    movement_type: StockMovementType,
    /// Количество единиц в движении. Ноль запрещен.
    quantity: PartQuantity,
    /// Бизнес-причина движения.
    reason: StockMovementReason,
    /// Необязательный комментарий к движению.
    comment: Option<StockMovementComment>,
    /// Фактическое время движения.
    occurred_at: DateTime<Utc>,
    /// Момент создания записи в системе.
    created_at: DateTime<Utc>,
}

impl StockMovement {
    /// Создает новое движение склада.
    ///
    /// Алгоритм:
    /// 1. Проверяем, что количество не равно нулю. Движение на 0 единиц не
    ///    меняет склад и только засоряет историю.
    /// 2. Проверяем временной порядок: фактическое движение не может быть позже
    ///    момента создания записи в системе. Задним числом движение внести
    ///    можно.
    /// 3. Сохраняем факт без изменения связанного `Part`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: StockMovementId,
        part_id: PartId,
        movement_type: StockMovementType,
        quantity: PartQuantity,
        reason: StockMovementReason,
        comment: Option<StockMovementComment>,
        occurred_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    ) -> Result<Self, StockMovementError> {
        Self::restore(
            id,
            part_id,
            movement_type,
            quantity,
            reason,
            comment,
            occurred_at,
            created_at,
        )
    }

    /// Восстанавливает движение склада из сохраненного состояния.
    ///
    /// `restore` проверяет те же инварианты, что и `new`: нулевое количество и
    /// невозможный временной порядок не становятся валидными после чтения из
    /// хранилища.
    #[allow(clippy::too_many_arguments)]
    pub fn restore(
        id: StockMovementId,
        part_id: PartId,
        movement_type: StockMovementType,
        quantity: PartQuantity,
        reason: StockMovementReason,
        comment: Option<StockMovementComment>,
        occurred_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    ) -> Result<Self, StockMovementError> {
        if quantity.is_zero() {
            return Err(StockMovementError::ZeroQuantity);
        }

        if occurred_at > created_at {
            return Err(StockMovementError::OccurredAtAfterCreatedAt);
        }

        Ok(Self {
            id,
            part_id,
            movement_type,
            quantity,
            reason,
            comment,
            occurred_at,
            created_at,
        })
    }

    /// Возвращает идентификатор движения склада.
    pub fn id(&self) -> StockMovementId {
        self.id
    }

    /// Возвращает идентификатор складской позиции.
    pub fn part_id(&self) -> PartId {
        self.part_id
    }

    /// Возвращает направление движения.
    pub fn movement_type(&self) -> StockMovementType {
        self.movement_type
    }

    /// Возвращает количество единиц в движении.
    pub fn quantity(&self) -> PartQuantity {
        self.quantity
    }

    /// Возвращает бизнес-причину движения.
    pub fn reason(&self) -> StockMovementReason {
        self.reason
    }

    /// Возвращает комментарий, если он есть.
    pub fn comment(&self) -> Option<&StockMovementComment> {
        self.comment.as_ref()
    }

    /// Возвращает фактическое время движения.
    pub fn occurred_at(&self) -> &DateTime<Utc> {
        &self.occurred_at
    }

    /// Возвращает момент создания записи в системе.
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}

/// Ошибка движения склада и комментария к нему.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StockMovementError {
    /// Движение с нулевым количеством не имеет бизнес-смысла.
    #[error("stock movement quantity cannot be zero")]
    ZeroQuantity,

    /// Фактическое движение не может быть позже создания записи в системе.
    #[error("stock movement occurred_at cannot be later than created_at")]
    OccurredAtAfterCreatedAt,

    /// Комментарий превышает допустимую длину в Unicode-символах.
    #[error("stock movement comment is too long: max={max}, actual={actual}")]
    CommentTooLong { max: usize, actual: usize },
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use super::{
        StockMovement, StockMovementComment, StockMovementError, StockMovementReason,
        StockMovementType, MAX_STOCK_MOVEMENT_COMMENT_LEN,
    };
    use crate::{PartId, PartQuantity, StockMovementId};

    fn fixed_time(seconds: i64) -> chrono::DateTime<Utc> {
        Utc.timestamp_opt(seconds, 0).single().unwrap()
    }

    fn stock_movement_id() -> StockMovementId {
        StockMovementId::from_uuid(Uuid::from_u128(1))
    }

    fn part_id() -> PartId {
        PartId::from_uuid(Uuid::from_u128(2))
    }

    fn comment(value: &str) -> StockMovementComment {
        StockMovementComment::parse(value).unwrap().unwrap()
    }

    #[test]
    fn stock_movement_comment_parse_trims_non_empty_comment() {
        let comment = StockMovementComment::parse("  списано на ремонт BMW  ")
            .unwrap()
            .unwrap();

        assert_eq!(comment.as_str(), "списано на ремонт BMW");
        assert_eq!(comment.to_string(), "списано на ремонт BMW");
    }

    #[test]
    fn stock_movement_comment_parse_returns_none_for_empty_input() {
        let comment = StockMovementComment::parse("   ").unwrap();

        assert!(comment.is_none());
    }

    #[test]
    fn stock_movement_comment_parse_allows_unicode_comment_at_max_length() {
        let input = "ж".repeat(MAX_STOCK_MOVEMENT_COMMENT_LEN);

        let comment = StockMovementComment::parse(&input).unwrap().unwrap();

        assert_eq!(
            comment.as_str().chars().count(),
            MAX_STOCK_MOVEMENT_COMMENT_LEN
        );
    }

    #[test]
    fn stock_movement_comment_parse_rejects_too_long_comment() {
        let input = "ж".repeat(MAX_STOCK_MOVEMENT_COMMENT_LEN + 1);

        let error = StockMovementComment::parse(&input).unwrap_err();

        assert_eq!(
            error,
            StockMovementError::CommentTooLong {
                max: MAX_STOCK_MOVEMENT_COMMENT_LEN,
                actual: MAX_STOCK_MOVEMENT_COMMENT_LEN + 1,
            }
        );
    }

    #[test]
    fn stock_movement_new_accepts_valid_movement() {
        let occurred_at = fixed_time(1_700_000_000);
        let created_at = fixed_time(1_700_000_100);
        let movement = StockMovement::new(
            stock_movement_id(),
            part_id(),
            StockMovementType::Out,
            PartQuantity::new(2),
            StockMovementReason::RepairUsage,
            Some(comment("Списано на ремонт BMW")),
            occurred_at,
            created_at,
        )
        .unwrap();

        assert_eq!(movement.id(), stock_movement_id());
        assert_eq!(movement.part_id(), part_id());
        assert_eq!(movement.movement_type(), StockMovementType::Out);
        assert_eq!(movement.quantity(), PartQuantity::new(2));
        assert_eq!(movement.reason(), StockMovementReason::RepairUsage);
        assert_eq!(
            movement.comment().unwrap().as_str(),
            "Списано на ремонт BMW"
        );
        assert_eq!(*movement.occurred_at(), occurred_at);
        assert_eq!(*movement.created_at(), created_at);
    }

    #[test]
    fn stock_movement_new_rejects_zero_quantity() {
        let error = StockMovement::new(
            stock_movement_id(),
            part_id(),
            StockMovementType::In,
            PartQuantity::zero(),
            StockMovementReason::Supply,
            None,
            fixed_time(1_700_000_000),
            fixed_time(1_700_000_000),
        )
        .unwrap_err();

        assert_eq!(error, StockMovementError::ZeroQuantity);
    }

    #[test]
    fn stock_movement_new_accepts_backdated_movement() {
        let occurred_at = fixed_time(1_700_000_000);
        let created_at = fixed_time(1_700_000_500);

        let movement = StockMovement::new(
            stock_movement_id(),
            part_id(),
            StockMovementType::Adjustment,
            PartQuantity::new(1),
            StockMovementReason::InventoryCorrection,
            None,
            occurred_at,
            created_at,
        )
        .unwrap();

        assert_eq!(*movement.occurred_at(), occurred_at);
        assert_eq!(*movement.created_at(), created_at);
    }

    #[test]
    fn stock_movement_new_rejects_occurred_at_after_created_at() {
        let error = StockMovement::new(
            stock_movement_id(),
            part_id(),
            StockMovementType::Out,
            PartQuantity::new(2),
            StockMovementReason::RepairUsage,
            None,
            fixed_time(1_700_000_500),
            fixed_time(1_700_000_000),
        )
        .unwrap_err();

        assert_eq!(error, StockMovementError::OccurredAtAfterCreatedAt);
    }

    #[test]
    fn stock_movement_restore_rejects_zero_quantity() {
        let error = StockMovement::restore(
            stock_movement_id(),
            part_id(),
            StockMovementType::In,
            PartQuantity::zero(),
            StockMovementReason::Supply,
            None,
            fixed_time(1_700_000_000),
            fixed_time(1_700_000_000),
        )
        .unwrap_err();

        assert_eq!(error, StockMovementError::ZeroQuantity);
    }

    #[test]
    fn stock_movement_type_display_returns_stable_codes() {
        assert_eq!(StockMovementType::In.to_string(), "in");
        assert_eq!(StockMovementType::Out.to_string(), "out");
        assert_eq!(StockMovementType::Adjustment.to_string(), "adjustment");
    }

    #[test]
    fn stock_movement_reason_display_returns_stable_codes() {
        assert_eq!(StockMovementReason::Supply.to_string(), "supply");
        assert_eq!(StockMovementReason::RepairUsage.to_string(), "repair_usage");
        assert_eq!(
            StockMovementReason::ReturnFromRepair.to_string(),
            "return_from_repair"
        );
        assert_eq!(
            StockMovementReason::InventoryCorrection.to_string(),
            "inventory_correction"
        );
        assert_eq!(
            StockMovementReason::ManualCorrection.to_string(),
            "manual_correction"
        );
        assert_eq!(StockMovementReason::Other.to_string(), "other");
    }
}
