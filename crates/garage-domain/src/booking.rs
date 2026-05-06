//! Запись клиента на обслуживание автомобиля.
//!
//! Модуль моделирует booking как доменную сущность, а не как свободный набор
//! полей из формы. Это важно для Telegram-бота: пользовательский ввод приходит
//! строками, но после входной нормализации остальная система должна работать с
//! проверенными типами и предсказуемым жизненным циклом записи.
//!
//! Алгоритмически модуль разделен на два слоя:
//! 1. Value objects (`BookingReason`, `BookingNotes`) нормализуют текст: убирают
//!    внешние пробелы, проверяют обязательность и считают длину в
//!    Unicode-символах.
//! 2. `Booking` управляет состоянием записи: создание, восстановление из
//!    хранилища, перенос времени, редактирование описания и финальные переходы
//!    статуса.
//!
//! Центральный инвариант жизненного цикла: запись создается в статусе
//! `Scheduled`, а завершить ее можно только одним финальным статусом:
//! `Completed`, `Cancelled` или `NoShow`. После финального статуса перенос и
//! изменение причины запрещены, чтобы случайно не переписать историю приема.
//! Заметки можно менять отдельно: это операционный комментарий администратора
//! или мастера, а не бизнес-факт записи.

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::{BookingId, CarId, ClientId};

/// Максимальная длина причины записи в Unicode-символах.
///
/// Причина должна быть короткой и пригодной для списка записей: например,
/// `диагностика`, `замена масла`, `стук в подвеске`.
const MAX_BOOKING_REASON_LEN: usize = 200;
/// Максимальная длина заметки по записи в Unicode-символах.
const MAX_BOOKING_NOTES_LEN: usize = 1000;

/// Текущее состояние записи на обслуживание.
///
/// Статус намеренно компактный: домену важно отличать активную запись
/// (`Scheduled`) от трех финальных исходов. Финальные статусы не переводятся
/// друг в друга, потому что это уже зафиксированный результат визита клиента.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BookingStatus {
    /// Клиент записан, визит еще не обработан.
    Scheduled,
    /// Клиент приехал, обслуживание по записи завершено.
    Completed,
    /// Запись отменена до визита.
    Cancelled,
    /// Клиент не приехал и запись закрыта как неявка.
    NoShow,
}

/// Человекочитаемое представление статуса для ошибок, логов и простого UI.
impl std::fmt::Display for BookingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BookingStatus::Scheduled => write!(f, "scheduled"),
            BookingStatus::Completed => write!(f, "completed"),
            BookingStatus::Cancelled => write!(f, "cancelled"),
            BookingStatus::NoShow => write!(f, "no_show"),
        }
    }
}

/// Причина записи на обслуживание.
///
/// Внутренняя строка закрыта, чтобы нельзя было создать пустую или слишком
/// длинную причину в обход `parse`. После успешного парсинга доменная сущность
/// может полагаться на два инварианта: причина не пустая и не длиннее
/// `MAX_BOOKING_REASON_LEN`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BookingReason(String);

impl BookingReason {
    /// Нормализует пользовательский ввод и создает валидную причину записи.
    ///
    /// Алгоритм:
    /// 1. Убираем пробелы по краям. Это обычный шум из сообщений и форм.
    /// 2. Проверяем пустоту после `trim`: строка из одних пробелов не является
    ///    причиной записи.
    /// 3. Считаем длину через `chars().count()`, а не через `len()`, чтобы
    ///    кириллица и другие Unicode-символы считались как символы, а не как
    ///    UTF-8 байты.
    /// 4. При превышении лимита возвращаем структурированную ошибку с `max` и
    ///    `actual`. Это позволяет прикладному слою показать точное сообщение.
    /// 5. Сохраняем уже обрезанную строку как канонический внутренний формат.
    pub fn parse(input: &str) -> Result<Self, BookingError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Err(BookingError::EmptyReason);
        }

        let actual = trimmed.chars().count();

        if actual > MAX_BOOKING_REASON_LEN {
            return Err(BookingError::ReasonTooLong {
                max: MAX_BOOKING_REASON_LEN,
                actual,
            });
        }

        Ok(Self(trimmed.to_string()))
    }

    /// Возвращает причину без копирования и без передачи владения строкой.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Показывает причину ровно в том виде, который был сохранен после `parse`.
