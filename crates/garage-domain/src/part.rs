//! Запчасть или расходник склада автосервиса.
//!
//! В этом модуле складская позиция моделируется не как набор свободных строк и
//! чисел, а как доменная сущность с value objects вокруг каждого поля, где есть
//! инварианты. Это важно для Telegram-бота: пользовательский ввод может прийти
//! с пробелами, разным регистром SKU, пустыми необязательными полями и длинными
//! заметками.
//!
//! Дизайн разделен на два слоя:
//! 1. `PartName`, `PartSku`, `PartQuantity` и `PartNotes` нормализуют и
//!    защищают отдельные значения.
//! 2. `Part` управляет жизненным циклом складской позиции: создание,
//!    восстановление из хранилища, редактирование карточки и движение остатка.
//!
//! Главный принцип: после успешного создания value object прикладной слой больше
//! не должен повторять базовые проверки. Он работает с уже валидной моделью, а
//! доменная сущность отвечает за атомарность изменений и корректный `updated_at`.

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::{Money, PartId};

/// Максимальная длина названия запчасти в Unicode-символах.
const MAX_PART_NAME_LEN: usize = 150;
/// Максимальная длина нормализованного артикула в Unicode-символах.
const MAX_PART_SKU_LEN: usize = 100;
/// Максимальная длина заметки по запчасти в Unicode-символах.
const MAX_PART_NOTES_LEN: usize = 1000;

/// Проверенное название запчасти.
///
/// Название обязательно: пустая строка не несет бизнес-смысла и быстро приводит
/// к неразличимым позициям в складском списке. Внутренняя строка закрыта, чтобы
/// нельзя было обойти `parse` и создать невалидное значение напрямую.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PartName(String);

impl PartName {
    /// Разбирает пользовательское название запчасти.
    ///
    /// Алгоритм:
    /// 1. Убираем пробелы по краям, потому что это типичный шум из форм,
    ///    Telegram-сообщений и copy-paste.
    /// 2. Пустой результат отклоняем отдельной ошибкой.
    /// 3. Длину считаем через `chars().count()`, а не через `len()`: лимит
    ///    должен работать в пользовательских символах, а не в байтах UTF-8.
    /// 4. Сохраняем уже обрезанную строку как каноническое значение.
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

    /// Возвращает название без копирования.
    ///
    /// Это основной способ передать значение в UI, репозиторий или сообщение,
    /// не раскрывая возможность изменить внутреннюю строку.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Печатает название в том же виде, в котором оно хранится внутри.
impl std::fmt::Display for PartName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Проверенный артикул или складской код запчасти.
///
/// SKU необязателен: у расходников и разовых позиций его может не быть. Поэтому
/// парсер возвращает `Option<PartSku>`, а не создает специальное пустое значение.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PartSku(String);

impl PartSku {
    /// Разбирает SKU и приводит его к стабильному виду.
    ///
    /// Алгоритм:
    /// 1. Пробелы по краям игнорируются.
    /// 2. Пустая строка означает отсутствие SKU и возвращает `Ok(None)`.
    /// 3. Непустой SKU переводится в верхний регистр. Это делает поиск и
    ///    сравнение устойчивыми к разному пользовательскому вводу: `abc-1` и
    ///    `ABC-1` становятся одним значением.
    /// 4. Лимит проверяется после нормализации, потому что некоторые Unicode
    ///    преобразования регистра могут изменить количество символов.
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

    /// Возвращает нормализованный SKU без копирования.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Печатает SKU в каноническом верхнем регистре.
impl std::fmt::Display for PartSku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Количество запчастей на складе.
///
/// Количество хранится как `u32`, потому что отрицательный остаток запрещен
/// самой моделью. Все операции изменения остатка используют checked-арифметику:
/// переполнение или списание ниже нуля возвращаются как доменные ошибки.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PartQuantity(u32);

impl PartQuantity {
    /// Создает количество из уже проверенного числового значения.
    ///
    /// Любой `u32` валиден: ноль означает отсутствие на складе, положительное
    /// значение - доступный остаток.
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    /// Возвращает нулевой остаток.
    ///
    /// Именованный конструктор делает намерение вызывающего кода явным при
    /// создании отсутствующей позиции или списании до нуля.
    pub fn zero() -> Self {
        Self(0)
    }

