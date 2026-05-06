//! Поставка запчасти на склад автосервиса.
//!
//! Модуль описывает не саму складскую позицию, а ожидаемое пополнение этой
//! позиции: сколько единиц должно приехать, когда поставка ожидается, кто
//! поставщик и чем закончился жизненный цикл поставки.
//!
//! Дизайн повторяет остальные доменные модули:
//! 1. `PartSupplier` и `PartSupplyNotes` нормализуют пользовательский текст:
//!    убирают внешние пробелы, превращают пустой ввод в `None` и ограничивают
//!    длину в Unicode-символах.
//! 2. `PartSupply` управляет жизненным циклом поставки: создание, восстановление
//!    из хранилища, редактирование ожидаемой поставки и финальные переходы.
//!
//! Главный инвариант: новая поставка всегда создается как `Expected`, а из
//! этого состояния ее можно закрыть только как `Received` или `Cancelled`.
//! После финального статуса нельзя менять бизнес-поля поставки, чтобы не
//! переписать уже зафиксированную историю. Заметки остаются редактируемыми:
//! это операционный комментарий, который может понадобиться и после закрытия.

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::{PartId, PartQuantity, PartSupplyId};

/// Максимальная длина имени поставщика в Unicode-символах.
const MAX_PART_SUPPLIER_LEN: usize = 150;
/// Максимальная длина заметки по поставке в Unicode-символах.
const MAX_PART_SUPPLY_NOTES_LEN: usize = 1000;

/// Текущее состояние поставки запчасти.
///
/// Статусы намеренно сведены к трем бизнес-состояниям: поставка ожидается,
/// получена или отменена. `Received` и `Cancelled` являются финальными.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PartSupplyStatus {
    /// Поставка создана и еще не закрыта.
    Expected,
    /// Поставка приехала на склад.
    Received,
    /// Поставка отменена и больше не ожидается.
    Cancelled,
}

/// Человекочитаемое представление статуса для ошибок, логов и простого UI.
impl std::fmt::Display for PartSupplyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartSupplyStatus::Expected => write!(f, "expected"),
            PartSupplyStatus::Received => write!(f, "received"),
            PartSupplyStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Проверенное имя поставщика.
///
/// Поставщик необязателен: для разовой покупки или ручного пополнения его может
/// не быть. Поэтому парсер возвращает `Option<PartSupplier>`, а пустая строка
/// не хранится как отдельное значение.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PartSupplier(String);

impl PartSupplier {
    /// Нормализует пользовательское имя поставщика.
    ///
    /// Алгоритм:
    /// 1. Убираем пробелы по краям.
    /// 2. Пустой результат превращаем в `Ok(None)`.
    /// 3. Непустой текст ограничиваем по количеству Unicode-символов, чтобы
    ///    кириллица и другие UTF-8 символы считались пользовательски ожидаемо.
    /// 4. Сохраняем уже очищенную строку как каноническое значение.
    pub fn parse(input: &str) -> Result<Option<Self>, PartSupplyError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(None);
        }

        let actual = trimmed.chars().count();

        if actual > MAX_PART_SUPPLIER_LEN {
            return Err(PartSupplyError::SupplierTooLong {
                max: MAX_PART_SUPPLIER_LEN,
                actual,
            });
        }

        Ok(Some(Self(trimmed.to_string())))
    }

    /// Возвращает имя поставщика без копирования строки.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Печатает поставщика в сохраненном виде.
impl std::fmt::Display for PartSupplier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Проверенная заметка по поставке.
///
/// Заметка необязательна и хранится как `None`, если пользователь оставил поле
/// пустым. Это убирает неоднозначность между "заметки нет" и `Some("")`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PartSupplyNotes(String);

impl PartSupplyNotes {
    /// Нормализует пользовательскую заметку по поставке.
    ///
    /// Алгоритм такой же, как у других необязательных текстовых полей домена:
    /// внешние пробелы удаляются, пустой ввод становится `None`, а непустой
    /// текст проверяется по лимиту Unicode-символов.
    pub fn parse(input: &str) -> Result<Option<Self>, PartSupplyError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(None);
        }

        let actual = trimmed.chars().count();

        if actual > MAX_PART_SUPPLY_NOTES_LEN {
            return Err(PartSupplyError::NotesTooLong {
                max: MAX_PART_SUPPLY_NOTES_LEN,
                actual,
            });
        }

        Ok(Some(Self(trimmed.to_string())))
    }

    /// Возвращает заметку без копирования строки.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Печатает заметку без дополнительного форматирования.