impl std::fmt::Display for BookingReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Заметка по записи.
///
/// Заметка опциональна: пустой ввод не превращается в `BookingNotes`, а
/// представляется как `None`. Так в домене остается один способ сказать
/// "заметки нет", без неоднозначности между `None` и `Some("")`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BookingNotes(String);

impl BookingNotes {
    /// Нормализует пользовательскую заметку.
    ///
    /// Алгоритм:
    /// 1. Убираем внешние пробелы.
    /// 2. Если после `trim` строка пустая, возвращаем `Ok(None)`. Это не
    ///    ошибка: пользователь просто не оставил комментарий.
    /// 3. Считаем длину в Unicode-символах, чтобы русские комментарии не
    ///    штрафовались из-за многобайтового UTF-8.
    /// 4. При превышении лимита возвращаем `NotesTooLong`.
    /// 5. Иначе сохраняем trimmed-текст как `Some(BookingNotes)`.
    pub fn parse(input: &str) -> Result<Option<Self>, BookingError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(None);
        }

        let actual = trimmed.chars().count();

        if actual > MAX_BOOKING_NOTES_LEN {
            return Err(BookingError::NotesTooLong {
                max: MAX_BOOKING_NOTES_LEN,
                actual,
            });
        }

        Ok(Some(Self(trimmed.to_string())))
    }

    /// Возвращает текст заметки без копирования.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Показывает заметку без дополнительного форматирования.
impl std::fmt::Display for BookingNotes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Запись клиента на обслуживание конкретного автомобиля.
///
/// `Booking` - доменная сущность: у нее есть стабильный `id`, связи с клиентом
/// и автомобилем, запланированное время и управляемый статус. Все поля закрыты,
/// чтобы изменения проходили через методы сущности и не обходили правила
/// переходов статуса или обновления `updated_at`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Booking {
    /// Стабильный идентификатор записи.
    id: BookingId,
    /// Клиент, который записан на обслуживание.
    client_id: ClientId,
    /// Автомобиль клиента, по которому создана запись.
    car_id: CarId,
    /// Запланированная дата и время визита.
    scheduled_at: DateTime<Utc>,
    /// Текущий статус записи.
    status: BookingStatus,
    /// Проверенная причина обращения.
    reason: BookingReason,
    /// Опциональная внутренняя заметка по записи.
    notes: Option<BookingNotes>,
    /// Момент создания записи.
    created_at: DateTime<Utc>,
    /// Момент последнего изменения записи.
    updated_at: DateTime<Utc>,
}