    /// Возвращает сырое количество для сохранения, сравнения и отображения.
    pub fn value(&self) -> u32 {
        self.0
    }

    /// Проверяет, равен ли остаток нулю.
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// Безопасно увеличивает количество.
    ///
    /// `checked_add` защищает от тихого переполнения `u32`. В складском домене
    /// переполнение почти всегда означает поврежденные данные или ошибку
    /// интеграции, поэтому его нужно вернуть наружу как явную ошибку.
    pub fn checked_add(self, other: Self) -> Result<Self, PartError> {
        let value = self
            .0
            .checked_add(other.0)
            .ok_or(PartError::QuantityOverflow)?;

        Ok(Self(value))
    }

    /// Безопасно уменьшает количество.
    ///
    /// Если запрошено больше, чем есть на складе, метод возвращает
    /// `InsufficientStock` с двумя числами. Это полезнее простой булевой ошибки:
    /// UI или лог могут показать точный доступный и запрошенный остаток.
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

/// Печатает количество как обычное целое число.
impl std::fmt::Display for PartQuantity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Проверенная заметка по запчасти.
///
/// Заметка необязательна. Отсутствие заметки хранится как `None`, а не как
/// пустая строка, чтобы репозитории и прикладной слой не гадали, является ли
/// пустая строка осознанным значением или просто отсутствием данных.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PartNotes(String);

impl PartNotes {
    /// Разбирает заметку по складской позиции.
    ///
    /// Алгоритм повторяет паттерн необязательных текстовых полей:
    /// 1. Обрезаем пробелы по краям.
    /// 2. Пустой результат превращаем в `Ok(None)`.
    /// 3. Непустой текст ограничиваем по количеству Unicode-символов.
    /// 4. Сохраняем уже нормализованный текст.
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

    /// Возвращает заметку без копирования строки.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Печатает заметку в сохраненном виде.
impl std::fmt::Display for PartNotes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Складская позиция запчасти или расходника.
///
/// `Part` - доменная сущность: у нее есть стабильный `id`, изменяемая карточка
/// и остаток на складе. Поля закрыты намеренно. Изменения проходят через методы,
/// которые сначала проверяют временной инвариант, а затем меняют состояние.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    /// Стабильный идентификатор складской позиции.
    id: PartId,
    /// Проверенное название.
    name: PartName,
    /// Опциональный нормализованный артикул.
    sku: Option<PartSku>,
    /// Текущий остаток на складе.
    quantity: PartQuantity,
    /// Минимальный допустимый остаток для сигнала о необходимости пополнения.
    min_quantity: PartQuantity,
    /// Цена за одну единицу в выбранной валюте.
    unit_price: Money,
    /// Опциональная заметка.
    notes: Option<PartNotes>,
    /// Момент создания позиции.
    created_at: DateTime<Utc>,
    /// Момент последнего изменения позиции.
    updated_at: DateTime<Utc>,
}

impl Part {
    /// Создает новую складскую позицию.
    ///
    /// Метод принимает уже проверенные value objects, поэтому не повторяет
    /// парсинг строк и не возвращает `Result`. Единственный временной параметр
    /// `now` записывается в `created_at` и `updated_at`, потому что новая
    /// позиция еще не менялась после создания.
    #[allow(clippy::too_many_arguments)]
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

