//! Клиент автосервиса и связанные с ним value objects.
//!
//! В этом модуле клиент моделируется не как набор публичных строк, а как
//! доменная сущность с проверенными полями. Такой подход дает два важных
//! свойства:
//! - имя и заметки проходят нормализацию до попадания в `Client`;
//! - дата обновления не может быть раньше даты создания при восстановлении из
//!   хранилища.
//!
//! Алгоритмически модуль разделен на два уровня:
//! 1. `ClientName` и `ClientNotes` отвечают за чистоту пользовательского ввода:
//!    обрезают пробелы, проверяют пустые значения и ограничивают длину.
//! 2. `Client` отвечает за жизненный цикл сущности: создание, восстановление и
//!    изменения, которые двигают `updated_at`.
//!
//! Это удерживает инварианты рядом с данными и не заставляет обработчики команд,
//! сервисы и репозитории повторять одни и те же проверки.

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::{ClientId, PhoneNumber};

/// Максимальная длина имени в Unicode-символах, а не в байтах.
///
/// Для кириллицы это принципиально: `String::len()` считает байты и дал бы
/// неверное ограничение для русских имен. Поэтому ниже используется
/// `chars().count()`.
const MAX_CLIENT_NAME_LEN: usize = 100;
/// Максимальная длина заметки в Unicode-символах.
const MAX_CLIENT_NOTES_LEN: usize = 1000;

/// Имя клиента.
///
/// Это не обязательно паспортное ФИО. В реальном автосервисе клиент может быть
/// записан как `Иван Петрович`, `Сергей BMW` или `Дима Passat`.
///
/// Внутренняя строка закрыта, чтобы имя нельзя было создать в обход `parse`.
/// После успешного парсинга домен может полагаться на два инварианта:
/// имя не пустое и не длиннее `MAX_CLIENT_NAME_LEN`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientName(String);

impl ClientName {
    /// Нормализует пользовательский ввод и создает валидное имя клиента.
    ///
    /// Алгоритм:
    /// 1. Убираем пробелы по краям. Это типичный шум из Telegram-сообщений,
    ///    web-форм и copy-paste.
    /// 2. Проверяем пустоту после trim. Строка из одних пробелов не должна
    ///    становиться именем.
    /// 3. Считаем длину через `chars().count()`, чтобы корректно работать с
    ///    кириллицей и другими Unicode-символами.
    /// 4. Если лимит превышен, возвращаем ошибку с `max` и `actual`, чтобы
    ///    прикладной слой мог показать точное сообщение.
    /// 5. Сохраняем уже обрезанную строку как единственный внутренний формат.
    pub fn parse(input: &str) -> Result<Self, ClientError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Err(ClientError::EmptyName);
        }

        let actual = trimmed.chars().count();

        if actual > MAX_CLIENT_NAME_LEN {
            return Err(ClientError::NameTooLong {
                max: MAX_CLIENT_NAME_LEN,
                actual,
            });
        }

        Ok(Self(trimmed.to_string()))
    }

    /// Возвращает имя без передачи владения внутренней строкой.
    ///
    /// Это безопасный способ отдать значение в UI, БД или лог, не позволяя
    /// вызывающему коду изменить строку и нарушить инварианты `ClientName`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Показывает имя ровно в том каноническом виде, который был сохранен после
/// `ClientName::parse`.
impl std::fmt::Display for ClientName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Заметки по клиенту.
///
/// Пустая заметка не должна превращаться в доменное значение. Если пользователь
/// ничего не указал, лучше хранить `None`.
///
/// Это важное отличие от `ClientName`: имя обязательно, а заметка опциональна.
/// Поэтому `parse` возвращает `Result<Option<ClientNotes>, ClientError>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientNotes(String);