impl Booking {
    /// Создает новую запись.
    ///
    /// Алгоритм создания:
    /// 1. Вызывающий слой заранее превращает пользовательский ввод в value
    ///    objects (`BookingReason`, `BookingNotes`) и передает типобезопасные
    ///    идентификаторы клиента и автомобиля.
    /// 2. Новая запись всегда получает статус `Scheduled`. Другие статусы
    ///    означают уже случившийся исход и не должны появляться при создании.
    /// 3. Один момент времени `now` записывается и в `created_at`, и в
    ///    `updated_at`, потому что после создания запись еще не менялась.
    ///
    /// Метод не возвращает `Result`: при таких входных типах невозможно
    /// нарушить обязательные текстовые инварианты или порядок дат.
    pub fn new(
        id: BookingId,
        client_id: ClientId,
        car_id: CarId,
        scheduled_at: DateTime<Utc>,
        reason: BookingReason,
        notes: Option<BookingNotes>,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            client_id,
            car_id,
            scheduled_at,
            status: BookingStatus::Scheduled,
            reason,
            notes,
            created_at: now,
            updated_at: now,
        }
    }

    /// Восстанавливает запись из уже существующего состояния.
    ///
    /// Этот метод нужен репозиторию при чтении из базы данных. В отличие от
    /// `new`, статус и даты приходят извне, поэтому домен обязан проверить хотя
    /// бы временной порядок.
    ///
    /// Алгоритм восстановления:
    /// 1. Репозиторий восстанавливает идентификаторы и value objects из
    ///    сохраненных значений.
    /// 2. `restore` сравнивает `updated_at` и `created_at`.
    /// 3. Если обновление раньше создания, состояние считается поврежденным и
    ///    возвращается `UpdatedAtBeforeCreatedAt`.
    /// 4. Иначе сущность собирается без изменения переданных дат и статуса.
    #[allow(clippy::too_many_arguments)]
    pub fn restore(
        id: BookingId,
        client_id: ClientId,
        car_id: CarId,
        scheduled_at: DateTime<Utc>,
        status: BookingStatus,
        reason: BookingReason,
        notes: Option<BookingNotes>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, BookingError> {
        if updated_at < created_at {
            return Err(BookingError::UpdatedAtBeforeCreatedAt);
        }

        Ok(Self {
            id,
            client_id,
            car_id,
            scheduled_at,
            status,
            reason,
            notes,
            created_at,
            updated_at,
        })
    }

    /// Возвращает идентификатор записи.
    pub fn id(&self) -> BookingId {
        self.id
    }

    /// Возвращает идентификатор клиента.
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    /// Возвращает идентификатор автомобиля.
    pub fn car_id(&self) -> CarId {
        self.car_id
    }

    /// Возвращает запланированное время визита.
    pub fn scheduled_at(&self) -> &DateTime<Utc> {
        &self.scheduled_at
    }

    /// Возвращает текущий статус записи.
    pub fn status(&self) -> BookingStatus {
        self.status
    }

    /// Возвращает проверенную причину обращения.
    pub fn reason(&self) -> &BookingReason {
        &self.reason
    }

    /// Возвращает заметку, если она есть.
    ///
    /// Наружу отдается `Option<&BookingNotes>`, чтобы не копировать строку и не
    /// позволять вызывающему коду менять внутреннее состояние напрямую.
    pub fn notes(&self) -> Option<&BookingNotes> {
        self.notes.as_ref()
    }

    /// Возвращает дату создания записи.
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    /// Возвращает дату последнего изменения записи.
    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    /// Проверяет, остается ли запись активной.
    pub fn is_scheduled(&self) -> bool {
        self.status == BookingStatus::Scheduled
    }

    /// Проверяет, закрыта ли запись одним из финальных исходов.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            BookingStatus::Completed | BookingStatus::Cancelled | BookingStatus::NoShow
        )
    }

    /// Переносит запись на другое время.
    ///
    /// Алгоритм:
    /// 1. Проверяем, что запись все еще `Scheduled`. Финальные записи нельзя
    ///    переносить, иначе мы перепишем уже закрытую историю.
    /// 2. Через `touch` проверяем временной инвариант `created_at <= updated_at`.
    /// 3. Только после успешных проверок меняем `scheduled_at`.
    ///
    /// Порядок важен: при ошибке запись остается полностью неизменной.
    pub fn reschedule(
        &mut self,
        scheduled_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Result<(), BookingError> {
        self.ensure_scheduled_for_modification()?;
        self.touch(now)?;
        self.scheduled_at = scheduled_at;
        Ok(())
    }

    /// Меняет причину обращения.
    ///
    /// Причина является частью бизнес-смысла активной записи, поэтому ее можно
    /// менять только пока запись не закрыта финальным статусом.
    pub fn update_reason(
        &mut self,
        reason: BookingReason,
        now: DateTime<Utc>,
    ) -> Result<(), BookingError> {
        self.ensure_scheduled_for_modification()?;
        self.touch(now)?;
        self.reason = reason;
        Ok(())
    }

    /// Заменяет заметку по записи и фиксирует момент изменения.
    ///
    /// В отличие от причины, заметка считается операционным комментарием.
    /// Поэтому метод не требует статус `Scheduled`: администратор или мастер
    /// может дописать пояснение уже после завершения, отмены или неявки.
    pub fn update_notes(
        &mut self,
        notes: Option<BookingNotes>,
        now: DateTime<Utc>,
    ) -> Result<(), BookingError> {
        self.touch(now)?;
        self.notes = notes;
        Ok(())
    }

    /// Удаляет заметку и фиксирует момент изменения.
    ///
    /// Это явный сценарный метод поверх `update_notes(None, now)`. Он делает
    /// вызывающий код читаемее, когда пользователь именно очищает заметку.
    pub fn clear_notes(&mut self, now: DateTime<Utc>) -> Result<(), BookingError> {
        self.touch(now)?;
        self.notes = None;
        Ok(())
    }

    /// Закрывает запись как успешно завершенную.
    pub fn complete(&mut self, now: DateTime<Utc>) -> Result<(), BookingError> {
        self.transition_to(BookingStatus::Completed, now)
    }

    /// Закрывает запись как отмененную.
    pub fn cancel(&mut self, now: DateTime<Utc>) -> Result<(), BookingError> {
        self.transition_to(BookingStatus::Cancelled, now)
    }

    /// Закрывает запись как неявку клиента.
    pub fn mark_no_show(&mut self, now: DateTime<Utc>) -> Result<(), BookingError> {
        self.transition_to(BookingStatus::NoShow, now)
    }

    /// Обновляет `updated_at`, сохраняя временной инвариант сущности.
    ///
    /// Алгоритм:
    /// 1. Сравниваем новое время с `created_at`.
    /// 2. Если новое время раньше создания, возвращаем ошибку и не меняем
    ///    состояние записи.
    /// 3. Если время корректно, записываем его в `updated_at`.
    ///
    /// Метод приватный: это технический шаг жизненного цикла. Публичные методы
    /// вызывают его до изменения конкретного поля, поэтому при ошибке запись не
    /// остается в частично измененном состоянии.
    fn touch(&mut self, now: DateTime<Utc>) -> Result<(), BookingError> {
        if now < self.created_at {
            return Err(BookingError::UpdatedAtBeforeCreatedAt);
        }

        self.updated_at = now;
        Ok(())
    }

    /// Проверяет, что активную запись можно редактировать.
    ///
    /// Метод используется для операций, которые меняют бизнес-содержание
    /// будущего визита: время и причину. Финальный статус для таких операций
    /// является стоп-сигналом.
    fn ensure_scheduled_for_modification(&self) -> Result<(), BookingError> {
        if self.status != BookingStatus::Scheduled {
            return Err(BookingError::CannotModifyFinalBooking {
                status: self.status,
            });
        }

        Ok(())
    }

    /// Проверяет, что статус можно перевести в выбранный финальный исход.
    ///
    /// В текущей модели допустим только переход `Scheduled -> final`. Переходы
    /// между финальными статусами запрещены: если запись уже закрыта как
    /// `Cancelled`, нельзя без отдельного сценария превратить ее в `Completed`.
    fn ensure_scheduled_for_transition(&self, to: BookingStatus) -> Result<(), BookingError> {
        if self.status != BookingStatus::Scheduled {
            return Err(BookingError::CannotTransitionStatus {
                from: self.status,
                to,
            });
        }

        Ok(())
    }

    /// Выполняет общий алгоритм финального перехода статуса.
    ///
    /// Алгоритм:
    /// 1. Проверяем, что текущий статус допускает переход в `status`.
    /// 2. Обновляем `updated_at` через `touch`.
    /// 3. Записываем новый статус.
    ///
    /// Все публичные методы (`complete`, `cancel`, `mark_no_show`) используют
    /// один путь, чтобы правила переходов не расходились между сценариями.
    fn transition_to(
        &mut self,
        status: BookingStatus,
        now: DateTime<Utc>,
    ) -> Result<(), BookingError> {
        self.ensure_scheduled_for_transition(status)?;
        self.touch(now)?;
        self.status = status;
        Ok(())
    }
}