    /// Восстанавливает складскую позицию из сохраненного состояния.
    ///
    /// Этот метод предназначен для репозитория. Даты приходят извне, поэтому
    /// домен обязан проверить, что сохраненное состояние не нарушает правило
    /// `created_at <= updated_at`.
    ///
    /// Восстановление не исправляет поврежденные даты автоматически. Тихая
    /// коррекция скрыла бы проблему в данных, а доменная ошибка позволяет
    /// обработать ее явно на уровне приложения или миграции.
    #[allow(clippy::too_many_arguments)]
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

    /// Возвращает идентификатор позиции.
    pub fn id(&self) -> PartId {
        self.id
    }

    /// Возвращает название без копирования строки.
    pub fn name(&self) -> &PartName {
        &self.name
    }

    /// Возвращает SKU, если он был указан.
    pub fn sku(&self) -> Option<&PartSku> {
        self.sku.as_ref()
    }

    /// Возвращает текущий остаток.
    ///
    /// `PartQuantity` маленький и копируемый value object, поэтому его удобно
    /// отдавать по значению.
    pub fn quantity(&self) -> PartQuantity {
        self.quantity
    }

    /// Возвращает минимальный остаток для сигнала о пополнении.
    pub fn min_quantity(&self) -> PartQuantity {
        self.min_quantity
    }

    /// Возвращает цену за единицу.
    ///
    /// `Money` копируемый и неизменяемый, поэтому возврат по значению не ломает
    /// инварианты.
    pub fn unit_price(&self) -> Money {
        self.unit_price
    }

    /// Возвращает заметку, если она есть.
    pub fn notes(&self) -> Option<&PartNotes> {
        self.notes.as_ref()
    }

    /// Возвращает дату создания позиции.
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    /// Возвращает дату последнего изменения позиции.
    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    /// Проверяет, достиг ли остаток минимального порога.
    ///
    /// Используется `<=`, а не `<`: если остаток ровно равен минимальному
    /// уровню, склад уже требует внимания. Это консервативное правило для
    /// автосервиса, где задержка с пополнением расходников может остановить
    /// ремонт.
    pub fn is_low_stock(&self) -> bool {
        self.quantity.value() <= self.min_quantity.value()
    }

    /// Проверяет полное отсутствие позиции на складе.
    pub fn is_out_of_stock(&self) -> bool {
        self.quantity.is_zero()
    }

    /// Меняет название и фиксирует время изменения.
    ///
    /// Сначала вызывается `touch`. Если время некорректно, метод возвращает
    /// ошибку до изменения названия, и сущность не остается в частично
    /// обновленном состоянии.
    pub fn update_name(&mut self, name: PartName, now: DateTime<Utc>) -> Result<(), PartError> {
        self.touch(now)?;
        self.name = name;
        Ok(())
    }

    /// Заменяет SKU и фиксирует время изменения.
    ///
    /// `None` означает осознанное отсутствие артикула. Для удобства публичный
    /// API также содержит `clear_sku`, но оба пути сходятся к одному состоянию.
    pub fn update_sku(
        &mut self,
        sku: Option<PartSku>,
        now: DateTime<Utc>,
    ) -> Result<(), PartError> {
        self.touch(now)?;
        self.sku = sku;
        Ok(())
    }

    /// Удаляет SKU и фиксирует время изменения.
    pub fn clear_sku(&mut self, now: DateTime<Utc>) -> Result<(), PartError> {
        self.touch(now)?;
        self.sku = None;
        Ok(())
    }

    /// Меняет минимальный остаток для сигнала о пополнении.
    ///
    /// Метод не запрещает значение больше текущего остатка. Это валидный
    /// сценарий: после повышения порога позиция сразу становится low stock.
    pub fn update_min_quantity(
        &mut self,
        min_quantity: PartQuantity,
        now: DateTime<Utc>,
    ) -> Result<(), PartError> {
        self.touch(now)?;
        self.min_quantity = min_quantity;
        Ok(())
    }