impl std::fmt::Display for PartSupplyNotes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Ожидаемое пополнение конкретной складской позиции.
///
/// `PartSupply` - доменная сущность со стабильным идентификатором, связью с
/// `Part`, количеством, ожидаемой датой и управляемым статусом. Поля закрыты,
/// чтобы вызывающий код не обходил проверки количества, временного порядка и
/// допустимых переходов статуса.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartSupply {
    /// Стабильный идентификатор поставки.
    id: PartSupplyId,
    /// Складская позиция, которую пополняет поставка.
    part_id: PartId,
    /// Ожидаемое количество. Ноль запрещен для поставки.
    quantity: PartQuantity,
    /// Дата и время, когда поставка ожидается.
    expected_at: DateTime<Utc>,
    /// Текущий статус поставки.
    status: PartSupplyStatus,
    /// Опциональный поставщик.
    supplier: Option<PartSupplier>,
    /// Опциональная внутренняя заметка.
    notes: Option<PartSupplyNotes>,
    /// Момент создания поставки.
    created_at: DateTime<Utc>,
    /// Момент последнего изменения поставки.
    updated_at: DateTime<Utc>,
}

impl PartSupply {
    /// Создает новую ожидаемую поставку.
    ///
    /// Метод принимает уже проверенные value objects, но дополнительно
    /// проверяет количество: поставка с нулевым количеством не имеет
    /// бизнес-смысла и не должна попадать в очередь ожидания.
    ///
    /// После успешного создания статус всегда `Expected`, а `created_at` и
    /// `updated_at` равны переданному `now`.
    pub fn new(
        id: PartSupplyId,
        part_id: PartId,
        quantity: PartQuantity,
        expected_at: DateTime<Utc>,
        supplier: Option<PartSupplier>,
        notes: Option<PartSupplyNotes>,
        now: DateTime<Utc>,
    ) -> Result<Self, PartSupplyError> {
        if quantity.is_zero() {
            return Err(PartSupplyError::ZeroQuantity);
        }

        Ok(Self {
            id,
            part_id,
            quantity,
            expected_at,
            status: PartSupplyStatus::Expected,
            supplier,
            notes,
            created_at: now,
            updated_at: now,
        })
    }