/// Ошибка работы с записью и ее value objects.
///
/// Ошибки находятся рядом с доменными типами, потому что именно этот модуль
/// определяет правила валидности причины, заметок, временных меток и переходов
/// статуса записи.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BookingError {
    /// Причина пустая после удаления пробелов по краям.
    #[error("booking reason is empty")]
    EmptyReason,

    /// Причина превышает допустимую длину в Unicode-символах.
    #[error("booking reason is too long: max={max}, actual={actual}")]
    ReasonTooLong { max: usize, actual: usize },

    /// Заметка превышает допустимую длину в Unicode-символах.
    #[error("booking notes are too long: max={max}, actual={actual}")]
    NotesTooLong { max: usize, actual: usize },

    /// Восстановленное или обновленное состояние нарушает временной порядок.
    #[error("booking updated_at cannot be earlier than created_at")]
    UpdatedAtBeforeCreatedAt,

    /// Попытка изменить бизнес-поля записи после финального статуса.
    #[error("cannot modify booking with final status {status}")]
    CannotModifyFinalBooking { status: BookingStatus },

    /// Попытка выполнить недопустимый переход статуса.
    #[error("cannot transition booking status from {from} to {to}")]
    CannotTransitionStatus {
        from: BookingStatus,
        to: BookingStatus,
    },
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use super::{
        Booking, BookingError, BookingNotes, BookingReason, BookingStatus, MAX_BOOKING_NOTES_LEN,
        MAX_BOOKING_REASON_LEN,
    };
    use crate::{BookingId, CarId, ClientId};

    fn fixed_time(seconds: i64) -> chrono::DateTime<Utc> {
        Utc.timestamp_opt(seconds, 0).single().unwrap()
    }

    fn booking_id() -> BookingId {
        BookingId::from_uuid(Uuid::from_u128(1))
    }

    fn client_id() -> ClientId {
        ClientId::from_uuid(Uuid::from_u128(2))
    }

    fn car_id() -> CarId {
        CarId::from_uuid(Uuid::from_u128(3))
    }

    fn reason(value: &str) -> BookingReason {
        BookingReason::parse(value).unwrap()
    }

    fn notes(value: &str) -> BookingNotes {
        BookingNotes::parse(value).unwrap().unwrap()
    }

    fn scheduled_booking(now: chrono::DateTime<Utc>) -> Booking {
        Booking::new(
            booking_id(),
            client_id(),
            car_id(),
            fixed_time(1_700_010_000),
            reason("Диагностика подвески"),
            Some(notes("Клиент просил утром")),
            now,
        )
    }

    /// Причина очищается от внешних пробелов и сохраняется в каноническом виде.
    #[test]
    fn booking_reason_parse_trims_valid_reason() {
        let reason = BookingReason::parse("  Замена масла  ").unwrap();

        assert_eq!(reason.as_str(), "Замена масла");
        assert_eq!(reason.to_string(), "Замена масла");
    }

    /// Строка из пробелов не должна становиться причиной записи.
    #[test]
    fn booking_reason_parse_rejects_empty_reason_after_trim() {
        let error = BookingReason::parse("   ").unwrap_err();

        assert_eq!(error, BookingError::EmptyReason);
    }

    /// Лимит причины считается в Unicode-символах, а не в байтах.
    #[test]
    fn booking_reason_parse_allows_unicode_reason_at_max_length() {
        let input = "я".repeat(MAX_BOOKING_REASON_LEN);

        let reason = BookingReason::parse(&input).unwrap();

        assert_eq!(reason.as_str().chars().count(), MAX_BOOKING_REASON_LEN);
    }

    /// При превышении лимита причина возвращает структурированную ошибку.
    #[test]
    fn booking_reason_parse_rejects_too_long_reason() {
        let input = "a".repeat(MAX_BOOKING_REASON_LEN + 1);

        let error = BookingReason::parse(&input).unwrap_err();

        assert_eq!(
            error,
            BookingError::ReasonTooLong {
                max: MAX_BOOKING_REASON_LEN,
                actual: MAX_BOOKING_REASON_LEN + 1,
            }
        );
    }

    /// Непустая заметка очищается от пробелов и возвращается как `Some`.
    #[test]
    fn booking_notes_parse_trims_non_empty_notes() {
        let notes = BookingNotes::parse("  Проверить чек двигателя  ")
            .unwrap()
            .unwrap();

        assert_eq!(notes.as_str(), "Проверить чек двигателя");
        assert_eq!(notes.to_string(), "Проверить чек двигателя");
    }

    /// Пустая заметка не является ошибкой: в домене она представляется как
    /// отсутствие значения.
    #[test]
    fn booking_notes_parse_returns_none_for_empty_input() {
        let notes = BookingNotes::parse("   ").unwrap();

        assert!(notes.is_none());
    }

    /// Заметки считаются в Unicode-символах, что важно для русскоязычных
    /// комментариев администратора и мастера.
    #[test]
    fn booking_notes_parse_allows_unicode_notes_at_max_length() {
        let input = "ю".repeat(MAX_BOOKING_NOTES_LEN);

        let notes = BookingNotes::parse(&input).unwrap().unwrap();

        assert_eq!(notes.as_str().chars().count(), MAX_BOOKING_NOTES_LEN);
    }

    /// Слишком длинная заметка возвращает структурированную ошибку с лимитами.
    #[test]
    fn booking_notes_parse_rejects_too_long_notes() {
        let input = "a".repeat(MAX_BOOKING_NOTES_LEN + 1);

        let error = BookingNotes::parse(&input).unwrap_err();

        assert_eq!(
            error,
            BookingError::NotesTooLong {
                max: MAX_BOOKING_NOTES_LEN,
                actual: MAX_BOOKING_NOTES_LEN + 1,
            }
        );
    }

    /// Новый booking получает статус `Scheduled` и одинаковые даты создания и
    /// обновления, потому что после создания изменений еще не было.
    #[test]
    fn booking_new_sets_initial_state_and_timestamps() {
        let now = fixed_time(1_700_000_000);
        let scheduled_at = fixed_time(1_700_010_000);
        let booking = Booking::new(
            booking_id(),
            client_id(),
            car_id(),
            scheduled_at,
            reason("Замена масла"),
            Some(notes("С фильтром клиента")),
            now,
        );

        assert_eq!(booking.id(), booking_id());
        assert_eq!(booking.client_id(), client_id());
        assert_eq!(booking.car_id(), car_id());
        assert_eq!(*booking.scheduled_at(), scheduled_at);
        assert_eq!(booking.status(), BookingStatus::Scheduled);
        assert_eq!(booking.reason().as_str(), "Замена масла");
        assert_eq!(booking.notes().unwrap().as_str(), "С фильтром клиента");
        assert_eq!(*booking.created_at(), now);
        assert_eq!(*booking.updated_at(), now);
        assert!(booking.is_scheduled());
        assert!(!booking.is_terminal());
    }

    /// Restore принимает сохраненное состояние, если порядок дат корректен.
    #[test]
    fn booking_restore_accepts_valid_persisted_state() {
        let created_at = fixed_time(1_700_000_000);
        let updated_at = fixed_time(1_700_000_500);
        let scheduled_at = fixed_time(1_700_010_000);

        let booking = Booking::restore(
            booking_id(),
            client_id(),
            car_id(),
            scheduled_at,
            BookingStatus::Cancelled,
            reason("Диагностика"),
            None,
            created_at,
            updated_at,
        )
        .unwrap();

        assert_eq!(booking.status(), BookingStatus::Cancelled);
        assert_eq!(*booking.scheduled_at(), scheduled_at);
        assert_eq!(*booking.created_at(), created_at);
        assert_eq!(*booking.updated_at(), updated_at);
        assert!(booking.notes().is_none());
        assert!(!booking.is_scheduled());
        assert!(booking.is_terminal());
    }

    /// Данные из БД с `updated_at` раньше `created_at` не должны попадать в
    /// доменную модель как валидная запись.
    #[test]
    fn booking_restore_rejects_updated_at_before_created_at() {
        let error = Booking::restore(
            booking_id(),
            client_id(),
            car_id(),
            fixed_time(1_700_010_000),
            BookingStatus::Scheduled,
            reason("Диагностика"),
            None,
            fixed_time(1_700_000_500),
            fixed_time(1_700_000_000),
        )
        .unwrap_err();

        assert_eq!(error, BookingError::UpdatedAtBeforeCreatedAt);
    }

    /// Перенос меняет только время визита и `updated_at`, оставляя остальные
    /// бизнес-поля без изменений.
    #[test]
    fn reschedule_changes_scheduled_at_and_updates_timestamp() {
        let created_at = fixed_time(1_700_000_000);
        let updated_at = fixed_time(1_700_000_100);
        let new_scheduled_at = fixed_time(1_700_020_000);
        let mut booking = scheduled_booking(created_at);

        booking.reschedule(new_scheduled_at, updated_at).unwrap();

        assert_eq!(*booking.scheduled_at(), new_scheduled_at);
        assert_eq!(booking.status(), BookingStatus::Scheduled);
        assert_eq!(booking.reason().as_str(), "Диагностика подвески");
        assert_eq!(booking.notes().unwrap().as_str(), "Клиент просил утром");
        assert_eq!(*booking.created_at(), created_at);
        assert_eq!(*booking.updated_at(), updated_at);
    }

    /// Если новое время обновления раньше создания, перенос должен завершиться
    /// ошибкой без частичного изменения `scheduled_at`.
    #[test]
    fn reschedule_rejects_update_time_before_created_at_without_changing_booking() {
        let created_at = fixed_time(1_700_000_000);
        let original_scheduled_at = fixed_time(1_700_010_000);
        let new_scheduled_at = fixed_time(1_700_020_000);
        let mut booking = scheduled_booking(created_at);

        let error = booking
            .reschedule(new_scheduled_at, fixed_time(1_699_999_999))
            .unwrap_err();

        assert_eq!(error, BookingError::UpdatedAtBeforeCreatedAt);
        assert_eq!(*booking.scheduled_at(), original_scheduled_at);
        assert_eq!(*booking.updated_at(), created_at);
    }

    /// Причину можно менять только у активной записи.
    #[test]
    fn update_reason_changes_reason_and_updates_timestamp() {
        let created_at = fixed_time(1_700_000_000);
        let updated_at = fixed_time(1_700_000_100);
        let mut booking = scheduled_booking(created_at);

        booking
            .update_reason(reason("Компьютерная диагностика"), updated_at)
            .unwrap();

        assert_eq!(booking.reason().as_str(), "Компьютерная диагностика");
        assert_eq!(*booking.updated_at(), updated_at);
    }

    /// Обновление заметки принимает `Some`, когда нужно добавить или заменить
    /// внутренний комментарий.
    #[test]
    fn update_notes_sets_notes_and_updates_timestamp() {
        let created_at = fixed_time(1_700_000_000);
        let updated_at = fixed_time(1_700_000_100);
        let mut booking = Booking::new(
            booking_id(),
            client_id(),
            car_id(),
            fixed_time(1_700_010_000),
            reason("Диагностика"),
            None,
            created_at,
        );

        booking
            .update_notes(Some(notes("Ждет в клиентской зоне")), updated_at)
            .unwrap();

        assert_eq!(booking.notes().unwrap().as_str(), "Ждет в клиентской зоне");
        assert_eq!(*booking.updated_at(), updated_at);
    }

    /// `update_notes(None)` удаляет заметку тем же доменным способом, что и
    /// явный сценарий `clear_notes`.
    #[test]
    fn update_notes_can_remove_notes() {
        let created_at = fixed_time(1_700_000_000);
        let updated_at = fixed_time(1_700_000_100);
        let mut booking = scheduled_booking(created_at);

        booking.update_notes(None, updated_at).unwrap();

        assert!(booking.notes().is_none());
        assert_eq!(*booking.updated_at(), updated_at);
    }

    /// Очистка заметки оставляет запись без комментария и двигает `updated_at`.
    #[test]
    fn clear_notes_removes_notes_and_updates_timestamp() {
        let created_at = fixed_time(1_700_000_000);
        let updated_at = fixed_time(1_700_000_100);
        let mut booking = scheduled_booking(created_at);

        booking.clear_notes(updated_at).unwrap();

        assert!(booking.notes().is_none());
        assert_eq!(*booking.updated_at(), updated_at);
    }

    /// Заметку можно обновить даже после финального статуса: это операционный
    /// комментарий, а не изменение факта записи или ее исхода.
    #[test]
    fn update_notes_is_allowed_for_terminal_booking() {
        let created_at = fixed_time(1_700_000_000);
        let completed_at = fixed_time(1_700_000_100);
        let notes_updated_at = fixed_time(1_700_000_200);
        let mut booking = scheduled_booking(created_at);
        booking.complete(completed_at).unwrap();

        booking
            .update_notes(Some(notes("Работы закрыты по акту")), notes_updated_at)
            .unwrap();

        assert_eq!(booking.status(), BookingStatus::Completed);
        assert_eq!(booking.notes().unwrap().as_str(), "Работы закрыты по акту");
        assert_eq!(*booking.updated_at(), notes_updated_at);
    }

    /// Успешное завершение переводит запись в финальный статус и обновляет дату
    /// последнего изменения.
    #[test]
    fn complete_transitions_scheduled_booking_to_completed() {
        let created_at = fixed_time(1_700_000_000);
        let completed_at = fixed_time(1_700_000_100);
        let mut booking = scheduled_booking(created_at);

        booking.complete(completed_at).unwrap();

        assert_eq!(booking.status(), BookingStatus::Completed);
        assert!(!booking.is_scheduled());
        assert!(booking.is_terminal());
        assert_eq!(*booking.updated_at(), completed_at);
    }

    /// Отмена является финальным исходом записи.
    #[test]
    fn cancel_transitions_scheduled_booking_to_cancelled() {
        let created_at = fixed_time(1_700_000_000);
        let cancelled_at = fixed_time(1_700_000_100);
        let mut booking = scheduled_booking(created_at);

        booking.cancel(cancelled_at).unwrap();

        assert_eq!(booking.status(), BookingStatus::Cancelled);
        assert!(booking.is_terminal());
        assert_eq!(*booking.updated_at(), cancelled_at);
    }

    /// Неявка клиента закрывает активную запись отдельным финальным статусом.
    #[test]
    fn mark_no_show_transitions_scheduled_booking_to_no_show() {
        let created_at = fixed_time(1_700_000_000);
        let no_show_at = fixed_time(1_700_000_100);
        let mut booking = scheduled_booking(created_at);

        booking.mark_no_show(no_show_at).unwrap();

        assert_eq!(booking.status(), BookingStatus::NoShow);
        assert!(booking.is_terminal());
        assert_eq!(*booking.updated_at(), no_show_at);
    }

    /// Финальный переход с некорректным временем должен завершиться ошибкой без
    /// изменения статуса.
    #[test]
    fn transition_rejects_update_time_before_created_at_without_changing_status() {
        let created_at = fixed_time(1_700_000_000);
        let mut booking = scheduled_booking(created_at);

        let error = booking.complete(fixed_time(1_699_999_999)).unwrap_err();

        assert_eq!(error, BookingError::UpdatedAtBeforeCreatedAt);
        assert_eq!(booking.status(), BookingStatus::Scheduled);
        assert_eq!(*booking.updated_at(), created_at);
    }

    /// После финального статуса нельзя переносить запись: это уже закрытая
    /// история, а не будущий визит.
    #[test]
    fn reschedule_rejects_terminal_booking_without_changing_timestamp() {
        let created_at = fixed_time(1_700_000_000);
        let completed_at = fixed_time(1_700_000_100);
        let mut booking = scheduled_booking(created_at);
        booking.complete(completed_at).unwrap();

        let error = booking
            .reschedule(fixed_time(1_700_020_000), fixed_time(1_700_000_200))
            .unwrap_err();

        assert_eq!(
            error,
            BookingError::CannotModifyFinalBooking {
                status: BookingStatus::Completed,
            }
        );
        assert_eq!(*booking.updated_at(), completed_at);
    }

    /// После финального статуса нельзя менять причину обращения.
    #[test]
    fn update_reason_rejects_terminal_booking_without_changing_reason() {
        let created_at = fixed_time(1_700_000_000);
        let cancelled_at = fixed_time(1_700_000_100);
        let mut booking = scheduled_booking(created_at);
        booking.cancel(cancelled_at).unwrap();

        let error = booking
            .update_reason(reason("Новая причина"), fixed_time(1_700_000_200))
            .unwrap_err();

        assert_eq!(
            error,
            BookingError::CannotModifyFinalBooking {
                status: BookingStatus::Cancelled,
            }
        );
        assert_eq!(booking.reason().as_str(), "Диагностика подвески");
        assert_eq!(*booking.updated_at(), cancelled_at);
    }

    /// Финальные статусы нельзя переводить друг в друга без отдельного
    /// бизнес-сценария корректировки.
    #[test]
    fn transition_rejects_terminal_to_another_terminal_status() {
        let created_at = fixed_time(1_700_000_000);
        let no_show_at = fixed_time(1_700_000_100);
        let mut booking = scheduled_booking(created_at);
        booking.mark_no_show(no_show_at).unwrap();

        let error = booking.complete(fixed_time(1_700_000_200)).unwrap_err();

        assert_eq!(
            error,
            BookingError::CannotTransitionStatus {
                from: BookingStatus::NoShow,
                to: BookingStatus::Completed,
            }
        );
        assert_eq!(booking.status(), BookingStatus::NoShow);
        assert_eq!(*booking.updated_at(), no_show_at);
    }

    /// Display-формат статусов используется в ошибках и логах, поэтому тест
    /// фиксирует стабильные строковые значения.
    #[test]
    fn booking_status_display_uses_stable_snake_case_values() {
        assert_eq!(BookingStatus::Scheduled.to_string(), "scheduled");
        assert_eq!(BookingStatus::Completed.to_string(), "completed");
        assert_eq!(BookingStatus::Cancelled.to_string(), "cancelled");
        assert_eq!(BookingStatus::NoShow.to_string(), "no_show");
    }
}