    /// Меняет цену за единицу.
    ///
    /// Валидность денег уже обеспечена типом `Money`: отрицательная сумма или
    /// арифметическое переполнение не могут попасть сюда как корректное
    /// значение.
    pub fn update_unit_price(
        &mut self,
        unit_price: Money,
        now: DateTime<Utc>,
    ) -> Result<(), PartError> {
        self.touch(now)?;
        self.unit_price = unit_price;
        Ok(())
    }

    /// Заменяет заметку и фиксирует время изменения.
    ///
    /// `None` используется для удаления заметки без отдельной пустой строки.
    pub fn update_notes(
        &mut self,
        notes: Option<PartNotes>,
        now: DateTime<Utc>,
    ) -> Result<(), PartError> {
        self.touch(now)?;
        self.notes = notes;
        Ok(())
    }

    /// Удаляет заметку и фиксирует время изменения.
    pub fn clear_notes(&mut self, now: DateTime<Utc>) -> Result<(), PartError> {
        self.touch(now)?;
        self.notes = None;
        Ok(())
    }

    /// Увеличивает складской остаток.
    ///
    /// Алгоритм специально считает новый остаток до `touch`:
    /// 1. Сначала проверяем арифметику через `PartQuantity::checked_add`.
    /// 2. Если есть переполнение, возвращаем ошибку и не меняем `updated_at`.
    /// 3. Если количество валидно, проверяем время и только потом записываем
    ///    новый остаток.
    ///
    /// Такой порядок сохраняет атомарность доменной операции: ошибка не должна
    /// оставлять след в timestamp-е.
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

    /// Уменьшает складской остаток.
    ///
    /// Списание ниже нуля запрещено. Ошибка `InsufficientStock` содержит
    /// доступный и запрошенный остаток, чтобы вызывающий слой мог показать
    /// понятную причину отказа.
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

    /// Принудительно устанавливает складской остаток.
    ///
    /// Метод нужен для инвентаризации и исправления данных. Он не пытается
    /// вычислить разницу между старым и новым остатком, потому что аудит таких
    /// операций должен жить в отдельной модели событий или журнала.
    pub fn set_stock(
        &mut self,
        quantity: PartQuantity,
        now: DateTime<Utc>,
    ) -> Result<(), PartError> {
        self.touch(now)?;
        self.quantity = quantity;
        Ok(())
    }

    /// Обновляет `updated_at`, сохраняя временной инвариант сущности.
    ///
    /// Метод приватный, потому что сам по себе `touch` не является бизнес-
    /// действием. Его используют публичные методы перед записью нового
    /// состояния. При ошибке публичный метод завершится до мутации поля.
    fn touch(&mut self, now: DateTime<Utc>) -> Result<(), PartError> {
        if now < self.created_at {
            return Err(PartError::UpdatedAtBeforeCreatedAt);
        }

        self.updated_at = now;
        Ok(())
    }
}

/// Ошибки доменной модели складских позиций.
///
/// Ошибки описывают не технический сбой, а нарушение инвариантов: пустое
/// название, превышение лимита, переполнение остатка, попытка списать больше,
/// чем есть, или поврежденный порядок дат.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PartError {
    /// Название пустое после удаления пробелов по краям.
    #[error("part name is empty")]
    EmptyName,

    /// Название превышает допустимый лимит Unicode-символов.
    #[error("part name is too long: max={max}, actual={actual}")]
    NameTooLong { max: usize, actual: usize },

    /// SKU превышает допустимый лимит после нормализации регистра.
    #[error("part sku is too long: max={max}, actual={actual}")]
    SkuTooLong { max: usize, actual: usize },

    /// Заметка превышает допустимый лимит Unicode-символов.
    #[error("part notes are too long: max={max}, actual={actual}")]
    NotesTooLong { max: usize, actual: usize },

    /// Увеличение остатка переполнило `u32`.
    #[error("part quantity overflow")]
    QuantityOverflow,