    /// Восстанавливает поставку из сохраненного состояния.
    ///
    /// Этот метод предназначен для репозитория. В отличие от `new`, статус и
    /// даты приходят извне, поэтому домен проверяет инварианты, которые могли
    /// быть нарушены в хранилище или миграции:
    /// 1. Количество не может быть нулевым.
    /// 2. `updated_at` не может быть раньше `created_at`.
    ///
    /// Метод не исправляет поврежденные данные автоматически: такая коррекция
    /// скрыла бы проблему, а явная ошибка позволяет обработать ее на уровне
    /// приложения или миграции.
    #[allow(clippy::too_many_arguments)]
    pub fn restore(
        id: PartSupplyId,
        part_id: PartId,
        quantity: PartQuantity,
        expected_at: DateTime<Utc>,
        status: PartSupplyStatus,
        supplier: Option<PartSupplier>,
        notes: Option<PartSupplyNotes>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, PartSupplyError> {
        if quantity.is_zero() {
            return Err(PartSupplyError::ZeroQuantity);
        }

        if updated_at < created_at {
            return Err(PartSupplyError::UpdatedAtBeforeCreatedAt);
        }

        Ok(Self {
            id,
            part_id,
            quantity,
            expected_at,
            status,
            supplier,
            notes,
            created_at,
            updated_at,
        })
    }

    /// Возвращает идентификатор поставки.
    pub fn id(&self) -> PartSupplyId {
        self.id
    }

    /// Возвращает идентификатор складской позиции.
    pub fn part_id(&self) -> PartId {
        self.part_id
    }

    /// Возвращает ожидаемое количество.
    pub fn quantity(&self) -> PartQuantity {
        self.quantity
    }

    /// Возвращает ожидаемую дату поставки.
    pub fn expected_at(&self) -> &DateTime<Utc> {
        &self.expected_at
    }

    /// Возвращает текущий статус поставки.
    pub fn status(&self) -> PartSupplyStatus {
        self.status
    }

    /// Возвращает поставщика, если он указан.
    pub fn supplier(&self) -> Option<&PartSupplier> {
        self.supplier.as_ref()
    }

    /// Возвращает заметку, если она есть.
    pub fn notes(&self) -> Option<&PartSupplyNotes> {
        self.notes.as_ref()
    }

    /// Возвращает дату создания поставки.
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    /// Возвращает дату последнего изменения поставки.
    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    /// Проверяет, остается ли поставка ожидаемой.
    pub fn is_expected(&self) -> bool {
        self.status == PartSupplyStatus::Expected
    }

    /// Проверяет, закрыта ли поставка финальным статусом.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            PartSupplyStatus::Received | PartSupplyStatus::Cancelled
        )
    }

    /// Переносит ожидаемую дату поставки.
    ///
    /// Перенос разрешен только для `Expected`: после получения или отмены дата
    /// ожидания становится частью истории и не должна меняться обычным
    /// редактированием. Сначала выполняются проверки, затем меняется состояние,
    /// поэтому при ошибке сущность остается неизменной.
    pub fn reschedule(
        &mut self,
        expected_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Result<(), PartSupplyError> {
        self.ensure_expected_for_modification()?;
        self.touch(now)?;
        self.expected_at = expected_at;
        Ok(())
    }

    /// Меняет ожидаемое количество поставки.
    ///
    /// Нулевое количество запрещено. Проверка выполняется до `touch`, чтобы
    /// не обновлять timestamp при отклоненной операции.
    pub fn update_quantity(
        &mut self,
        quantity: PartQuantity,
        now: DateTime<Utc>,
    ) -> Result<(), PartSupplyError> {
        self.ensure_expected_for_modification()?;

        if quantity.is_zero() {
            return Err(PartSupplyError::ZeroQuantity);
        }

        self.touch(now)?;
        self.quantity = quantity;
        Ok(())
    }

    /// Заменяет поставщика и фиксирует время изменения.
    ///
    /// Изменение поставщика разрешено только пока поставка ожидается. `None`
    /// означает осознанное отсутствие поставщика.
    pub fn update_supplier(
        &mut self,
        supplier: Option<PartSupplier>,
        now: DateTime<Utc>,
    ) -> Result<(), PartSupplyError> {
        self.ensure_expected_for_modification()?;
        self.touch(now)?;
        self.supplier = supplier;
        Ok(())
    }

    /// Удаляет поставщика и фиксирует время изменения.
    pub fn clear_supplier(&mut self, now: DateTime<Utc>) -> Result<(), PartSupplyError> {
        self.ensure_expected_for_modification()?;
        self.touch(now)?;
        self.supplier = None;
        Ok(())
    }

    /// Заменяет заметку по поставке.
    ///
    /// Заметка считается операционным комментарием, поэтому ее можно менять и
    /// после финального статуса. Это позволяет дописать причину отмены,
    /// номер накладной или уточнение по фактическому получению.
    pub fn update_notes(
        &mut self,
        notes: Option<PartSupplyNotes>,
        now: DateTime<Utc>,
    ) -> Result<(), PartSupplyError> {
        self.touch(now)?;
        self.notes = notes;
        Ok(())
    }

    /// Удаляет заметку и фиксирует время изменения.
    pub fn clear_notes(&mut self, now: DateTime<Utc>) -> Result<(), PartSupplyError> {
        self.touch(now)?;
        self.notes = None;
        Ok(())
    }

    /// Закрывает поставку как полученную.
    pub fn mark_received(&mut self, now: DateTime<Utc>) -> Result<(), PartSupplyError> {
        self.transition_to(PartSupplyStatus::Received, now)
    }

    /// Закрывает поставку как отмененную.
    pub fn cancel(&mut self, now: DateTime<Utc>) -> Result<(), PartSupplyError> {
        self.transition_to(PartSupplyStatus::Cancelled, now)
    }

    /// Обновляет `updated_at`, сохраняя временной инвариант сущности.
    ///
    /// Метод приватный: сам по себе `touch` не является бизнес-действием.
    /// Публичные методы вызывают его до записи нового значения, поэтому ошибка
    /// времени останавливает операцию до частичного изменения состояния.
    fn touch(&mut self, now: DateTime<Utc>) -> Result<(), PartSupplyError> {
        if now < self.created_at {
            return Err(PartSupplyError::UpdatedAtBeforeCreatedAt);
        }

        self.updated_at = now;
        Ok(())
    }

    /// Проверяет, что бизнес-поля поставки еще можно редактировать.
    ///
    /// Полученная или отмененная поставка считается финальной: изменение даты,
    /// количества или поставщика в таком состоянии переписало бы историю.
    fn ensure_expected_for_modification(&self) -> Result<(), PartSupplyError> {
        if self.status != PartSupplyStatus::Expected {
            return Err(PartSupplyError::CannotModifyFinalSupply {
                status: self.status,
            });
        }

        Ok(())
    }

    /// Проверяет, что текущий статус допускает выбранный финальный переход.
    ///
    /// В текущей модели допустимы только переходы из `Expected` в финальный
    /// статус. Переходы между финальными статусами запрещены.
    fn ensure_expected_for_transition(&self, to: PartSupplyStatus) -> Result<(), PartSupplyError> {
        if self.status != PartSupplyStatus::Expected {
            return Err(PartSupplyError::CannotTransitionStatus {
                from: self.status,
                to,
            });
        }

        Ok(())
    }

    /// Выполняет общий алгоритм финального перехода статуса.
    ///
    /// Все публичные методы закрытия используют один путь, чтобы правила
    /// переходов и обновления `updated_at` не расходились.
    fn transition_to(
        &mut self,
        status: PartSupplyStatus,
        now: DateTime<Utc>,
    ) -> Result<(), PartSupplyError> {
        self.ensure_expected_for_transition(status)?;
        self.touch(now)?;
        self.status = status;
        Ok(())
    }
}