impl ClientNotes {
    /// Нормализует пользовательские заметки.
    ///
    /// Алгоритм:
    /// 1. Убираем пробелы по краям.
    /// 2. Если после trim ничего не осталось, возвращаем `Ok(None)`. Это не
    ///    ошибка: пользователь просто не оставил заметку.
    /// 3. Считаем длину в Unicode-символах, чтобы русские заметки не штрафовались
    ///    из-за многобайтового UTF-8.
    /// 4. При превышении лимита возвращаем `NotesTooLong`.
    /// 5. Иначе сохраняем trimmed-текст как `Some(ClientNotes)`.
    pub fn parse(input: &str) -> Result<Option<Self>, ClientError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(None);
        }

        let actual = trimmed.chars().count();

        if actual > MAX_CLIENT_NOTES_LEN {
            return Err(ClientError::NotesTooLong {
                max: MAX_CLIENT_NOTES_LEN,
                actual,
            });
        }

        Ok(Some(Self(trimmed.to_string())))
    }

    /// Возвращает текст заметки без копирования и без передачи владения.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Показывает заметку без дополнительного форматирования.
impl std::fmt::Display for ClientNotes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Клиент автосервиса.
///
/// `Client` - доменная сущность: у нее есть стабильный `id`, а остальные поля
/// могут меняться со временем. Все поля приватные, чтобы изменения проходили
/// через методы сущности и всегда обновляли `updated_at`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Client {
    /// Стабильный идентификатор клиента.
    id: ClientId,
    /// Проверенное имя клиента.
    name: ClientName,
    /// Проверенный телефонный номер.
    phone: PhoneNumber,
    /// Опциональная заметка. Пустая строка здесь не хранится: она представлена
    /// как `None`.
    notes: Option<ClientNotes>,
    /// Момент создания сущности.
    created_at: DateTime<Utc>,
    /// Момент последнего изменения данных клиента.
    updated_at: DateTime<Utc>,
}

impl Client {
    /// Создает нового клиента.
    ///
    /// Для новой сущности `created_at` и `updated_at` совпадают.
    ///
    /// Алгоритм создания:
    /// 1. Прикладной слой заранее создает проверенные value objects:
    ///    `ClientName`, `PhoneNumber`, `ClientNotes`.
    /// 2. В доменную сущность передается один момент времени `now`.
    /// 3. Этот момент записывается и как дата создания, и как дата обновления,
    ///    потому что новая сущность еще не изменялась после создания.
    ///
    /// Метод не возвращает `Result`, потому что все входные значения уже имеют
    /// корректные типы, а временной инвариант здесь невозможно нарушить.
    pub fn new(
        id: ClientId,
        name: ClientName,
        phone: PhoneNumber,
        notes: Option<ClientNotes>,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            phone,
            notes,
            created_at: now,
            updated_at: now,
        }
    }

    /// Восстанавливает клиента из уже существующего состояния.
    ///
    /// Этот метод нужен инфраструктурному слою при загрузке из базы данных.
    ///
    /// В отличие от `new`, здесь даты приходят извне, поэтому их нужно
    /// проверить. Единственный временной инвариант: `updated_at` не может быть
    /// раньше `created_at`.
    ///
    /// Метод не парсит строки из БД. Репозиторий должен сначала восстановить
    /// value objects через их собственные конструкторы, а затем собрать `Client`.
    pub fn restore(
        id: ClientId,
        name: ClientName,
        phone: PhoneNumber,
        notes: Option<ClientNotes>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, ClientError> {
        if updated_at < created_at {
            return Err(ClientError::UpdatedAtBeforeCreatedAt);
        }

        Ok(Self {
            id,
            name,
            phone,
            notes,
            created_at,
            updated_at,
        })
    }

    /// Возвращает идентификатор клиента.
    ///
    /// `ClientId` копируемый, поэтому можно вернуть его по значению без риска
    /// отдать наружу изменяемую ссылку на состояние сущности.
    pub fn id(&self) -> ClientId {
        self.id
    }

    /// Возвращает проверенное имя клиента.
    pub fn name(&self) -> &ClientName {
        &self.name
    }

    /// Возвращает проверенный номер телефона клиента.
    pub fn phone(&self) -> &PhoneNumber {
        &self.phone
    }

    /// Возвращает заметку, если она есть.
    ///
    /// Наружу отдается `Option<&ClientNotes>`, чтобы не копировать строку и не
    /// позволять вызывающему коду менять внутреннее состояние напрямую.
    pub fn notes(&self) -> Option<&ClientNotes> {
        self.notes.as_ref()
    }

    /// Возвращает дату создания клиента.
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    /// Возвращает дату последнего изменения клиента.
    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    /// Меняет имя клиента и фиксирует момент изменения.
    ///
    /// Метод принимает уже проверенный `ClientName`, поэтому не занимается
    /// парсингом. Его ответственность - заменить поле и синхронно обновить
    /// `updated_at`, чтобы история состояния оставалась честной.
    pub fn rename(&mut self, name: ClientName, now: DateTime<Utc>) {
        self.name = name;
        self.updated_at = now;
    }

    /// Меняет телефон клиента и фиксирует момент изменения.
    ///
    /// Валидация номера находится в `PhoneNumber::parse`; сущность работает
    /// только с валидным value object.
    pub fn change_phone(&mut self, phone: PhoneNumber, now: DateTime<Utc>) {
        self.phone = phone;
        self.updated_at = now;
    }

    /// Заменяет заметку клиента и фиксирует момент изменения.
    ///
    /// `None` означает отсутствие заметки, а не пустую строку. Это сохраняет
    /// один понятный способ представить пустое значение.
    pub fn update_notes(&mut self, notes: Option<ClientNotes>, now: DateTime<Utc>) {
        self.notes = notes;
        self.updated_at = now;
    }

    /// Удаляет заметку клиента и фиксирует момент изменения.
    ///
    /// Это явный сценарный метод поверх `update_notes(None, now)`. Он делает код
    /// вызывающей стороны читаемее, когда пользователь именно очищает заметку.
    pub fn clear_notes(&mut self, now: DateTime<Utc>) {
        self.notes = None;
        self.updated_at = now;
    }
}