    /// Запрошено списание большего количества, чем есть на складе.
    #[error("insufficient stock: available={available}, requested={requested}")]
    InsufficientStock { available: u32, requested: u32 },

    /// `updated_at` оказался раньше `created_at`.
    #[error("part updated_at cannot be earlier than created_at")]
    UpdatedAtBeforeCreatedAt,
}

#[cfg(test)]
mod tests {
    use super::{
        Part, PartError, PartName, PartNotes, PartQuantity, PartSku, MAX_PART_NAME_LEN,
        MAX_PART_NOTES_LEN, MAX_PART_SKU_LEN,
    };
    use crate::{Currency, Money, PartId};
    use chrono::{DateTime, Duration, Utc};
    use uuid::Uuid;

    fn fixed_time() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-05-06T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn later_time() -> DateTime<Utc> {
        fixed_time() + Duration::minutes(5)
    }

    fn earlier_time() -> DateTime<Utc> {
        fixed_time() - Duration::minutes(5)
    }

    fn part_id() -> PartId {
        PartId::from_uuid(Uuid::from_u128(42))
    }

    fn money(amount_minor: i64) -> Money {
        Money::byn_minor(amount_minor).unwrap()
    }

    fn part_fixture() -> Part {
        Part::new(
            part_id(),
            PartName::parse("Oil filter").unwrap(),
            PartSku::parse("of-123").unwrap(),
            PartQuantity::new(10),
            PartQuantity::new(3),
            money(2500),
            PartNotes::parse("Original supplier").unwrap(),
            fixed_time(),
        )
    }

    // Базовая валидация обязательного названия.
    #[test]
    fn part_name_parse_trims_valid_name() {
        let name = PartName::parse("  Brake pads  ").unwrap();

        assert_eq!(name.as_str(), "Brake pads");
        assert_eq!(name.to_string(), "Brake pads");
    }

    #[test]
    fn part_name_parse_rejects_empty_name_after_trim() {
        let error = PartName::parse("   ").unwrap_err();

        assert_eq!(error, PartError::EmptyName);
    }

    #[test]
    fn part_name_parse_allows_unicode_name_at_max_length() {
        let input = "ф".repeat(MAX_PART_NAME_LEN);
        let name = PartName::parse(&input).unwrap();

        assert_eq!(name.as_str().chars().count(), MAX_PART_NAME_LEN);
    }

    #[test]
    fn part_name_parse_rejects_too_long_name() {
        let input = "A".repeat(MAX_PART_NAME_LEN + 1);
        let error = PartName::parse(&input).unwrap_err();

        assert_eq!(
            error,
            PartError::NameTooLong {
                max: MAX_PART_NAME_LEN,
                actual: MAX_PART_NAME_LEN + 1
            }
        );
    }

    // SKU необязателен, но непустое значение нормализуется для поиска.
    #[test]
    fn part_sku_parse_normalizes_case_and_whitespace() {
        let sku = PartSku::parse("  ab-12/c  ").unwrap().unwrap();

        assert_eq!(sku.as_str(), "AB-12/C");
        assert_eq!(sku.to_string(), "AB-12/C");
    }

    #[test]
    fn part_sku_parse_returns_none_for_empty_input() {
        let sku = PartSku::parse("   ").unwrap();

        assert_eq!(sku, None);
    }

    #[test]
    fn part_sku_parse_allows_sku_at_max_length() {
        let input = "a".repeat(MAX_PART_SKU_LEN);
        let sku = PartSku::parse(&input).unwrap().unwrap();

        assert_eq!(sku.as_str().chars().count(), MAX_PART_SKU_LEN);
        assert!(sku.as_str().chars().all(|ch| ch == 'A'));
    }