/// Ошибки доменной модели поставок запчастей.
///
/// Ошибки описывают нарушения бизнес-инвариантов: нулевое количество,
/// превышение лимита текстовых полей, поврежденный порядок дат или попытку
/// изменить/перевести поставку из недопустимого статуса.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PartSupplyError {
    /// Поставка не может иметь нулевое количество.
    #[error("part supply quantity cannot be zero")]
    ZeroQuantity,

    /// Имя поставщика превышает допустимый лимит Unicode-символов.
    #[error("part supplier is too long: max={max}, actual={actual}")]
    SupplierTooLong { max: usize, actual: usize },

    /// Заметка превышает допустимый лимит Unicode-символов.
    #[error("part supply notes are too long: max={max}, actual={actual}")]
    NotesTooLong { max: usize, actual: usize },

    /// `updated_at` оказался раньше `created_at`.
    #[error("part supply updated_at cannot be earlier than created_at")]
    UpdatedAtBeforeCreatedAt,

    /// Попытка изменить бизнес-поля поставки после финального статуса.
    #[error("cannot modify part supply with final status {status}")]
    CannotModifyFinalSupply { status: PartSupplyStatus },

    /// Попытка выполнить недопустимый переход статуса.
    #[error("cannot transition part supply status from {from} to {to}")]
    CannotTransitionStatus {
        from: PartSupplyStatus,
        to: PartSupplyStatus,
    },
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeZone, Utc};
    use uuid::Uuid;

    use super::{
        PartSupplier, PartSupply, PartSupplyError, PartSupplyNotes, PartSupplyStatus,
        MAX_PART_SUPPLIER_LEN, MAX_PART_SUPPLY_NOTES_LEN,
    };
    use crate::{PartId, PartQuantity, PartSupplyId};

    fn fixed_time(seconds: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(seconds, 0).single().unwrap()
    }

    fn part_supply_id() -> PartSupplyId {
        PartSupplyId::from_uuid(Uuid::from_u128(1))
    }

    fn part_id() -> PartId {
        PartId::from_uuid(Uuid::from_u128(2))
    }

    fn supplier(value: &str) -> PartSupplier {
        PartSupplier::parse(value).unwrap().unwrap()
    }

    fn notes(value: &str) -> PartSupplyNotes {
        PartSupplyNotes::parse(value).unwrap().unwrap()
    }

    fn expected_supply(now: DateTime<Utc>) -> PartSupply {
        PartSupply::new(
            part_supply_id(),
            part_id(),
            PartQuantity::new(5),
            fixed_time(1_700_100_000),
            Some(supplier("ООО АвтоПоставка")),
            Some(notes("Ожидаем утром")),
            now,
        )
        .unwrap()
    }

    /// Поставщик очищается от внешних пробелов и сохраняется как `Some`.
    #[test]
    fn supplier_parse_trims_non_empty_supplier() {
        let supplier = PartSupplier::parse("  Bosch Service  ").unwrap().unwrap();

        assert_eq!(supplier.as_str(), "Bosch Service");
        assert_eq!(supplier.to_string(), "Bosch Service");
    }

    /// Пустой поставщик не является ошибкой и представляется как отсутствие значения.
    #[test]
    fn supplier_parse_returns_none_for_blank_input() {
        let supplier = PartSupplier::parse("   ").unwrap();

        assert_eq!(supplier, None);
    }

    /// Лимит поставщика считается в Unicode-символах, а не в байтах UTF-8.
    #[test]
    fn supplier_parse_allows_unicode_supplier_at_max_length() {
        let input = "я".repeat(MAX_PART_SUPPLIER_LEN);

        let supplier = PartSupplier::parse(&input).unwrap().unwrap();

        assert_eq!(supplier.as_str().chars().count(), MAX_PART_SUPPLIER_LEN);
    }

    /// При превышении лимита поставщик возвращает структурированную ошибку.
    #[test]
    fn supplier_parse_rejects_too_long_supplier() {
        let input = "a".repeat(MAX_PART_SUPPLIER_LEN + 1);

        let error = PartSupplier::parse(&input).unwrap_err();

        assert_eq!(
            error,
            PartSupplyError::SupplierTooLong {
                max: MAX_PART_SUPPLIER_LEN,
                actual: MAX_PART_SUPPLIER_LEN + 1,
            }
        );
    }

    /// Непустая заметка очищается от внешних пробелов.
    #[test]
    fn notes_parse_trims_non_empty_notes() {
        let notes = PartSupplyNotes::parse("  Накладная будет позже  ")
            .unwrap()
            .unwrap();

        assert_eq!(notes.as_str(), "Накладная будет позже");
        assert_eq!(notes.to_string(), "Накладная будет позже");
    }

    /// Пустая заметка хранится как отсутствие значения.
    #[test]
    fn notes_parse_returns_none_for_blank_input() {
        let notes = PartSupplyNotes::parse("\n\t ").unwrap();

        assert_eq!(notes, None);
    }

    /// Лимит заметки также считается в Unicode-символах.
    #[test]
    fn notes_parse_allows_unicode_notes_at_max_length() {
        let input = "ю".repeat(MAX_PART_SUPPLY_NOTES_LEN);

        let notes = PartSupplyNotes::parse(&input).unwrap().unwrap();

        assert_eq!(notes.as_str().chars().count(), MAX_PART_SUPPLY_NOTES_LEN);
    }

    /// Слишком длинная заметка возвращает точный `actual`.
    #[test]
    fn notes_parse_rejects_too_long_notes() {
        let input = "a".repeat(MAX_PART_SUPPLY_NOTES_LEN + 1);

        let error = PartSupplyNotes::parse(&input).unwrap_err();

        assert_eq!(
            error,
            PartSupplyError::NotesTooLong {
                max: MAX_PART_SUPPLY_NOTES_LEN,
                actual: MAX_PART_SUPPLY_NOTES_LEN + 1,
            }
        );
    }

    /// Новая поставка создается ожидаемой и фиксирует одинаковые даты создания
    /// и обновления.
    #[test]
    fn new_creates_expected_supply() {
        let now = fixed_time(1_700_000_000);

        let supply = expected_supply(now);

        assert_eq!(supply.id(), part_supply_id());
        assert_eq!(supply.part_id(), part_id());
        assert_eq!(supply.quantity(), PartQuantity::new(5));
        assert_eq!(*supply.expected_at(), fixed_time(1_700_100_000));
        assert_eq!(supply.status(), PartSupplyStatus::Expected);
        assert!(supply.is_expected());
        assert!(!supply.is_terminal());
        assert_eq!(supply.supplier().unwrap().as_str(), "ООО АвтоПоставка");
        assert_eq!(supply.notes().unwrap().as_str(), "Ожидаем утром");
        assert_eq!(*supply.created_at(), now);
        assert_eq!(*supply.updated_at(), now);
    }

    /// Поставка с нулевым количеством не должна попадать в домен.
    #[test]
    fn new_rejects_zero_quantity() {
        let now = fixed_time(1_700_000_000);

        let error = PartSupply::new(
            part_supply_id(),
            part_id(),
            PartQuantity::zero(),
            fixed_time(1_700_100_000),
            None,
            None,
            now,
        )
        .unwrap_err();

        assert_eq!(error, PartSupplyError::ZeroQuantity);
    }

    /// Восстановление принимает сохраненный финальный статус и даты.
    #[test]
    fn restore_accepts_valid_persisted_supply() {
        let created_at = fixed_time(1_700_000_000);
        let updated_at = fixed_time(1_700_000_500);

        let supply = PartSupply::restore(
            part_supply_id(),
            part_id(),
            PartQuantity::new(3),
            fixed_time(1_700_100_000),
            PartSupplyStatus::Received,
            Some(supplier("Поставщик")),
            None,
            created_at,
            updated_at,
        )
        .unwrap();

        assert_eq!(supply.status(), PartSupplyStatus::Received);
        assert!(supply.is_terminal());
        assert_eq!(*supply.created_at(), created_at);
        assert_eq!(*supply.updated_at(), updated_at);
    }

    /// Поврежденный порядок дат при восстановлении возвращается как доменная ошибка.
    #[test]
    fn restore_rejects_updated_at_before_created_at() {
        let created_at = fixed_time(1_700_000_500);
        let updated_at = fixed_time(1_700_000_000);

        let error = PartSupply::restore(
            part_supply_id(),
            part_id(),
            PartQuantity::new(3),
            fixed_time(1_700_100_000),
            PartSupplyStatus::Expected,
            None,
            None,
            created_at,
            updated_at,
        )
        .unwrap_err();

        assert_eq!(error, PartSupplyError::UpdatedAtBeforeCreatedAt);
    }

    /// Перенос ожидаемой поставки меняет дату и `updated_at`.
    #[test]
    fn reschedule_updates_expected_at_and_timestamp() {
        let now = fixed_time(1_700_000_000);
        let changed_at = fixed_time(1_700_000_100);
        let expected_at = fixed_time(1_700_200_000);
        let mut supply = expected_supply(now);

        supply.reschedule(expected_at, changed_at).unwrap();

        assert_eq!(*supply.expected_at(), expected_at);
        assert_eq!(*supply.updated_at(), changed_at);
    }

    /// Обновление количества запрещает ноль и не меняет timestamp при ошибке.
    #[test]
    fn update_quantity_rejects_zero_without_touching_supply() {
        let now = fixed_time(1_700_000_000);
        let changed_at = fixed_time(1_700_000_100);
        let mut supply = expected_supply(now);

        let error = supply
            .update_quantity(PartQuantity::zero(), changed_at)
            .unwrap_err();

        assert_eq!(error, PartSupplyError::ZeroQuantity);
        assert_eq!(supply.quantity(), PartQuantity::new(5));
        assert_eq!(*supply.updated_at(), now);
    }

    /// Поставщика можно заменить и затем явно очистить, пока поставка ожидается.
    #[test]
    fn update_and_clear_supplier_work_for_expected_supply() {
        let now = fixed_time(1_700_000_000);
        let changed_at = fixed_time(1_700_000_100);
        let cleared_at = fixed_time(1_700_000_200);
        let mut supply = expected_supply(now);

        supply
            .update_supplier(Some(supplier("Новый поставщик")), changed_at)
            .unwrap();

        assert_eq!(supply.supplier().unwrap().as_str(), "Новый поставщик");
        assert_eq!(*supply.updated_at(), changed_at);

        supply.clear_supplier(cleared_at).unwrap();

        assert_eq!(supply.supplier(), None);
        assert_eq!(*supply.updated_at(), cleared_at);
    }

    /// Заметки можно менять и очищать независимо от финального статуса.
    #[test]
    fn notes_can_be_updated_after_supply_is_received() {
        let now = fixed_time(1_700_000_000);
        let received_at = fixed_time(1_700_000_100);
        let noted_at = fixed_time(1_700_000_200);
        let cleared_at = fixed_time(1_700_000_300);
        let mut supply = expected_supply(now);
        supply.mark_received(received_at).unwrap();

        supply
            .update_notes(Some(notes("Получено по накладной 15")), noted_at)
            .unwrap();

        assert_eq!(supply.notes().unwrap().as_str(), "Получено по накладной 15");
        assert_eq!(*supply.updated_at(), noted_at);

        supply.clear_notes(cleared_at).unwrap();

        assert_eq!(supply.notes(), None);
        assert_eq!(*supply.updated_at(), cleared_at);
    }

    /// Получение переводит поставку в финальный статус.
    #[test]
    fn mark_received_makes_supply_terminal() {
        let now = fixed_time(1_700_000_000);
        let received_at = fixed_time(1_700_000_100);
        let mut supply = expected_supply(now);

        supply.mark_received(received_at).unwrap();

        assert_eq!(supply.status(), PartSupplyStatus::Received);
        assert!(!supply.is_expected());
        assert!(supply.is_terminal());
        assert_eq!(*supply.updated_at(), received_at);
    }

    /// Отмена переводит поставку в финальный статус.
    #[test]
    fn cancel_makes_supply_terminal() {
        let now = fixed_time(1_700_000_000);
        let cancelled_at = fixed_time(1_700_000_100);
        let mut supply = expected_supply(now);

        supply.cancel(cancelled_at).unwrap();

        assert_eq!(supply.status(), PartSupplyStatus::Cancelled);
        assert!(supply.is_terminal());
        assert_eq!(*supply.updated_at(), cancelled_at);
    }

    /// Финальные поставки нельзя редактировать по бизнес-полям.
    #[test]
    fn final_supply_cannot_be_rescheduled_or_requantified() {
        let now = fixed_time(1_700_000_000);
        let received_at = fixed_time(1_700_000_100);
        let changed_at = fixed_time(1_700_000_200);
        let mut supply = expected_supply(now);
        supply.mark_received(received_at).unwrap();

        let reschedule_error = supply
            .reschedule(fixed_time(1_700_200_000), changed_at)
            .unwrap_err();
        let quantity_error = supply
            .update_quantity(PartQuantity::new(8), changed_at)
            .unwrap_err();

        assert_eq!(
            reschedule_error,
            PartSupplyError::CannotModifyFinalSupply {
                status: PartSupplyStatus::Received,
            }
        );
        assert_eq!(
            quantity_error,
            PartSupplyError::CannotModifyFinalSupply {
                status: PartSupplyStatus::Received,
            }
        );
        assert_eq!(supply.quantity(), PartQuantity::new(5));
        assert_eq!(*supply.updated_at(), received_at);
    }

    /// Финальный статус нельзя перевести в другой финальный статус.
    #[test]
    fn final_supply_cannot_transition_again() {
        let now = fixed_time(1_700_000_000);
        let cancelled_at = fixed_time(1_700_000_100);
        let received_at = fixed_time(1_700_000_200);
        let mut supply = expected_supply(now);
        supply.cancel(cancelled_at).unwrap();

        let error = supply.mark_received(received_at).unwrap_err();

        assert_eq!(
            error,
            PartSupplyError::CannotTransitionStatus {
                from: PartSupplyStatus::Cancelled,
                to: PartSupplyStatus::Received,
            }
        );
        assert_eq!(supply.status(), PartSupplyStatus::Cancelled);
        assert_eq!(*supply.updated_at(), cancelled_at);
    }

    /// Некорректное время изменения не должно оставлять частично измененное состояние.
    #[test]
    fn update_rejects_now_before_created_at_without_mutation() {
        let now = fixed_time(1_700_000_000);
        let earlier = fixed_time(1_699_999_999);
        let mut supply = expected_supply(now);

        let error = supply
            .reschedule(fixed_time(1_700_200_000), earlier)
            .unwrap_err();

        assert_eq!(error, PartSupplyError::UpdatedAtBeforeCreatedAt);
        assert_eq!(*supply.expected_at(), fixed_time(1_700_100_000));
        assert_eq!(*supply.updated_at(), now);
    }
}