/// Ошибка работы с клиентом.
///
/// Ошибки находятся рядом с доменными типами, потому что именно они определяют
/// правила валидности имени, заметок и временных меток клиента.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ClientError {
    /// Имя пустое после удаления пробелов по краям.
    #[error("client name is empty")]
    EmptyName,

    /// Имя превышает допустимую длину в Unicode-символах.
    #[error("client name is too long: max={max}, actual={actual}")]
    NameTooLong { max: usize, actual: usize },

    /// Заметка превышает допустимую длину в Unicode-символах.
    #[error("client notes are too long: max={max}, actual={actual}")]
    NotesTooLong { max: usize, actual: usize },

    /// Восстановленное состояние клиента нарушает временной порядок.
    #[error("client updated_at cannot be earlier than created_at")]
    UpdatedAtBeforeCreatedAt,
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use super::{
        Client, ClientError, ClientName, ClientNotes, MAX_CLIENT_NAME_LEN, MAX_CLIENT_NOTES_LEN,
    };
    use crate::{ClientId, PhoneNumber};

    fn fixed_time(seconds: i64) -> chrono::DateTime<Utc> {
        Utc.timestamp_opt(seconds, 0).single().unwrap()
    }

    fn client_id() -> ClientId {
        ClientId::from_uuid(Uuid::from_u128(1))
    }

    fn client_name(value: &str) -> ClientName {
        ClientName::parse(value).unwrap()
    }

    fn client_phone(value: &str) -> PhoneNumber {
        PhoneNumber::parse(value).unwrap()
    }

    fn client_notes(value: &str) -> ClientNotes {
        ClientNotes::parse(value).unwrap().unwrap()
    }

    /// Имя очищается от внешних пробелов и сохраняется в каноническом виде.
    /// Это проверяет основной алгоритм `ClientName::parse`.
    #[test]
    fn client_name_parse_trims_valid_name() {
        let name = ClientName::parse("  Иван Петрович  ").unwrap();

        assert_eq!(name.as_str(), "Иван Петрович");
        assert_eq!(name.to_string(), "Иван Петрович");
    }

    /// Строка из пробелов не должна становиться именем клиента.
    #[test]
    fn client_name_parse_rejects_empty_name_after_trim() {
        let error = ClientName::parse("   ").unwrap_err();

        assert_eq!(error, ClientError::EmptyName);
    }

    /// Лимит имени считается в символах, а не в байтах. Тест с кириллицей
    /// защищает от случайной замены `chars().count()` на `String::len()`.
    #[test]
    fn client_name_parse_allows_unicode_name_at_max_length() {
        let input = "Я".repeat(MAX_CLIENT_NAME_LEN);

        let name = ClientName::parse(&input).unwrap();

        assert_eq!(name.as_str().chars().count(), MAX_CLIENT_NAME_LEN);
    }

    /// При превышении лимита ошибка должна вернуть и максимум, и фактическую
    /// длину, чтобы UI мог объяснить проблему пользователю.
    #[test]
    fn client_name_parse_rejects_too_long_name() {
        let input = "a".repeat(MAX_CLIENT_NAME_LEN + 1);

        let error = ClientName::parse(&input).unwrap_err();

        assert_eq!(
            error,
            ClientError::NameTooLong {
                max: MAX_CLIENT_NAME_LEN,
                actual: MAX_CLIENT_NAME_LEN + 1,
            }
        );
    }

    /// Непустая заметка очищается от пробелов и возвращается как `Some`.
    #[test]
    fn client_notes_parse_trims_non_empty_notes() {
        let notes = ClientNotes::parse("  Позвонить после 18:00  ")
            .unwrap()
            .unwrap();

        assert_eq!(notes.as_str(), "Позвонить после 18:00");
        assert_eq!(notes.to_string(), "Позвонить после 18:00");
    }

    /// Пустая заметка не является ошибкой: в домене она представляется как
    /// отсутствие значения.
    #[test]
    fn client_notes_parse_returns_none_for_empty_input() {
        let notes = ClientNotes::parse("   ").unwrap();

        assert!(notes.is_none());
    }

    /// Заметки тоже считаются в Unicode-символах. Это важно для русскоязычных
    /// комментариев мастера или администратора.
    #[test]
    fn client_notes_parse_allows_unicode_notes_at_max_length() {
        let input = "ю".repeat(MAX_CLIENT_NOTES_LEN);

        let notes = ClientNotes::parse(&input).unwrap().unwrap();

        assert_eq!(notes.as_str().chars().count(), MAX_CLIENT_NOTES_LEN);
    }

    /// Слишком длинная заметка возвращает структурированную ошибку с лимитами.
    #[test]
    fn client_notes_parse_rejects_too_long_notes() {
        let input = "a".repeat(MAX_CLIENT_NOTES_LEN + 1);

        let error = ClientNotes::parse(&input).unwrap_err();

        assert_eq!(
            error,
            ClientError::NotesTooLong {
                max: MAX_CLIENT_NOTES_LEN,
                actual: MAX_CLIENT_NOTES_LEN + 1,
            }
        );
    }

    /// Новый клиент получает одинаковые `created_at` и `updated_at`, потому что
    /// изменений после создания еще не было.
    #[test]
    fn client_new_sets_initial_state_and_timestamps() {
        let now = fixed_time(1_700_000_000);
        let name = client_name("Иван");
        let phone = client_phone("+375291234567");
        let notes = Some(client_notes("Постоянный клиент"));

        let client = Client::new(client_id(), name, phone, notes, now);

        assert_eq!(client.id(), client_id());
        assert_eq!(client.name().as_str(), "Иван");
        assert_eq!(client.phone().as_str(), "+375291234567");
        assert_eq!(client.notes().unwrap().as_str(), "Постоянный клиент");
        assert_eq!(*client.created_at(), now);
        assert_eq!(*client.updated_at(), now);
    }

    /// Restore собирает клиента из уже существующего состояния и сохраняет
    /// разные даты создания/обновления, если порядок дат корректен.
    #[test]
    fn client_restore_accepts_valid_persisted_state() {
        let created_at = fixed_time(1_700_000_000);
        let updated_at = fixed_time(1_700_000_500);

        let client = Client::restore(
            client_id(),
            client_name("Иван"),
            client_phone("+375291234567"),
            None,
            created_at,
            updated_at,
        )
        .unwrap();

        assert_eq!(*client.created_at(), created_at);
        assert_eq!(*client.updated_at(), updated_at);
        assert!(client.notes().is_none());
    }

    /// Если данные из БД нарушают временной порядок, доменная модель не должна
    /// принимать такое состояние как валидное.
    #[test]
    fn client_restore_rejects_updated_at_before_created_at() {
        let created_at = fixed_time(1_700_000_500);
        let updated_at = fixed_time(1_700_000_000);

        let error = Client::restore(
            client_id(),
            client_name("Иван"),
            client_phone("+375291234567"),
            None,
            created_at,
            updated_at,
        )
        .unwrap_err();

        assert_eq!(error, ClientError::UpdatedAtBeforeCreatedAt);
    }

    /// Переименование меняет только имя и `updated_at`, не трогая идентификатор,
    /// телефон, заметки и дату создания.
    #[test]
    fn rename_changes_name_and_updates_timestamp() {
        let created_at = fixed_time(1_700_000_000);
        let updated_at = fixed_time(1_700_000_100);
        let mut client = Client::new(
            client_id(),
            client_name("Иван"),
            client_phone("+375291234567"),
            Some(client_notes("Заметка")),
            created_at,
        );

        client.rename(client_name("Петр"), updated_at);

        assert_eq!(client.name().as_str(), "Петр");
        assert_eq!(client.phone().as_str(), "+375291234567");
        assert_eq!(client.notes().unwrap().as_str(), "Заметка");
        assert_eq!(*client.created_at(), created_at);
        assert_eq!(*client.updated_at(), updated_at);
    }

    /// Смена телефона работает с уже валидным `PhoneNumber` и фиксирует время
    /// изменения сущности.
    #[test]
    fn change_phone_changes_phone_and_updates_timestamp() {
        let created_at = fixed_time(1_700_000_000);
        let updated_at = fixed_time(1_700_000_100);
        let mut client = Client::new(
            client_id(),
            client_name("Иван"),
            client_phone("+375291234567"),
            None,
            created_at,
        );

        client.change_phone(client_phone("8 033 765 43 21"), updated_at);

        assert_eq!(client.phone().as_str(), "+375337654321");
        assert_eq!(*client.created_at(), created_at);
        assert_eq!(*client.updated_at(), updated_at);
    }

    /// Обновление заметки принимает `Some`, когда пользователь добавляет или
    /// заменяет текст заметки.
    #[test]
    fn update_notes_sets_notes_and_updates_timestamp() {
        let created_at = fixed_time(1_700_000_000);
        let updated_at = fixed_time(1_700_000_100);
        let mut client = Client::new(
            client_id(),
            client_name("Иван"),
            client_phone("+375291234567"),
            None,
            created_at,
        );

        client.update_notes(Some(client_notes("Новая заметка")), updated_at);

        assert_eq!(client.notes().unwrap().as_str(), "Новая заметка");
        assert_eq!(*client.updated_at(), updated_at);
    }

    /// `update_notes(None)` и `clear_notes` должны оставлять один и тот же
    /// доменный смысл: заметки у клиента нет.
    #[test]
    fn update_notes_can_remove_notes() {
        let created_at = fixed_time(1_700_000_000);
        let updated_at = fixed_time(1_700_000_100);
        let mut client = Client::new(
            client_id(),
            client_name("Иван"),
            client_phone("+375291234567"),
            Some(client_notes("Старая заметка")),
            created_at,
        );

        client.update_notes(None, updated_at);

        assert!(client.notes().is_none());
        assert_eq!(*client.updated_at(), updated_at);
    }

    /// Явная очистка заметки делает сценарий читаемым и также обновляет
    /// timestamp последнего изменения.
    #[test]
    fn clear_notes_removes_notes_and_updates_timestamp() {
        let created_at = fixed_time(1_700_000_000);
        let updated_at = fixed_time(1_700_000_100);
        let mut client = Client::new(
            client_id(),
            client_name("Иван"),
            client_phone("+375291234567"),
            Some(client_notes("Старая заметка")),
            created_at,
        );

        client.clear_notes(updated_at);

        assert!(client.notes().is_none());
        assert_eq!(*client.created_at(), created_at);
        assert_eq!(*client.updated_at(), updated_at);
    }
}