    #[test]
    fn part_sku_parse_rejects_too_long_sku_after_normalization() {
        let input = "a".repeat(MAX_PART_SKU_LEN + 1);
        let error = PartSku::parse(&input).unwrap_err();

        assert_eq!(
            error,
            PartError::SkuTooLong {
                max: MAX_PART_SKU_LEN,
                actual: MAX_PART_SKU_LEN + 1
            }
        );
    }

    // Остаток проверяется через checked-арифметику без отрицательных значений.
    #[test]
    fn part_quantity_exposes_value_zero_and_display() {
        let zero = PartQuantity::zero();
        let non_zero = PartQuantity::new(7);

        assert_eq!(zero.value(), 0);
        assert!(zero.is_zero());
        assert!(!non_zero.is_zero());
        assert_eq!(non_zero.to_string(), "7");
    }

    #[test]
    fn part_quantity_checked_add_adds_values() {
        let quantity = PartQuantity::new(4)
            .checked_add(PartQuantity::new(6))
            .unwrap();

        assert_eq!(quantity.value(), 10);
    }

    #[test]
    fn part_quantity_checked_add_rejects_overflow() {
        let error = PartQuantity::new(u32::MAX)
            .checked_add(PartQuantity::new(1))
            .unwrap_err();

        assert_eq!(error, PartError::QuantityOverflow);
    }

    #[test]
    fn part_quantity_checked_sub_subtracts_values() {
        let quantity = PartQuantity::new(10)
            .checked_sub(PartQuantity::new(4))
            .unwrap();

        assert_eq!(quantity.value(), 6);
    }

    #[test]
    fn part_quantity_checked_sub_rejects_insufficient_stock() {
        let error = PartQuantity::new(2)
            .checked_sub(PartQuantity::new(5))
            .unwrap_err();

        assert_eq!(
            error,
            PartError::InsufficientStock {
                available: 2,
                requested: 5
            }
        );
    }

    // Заметка ведет себя как optional text field: пустой ввод превращается в None.
    #[test]
    fn part_notes_parse_trims_non_empty_notes() {
        let notes = PartNotes::parse("  Shelf A1  ").unwrap().unwrap();

        assert_eq!(notes.as_str(), "Shelf A1");
        assert_eq!(notes.to_string(), "Shelf A1");
    }

    #[test]
    fn part_notes_parse_returns_none_for_empty_input() {
        let notes = PartNotes::parse("   ").unwrap();

        assert_eq!(notes, None);
    }

    #[test]
    fn part_notes_parse_allows_unicode_notes_at_max_length() {
        let input = "ж".repeat(MAX_PART_NOTES_LEN);
        let notes = PartNotes::parse(&input).unwrap().unwrap();

        assert_eq!(notes.as_str().chars().count(), MAX_PART_NOTES_LEN);
    }

    #[test]
    fn part_notes_parse_rejects_too_long_notes() {
        let input = "N".repeat(MAX_PART_NOTES_LEN + 1);
        let error = PartNotes::parse(&input).unwrap_err();

        assert_eq!(
            error,
            PartError::NotesTooLong {
                max: MAX_PART_NOTES_LEN,
                actual: MAX_PART_NOTES_LEN + 1
            }
        );
    }

    // Создание и восстановление фиксируют ключевой инвариант дат.
    #[test]
    fn part_new_sets_initial_state_and_timestamps() {
        let now = fixed_time();
        let part = part_fixture();

        assert_eq!(part.id(), part_id());
        assert_eq!(part.name().as_str(), "Oil filter");
        assert_eq!(part.sku().unwrap().as_str(), "OF-123");
        assert_eq!(part.quantity(), PartQuantity::new(10));
        assert_eq!(part.min_quantity(), PartQuantity::new(3));
        assert_eq!(part.unit_price(), money(2500));
        assert_eq!(part.notes().unwrap().as_str(), "Original supplier");
        assert_eq!(*part.created_at(), now);
        assert_eq!(*part.updated_at(), now);
    }

    #[test]
    fn part_restore_accepts_valid_persisted_state() {
        let created_at = fixed_time();
        let updated_at = later_time();
        let part = Part::restore(
            part_id(),
            PartName::parse("Air filter").unwrap(),
            None,
            PartQuantity::new(4),
            PartQuantity::new(1),
            money(1800),
            None,
            created_at,
            updated_at,
        )
        .unwrap();

        assert_eq!(part.name().as_str(), "Air filter");
        assert_eq!(part.sku(), None);
        assert_eq!(part.quantity(), PartQuantity::new(4));
        assert_eq!(part.notes(), None);
        assert_eq!(*part.created_at(), created_at);
        assert_eq!(*part.updated_at(), updated_at);
    }

    #[test]
    fn part_restore_rejects_updated_at_before_created_at() {
        let error = Part::restore(
            part_id(),
            PartName::parse("Air filter").unwrap(),
            None,
            PartQuantity::new(4),
            PartQuantity::new(1),
            money(1800),
            None,
            fixed_time(),
            earlier_time(),
        )
        .unwrap_err();

        assert_eq!(error, PartError::UpdatedAtBeforeCreatedAt);
    }

    // Low stock срабатывает на границе минимального остатка.
    #[test]
    fn stock_status_distinguishes_low_and_out_of_stock() {
        let low = Part::new(
            part_id(),
            PartName::parse("Low").unwrap(),
            None,
            PartQuantity::new(3),
            PartQuantity::new(3),
            money(100),
            None,
            fixed_time(),
        );
        let available = Part::new(
            part_id(),
            PartName::parse("Available").unwrap(),
            None,
            PartQuantity::new(4),
            PartQuantity::new(3),
            money(100),
            None,
            fixed_time(),
        );
        let empty = Part::new(
            part_id(),
            PartName::parse("Empty").unwrap(),
            None,
            PartQuantity::zero(),
            PartQuantity::new(3),
            money(100),
            None,
            fixed_time(),
        );

        assert!(low.is_low_stock());
        assert!(!low.is_out_of_stock());
        assert!(!available.is_low_stock());
        assert!(empty.is_low_stock());
        assert!(empty.is_out_of_stock());
    }

    // Редактирование карточки обновляет timestamp и не мутирует состояние при ошибке.
    #[test]
    fn update_name_changes_name_and_updates_timestamp() {
        let mut part = part_fixture();

        part.update_name(PartName::parse("Fuel filter").unwrap(), later_time())
            .unwrap();

        assert_eq!(part.name().as_str(), "Fuel filter");
        assert_eq!(*part.updated_at(), later_time());
    }

    #[test]
    fn update_name_rejects_time_before_creation_without_changes() {
        let mut part = part_fixture();
        let before = part.clone();

        let error = part
            .update_name(PartName::parse("Fuel filter").unwrap(), earlier_time())
            .unwrap_err();

        assert_eq!(error, PartError::UpdatedAtBeforeCreatedAt);
        assert_eq!(part, before);
    }

    #[test]
    fn update_sku_replaces_and_clears_sku() {
        let mut part = part_fixture();

        part.update_sku(PartSku::parse("new-1").unwrap(), later_time())
            .unwrap();
        assert_eq!(part.sku().unwrap().as_str(), "NEW-1");
        assert_eq!(*part.updated_at(), later_time());

        let next_time = later_time() + Duration::minutes(1);
        part.update_sku(None, next_time).unwrap();

        assert_eq!(part.sku(), None);
        assert_eq!(*part.updated_at(), next_time);
    }

    #[test]
    fn clear_sku_removes_sku_and_updates_timestamp() {
        let mut part = part_fixture();

        part.clear_sku(later_time()).unwrap();

        assert_eq!(part.sku(), None);
        assert_eq!(*part.updated_at(), later_time());
    }

    #[test]
    fn update_min_quantity_changes_threshold_and_low_stock_state() {
        let mut part = part_fixture();

        part.update_min_quantity(PartQuantity::new(12), later_time())
            .unwrap();

        assert_eq!(part.min_quantity(), PartQuantity::new(12));
        assert!(part.is_low_stock());
        assert_eq!(*part.updated_at(), later_time());
    }

    #[test]
    fn update_unit_price_changes_price_and_updates_timestamp() {
        let mut part = part_fixture();

        part.update_unit_price(money(3000), later_time()).unwrap();

        assert_eq!(part.unit_price(), money(3000));
        assert_eq!(part.unit_price().currency(), Currency::Byn);
        assert_eq!(*part.updated_at(), later_time());
    }

    #[test]
    fn update_notes_replaces_and_removes_notes() {
        let mut part = part_fixture();

        part.update_notes(PartNotes::parse("Second shelf").unwrap(), later_time())
            .unwrap();
        assert_eq!(part.notes().unwrap().as_str(), "Second shelf");
        assert_eq!(*part.updated_at(), later_time());

        let next_time = later_time() + Duration::minutes(1);
        part.update_notes(None, next_time).unwrap();

        assert_eq!(part.notes(), None);
        assert_eq!(*part.updated_at(), next_time);
    }

    #[test]
    fn clear_notes_removes_notes_and_updates_timestamp() {
        let mut part = part_fixture();

        part.clear_notes(later_time()).unwrap();

        assert_eq!(part.notes(), None);
        assert_eq!(*part.updated_at(), later_time());
    }

    // Складские операции атомарны: ошибка не должна менять остаток или updated_at.
    #[test]
    fn increase_stock_adds_quantity_and_updates_timestamp() {
        let mut part = part_fixture();

        part.increase_stock(PartQuantity::new(5), later_time())
            .unwrap();

        assert_eq!(part.quantity(), PartQuantity::new(15));
        assert_eq!(*part.updated_at(), later_time());
    }

    #[test]
    fn increase_stock_rejects_overflow_without_changes() {
        let mut part = Part::new(
            part_id(),
            PartName::parse("Huge stock").unwrap(),
            None,
            PartQuantity::new(u32::MAX),
            PartQuantity::new(1),
            money(100),
            None,
            fixed_time(),
        );
        let before = part.clone();

        let error = part
            .increase_stock(PartQuantity::new(1), later_time())
            .unwrap_err();

        assert_eq!(error, PartError::QuantityOverflow);
        assert_eq!(part, before);
    }

    #[test]
    fn decrease_stock_subtracts_quantity_and_updates_timestamp() {
        let mut part = part_fixture();

        part.decrease_stock(PartQuantity::new(4), later_time())
            .unwrap();

        assert_eq!(part.quantity(), PartQuantity::new(6));
        assert_eq!(*part.updated_at(), later_time());
    }

    #[test]
    fn decrease_stock_rejects_insufficient_stock_without_changes() {
        let mut part = part_fixture();
        let before = part.clone();

        let error = part
            .decrease_stock(PartQuantity::new(11), later_time())
            .unwrap_err();

        assert_eq!(
            error,
            PartError::InsufficientStock {
                available: 10,
                requested: 11
            }
        );
        assert_eq!(part, before);
    }

    #[test]
    fn set_stock_replaces_quantity_and_updates_timestamp() {
        let mut part = part_fixture();

        part.set_stock(PartQuantity::zero(), later_time()).unwrap();

        assert_eq!(part.quantity(), PartQuantity::zero());
        assert!(part.is_out_of_stock());
        assert_eq!(*part.updated_at(), later_time());
    }

    #[test]
    fn set_stock_rejects_time_before_creation_without_changes() {
        let mut part = part_fixture();
        let before = part.clone();

        let error = part
            .set_stock(PartQuantity::new(20), earlier_time())
            .unwrap_err();

        assert_eq!(error, PartError::UpdatedAtBeforeCreatedAt);
        assert_eq!(part, before);
    }
}
