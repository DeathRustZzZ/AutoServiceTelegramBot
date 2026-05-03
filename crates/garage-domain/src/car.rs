//! Автомобиль клиента и связанные с ним value objects.
//!
//! Модуль держит автомобиль не как набор свободных строк, а как доменную
//! сущность с проверенными полями. Это особенно важно для Telegram-бота: ввод
//! пользователя часто содержит пробелы, дефисы, разный регистр и пустые
//! необязательные поля.
//!
//! Алгоритмически здесь есть два уровня:
//! 1. Value objects (`CarMake`, `CarModel`, `CarYear`, `LicensePlate`, `Vin`,
//!    `CarNotes`) один раз нормализуют и валидируют пользовательский ввод.
//! 2. `Car` управляет жизненным циклом сущности: создание, восстановление из
//!    хранилища и изменения, которые обязаны двигать `updated_at`.
//!
//! Такой дизайн оставляет инварианты рядом с данными. Прикладной слой, команды
//! бота и репозитории работают уже с валидными типами и не дублируют проверки.

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::{CarId, ClientId};

/// Максимальная длина марки в Unicode-символах.
const MAX_CAR_MAKE_LEN: usize = 50;
/// Максимальная длина модели в Unicode-символах.
const MAX_CAR_MODEL_LEN: usize = 80;
/// Нижняя граница года выпуска, которую домен считает осмысленной.
const MIN_CAR_YEAR: u16 = 1900;
/// Верхняя граница года выпуска.
///
/// Значение намеренно не привязано к текущей дате: домен допускает предзаказы,
/// ошибки миграции или будущие модели, но оставляет разумный технический лимит.
const MAX_CAR_YEAR: u16 = 2100;
/// Максимальная длина нормализованного номера.
const MAX_LICENSE_PLATE_LEN: usize = 20;
/// VIN по стандарту ISO 3779 всегда состоит из 17 символов.
const VIN_LEN: usize = 17;
/// Максимальная длина заметки в Unicode-символах.
const MAX_CAR_NOTES_LEN: usize = 1000;

/// Автомобиль, принадлежащий клиенту автосервиса.
///
/// `Car` - доменная сущность: у нее есть стабильный `id`, связь с владельцем
/// через `client_id` и изменяемые характеристики автомобиля. Поля закрыты,
/// чтобы состояние менялось только через методы, которые сохраняют временной
/// инвариант `created_at <= updated_at`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Car {
    /// Стабильный идентификатор автомобиля.
    id: CarId,
    /// Идентификатор клиента-владельца.
    client_id: ClientId,
    /// Проверенная марка автомобиля.
    make: CarMake,
    /// Проверенная модель автомобиля.
    model: CarModel,
    /// Опциональный год выпуска. Отсутствие года лучше хранить как `None`, а не
    /// как фиктивный `0`.
    year: Option<CarYear>,
    /// Опциональный регистрационный номер в нормализованном виде.
    plate_number: Option<LicensePlate>,
    /// Опциональный VIN в нормализованном виде.
    vin: Option<Vin>,
    /// Опциональная заметка по автомобилю.
    notes: Option<CarNotes>,
    /// Момент создания сущности.
    created_at: DateTime<Utc>,
    /// Момент последнего изменения сущности.
    updated_at: DateTime<Utc>,
}

impl Car {
    /// Создает новый автомобиль.
    ///
    /// Алгоритм:
    /// 1. Вызывающий код заранее превращает пользовательский ввод в value
    ///    objects. Поэтому `new` не парсит строки и не может получить
    ///    невалидную марку, модель, номер или VIN.
    /// 2. Один момент времени `now` записывается и в `created_at`, и в
    ///    `updated_at`: новая сущность еще не менялась после создания.
    /// 3. Опциональные поля сохраняются как `Option`, чтобы не смешивать
    ///    отсутствие значения с пустой строкой.
    ///
    /// Метод не возвращает `Result`, потому что при таких типах входных данных
    /// временной инвариант нарушить невозможно.
    pub fn new(
        id: CarId,
        client_id: ClientId,
        make: CarMake,
        model: CarModel,
        year: Option<CarYear>,
        plate_number: Option<LicensePlate>,
        vin: Option<Vin>,
        notes: Option<CarNotes>,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            client_id,
            make,
            model,
            year,
            plate_number,
            vin,
            notes,
            created_at: now,
            updated_at: now,
        }
    }

    /// Восстанавливает автомобиль из уже существующего состояния.
    ///
    /// Этот метод нужен репозиторию при чтении из базы данных. В отличие от
    /// `new`, даты приходят из внешнего слоя, поэтому домен обязан проверить
    /// порядок времени.
    ///
    /// Алгоритм восстановления:
    /// 1. Репозиторий восстанавливает value objects через их конструкторы.
    /// 2. `restore` сравнивает `updated_at` и `created_at`.
    /// 3. Если обновление оказалось раньше создания, состояние считается
    ///    поврежденным и возвращается `UpdatedAtBeforeCreatedAt`.
    /// 4. Иначе сущность собирается без изменения переданных дат.
    pub fn restore(
        id: CarId,
        client_id: ClientId,
        make: CarMake,
        model: CarModel,
        year: Option<CarYear>,
        plate_number: Option<LicensePlate>,
        vin: Option<Vin>,
        notes: Option<CarNotes>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, CarError> {
        if updated_at < created_at {
            return Err(CarError::UpdatedAtBeforeCreatedAt);
        }

        Ok(Self {
            id,
            client_id,
            make,
            model,
            year,
            plate_number,
            vin,
            notes,
            created_at,
            updated_at,
        })
    }

    /// Возвращает идентификатор автомобиля.
    pub fn id(&self) -> CarId {
        self.id
    }

    /// Возвращает идентификатор клиента-владельца.
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    /// Возвращает проверенную марку без копирования строки.
    pub fn make(&self) -> &CarMake {
        &self.make
    }

    /// Возвращает проверенную модель без копирования строки.
    pub fn model(&self) -> &CarModel {
        &self.model
    }

    /// Возвращает год выпуска, если он известен.
    ///
    /// `CarYear` копируемый, поэтому его можно отдавать по значению.
    pub fn year(&self) -> Option<CarYear> {
        self.year
    }

    /// Возвращает регистрационный номер, если он был указан.
    pub fn plate_number(&self) -> Option<&LicensePlate> {
        self.plate_number.as_ref()
    }

    /// Возвращает VIN, если он был указан.
    pub fn vin(&self) -> Option<&Vin> {
        self.vin.as_ref()
    }

    /// Возвращает заметку по автомобилю, если она есть.
    pub fn notes(&self) -> Option<&CarNotes> {
        self.notes.as_ref()
    }

    /// Возвращает дату создания автомобиля.
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    /// Возвращает дату последнего изменения автомобиля.
    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    /// Обновляет `updated_at`, сохраняя временной инвариант сущности.
    ///
    /// Алгоритм:
    /// 1. Сравниваем новое время с `created_at`.
    /// 2. Если новое время раньше создания, возвращаем ошибку и не меняем
    ///    состояние автомобиля.
    /// 3. Если время корректно, записываем его в `updated_at`.
    ///
    /// Метод приватный: это техническая операция жизненного цикла. Публичные
    /// методы вызывают ее до изменения конкретного поля, поэтому при ошибке
    /// автомобиль не остается в частично обновленном состоянии.
    fn touch(&mut self, now: DateTime<Utc>) -> Result<(), CarError> {
        if now < self.created_at {
            return Err(CarError::UpdatedAtBeforeCreatedAt);
        }

        self.updated_at = now;
        Ok(())
    }

    /// Меняет марку, модель и год выпуска как единый блок идентичности авто.
    ///
    /// Марка и модель обычно редактируются вместе: например, если машину
    /// ошибочно завели как `Audi A4`, а нужно `Audi A6`. Поэтому метод меняет
    /// этот набор атомарно после успешного `touch`.
    pub fn update_identity(
        &mut self,
        make: CarMake,
        model: CarModel,
        year: Option<CarYear>,
        now: DateTime<Utc>,
    ) -> Result<(), CarError> {
        self.touch(now)?;
        self.make = make;
        self.model = model;
        self.year = year;
        Ok(())
    }

    /// Заменяет регистрационный номер и фиксирует момент изменения.
    ///
    /// `None` означает, что номер неизвестен или был очищен пользователем.
    pub fn update_plate_number(
        &mut self,
        plate_number: Option<LicensePlate>,
        now: DateTime<Utc>,
    ) -> Result<(), CarError> {
        self.touch(now)?;
        self.plate_number = plate_number;
        Ok(())
    }

    /// Заменяет VIN и фиксирует момент изменения.
    ///
    /// Сущность принимает уже проверенный `Vin`; синтаксические правила VIN
    /// остаются в `Vin::parse`.
    pub fn update_vin(&mut self, vin: Option<Vin>, now: DateTime<Utc>) -> Result<(), CarError> {
        self.touch(now)?;
        self.vin = vin;
        Ok(())
    }

    /// Заменяет заметку по автомобилю и фиксирует момент изменения.
    ///
    /// Пустая заметка представлена как `None`, а не как `Some("")`.
    pub fn update_notes(
        &mut self,
        notes: Option<CarNotes>,
        now: DateTime<Utc>,
    ) -> Result<(), CarError> {
        self.touch(now)?;
        self.notes = notes;
        Ok(())
    }

    /// Удаляет заметку и фиксирует момент изменения.
    ///
    /// Метод дублирует сценарий `update_notes(None, now)` по смыслу, но делает
    /// вызывающий код явнее: пользователь именно очистил заметку, а не передал
    /// новую опциональную структуру.
    pub fn clear_notes(&mut self, now: DateTime<Utc>) -> Result<(), CarError> {
        self.touch(now)?;
        self.notes = None;
        Ok(())
    }
}

/// Марка автомобиля.
///
/// Внутренняя строка закрыта, чтобы марку нельзя было создать в обход `parse`.
/// После успешного парсинга домен знает, что значение не пустое и не длиннее
/// `MAX_CAR_MAKE_LEN`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CarMake(String);

impl CarMake {
    /// Нормализует и проверяет марку автомобиля.
    ///
    /// Алгоритм:
    /// 1. Убираем внешние пробелы, типичные для ручного ввода.
    /// 2. Проверяем пустоту после `trim`.
    /// 3. Считаем длину через `chars().count()`, чтобы кириллица и другие
    ///    Unicode-символы считались как символы, а не как UTF-8 байты.
    /// 4. При превышении лимита возвращаем структурированную ошибку с `max` и
    ///    `actual`.
    /// 5. Сохраняем trimmed-строку как канонический вид марки.
    pub fn parse(input: &str) -> Result<Self, CarError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Err(CarError::EmptyMake);
        }

        let actual = trimmed.chars().count();

        if actual > MAX_CAR_MAKE_LEN {
            return Err(CarError::MakeTooLong {
                max: MAX_CAR_MAKE_LEN,
                actual,
            });
        }

        Ok(Self(trimmed.to_string()))
    }

    /// Возвращает марку без копирования внутренней строки.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Показывает марку в каноническом виде, сохраненном после `parse`.
impl std::fmt::Display for CarMake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Модель автомобиля.
///
/// Инварианты такие же, как у `CarMake`, но лимит отдельный: названия моделей
/// часто длиннее марки и могут включать поколение или кузов.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CarModel(String);

impl CarModel {
    /// Нормализует и проверяет модель автомобиля.
    ///
    /// Алгоритм совпадает с `CarMake::parse`: trim, проверка пустоты, подсчет
    /// Unicode-символов, проверка лимита и сохранение канонической строки.
    pub fn parse(input: &str) -> Result<Self, CarError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Err(CarError::EmptyModel);
        }

        let actual = trimmed.chars().count();

        if actual > MAX_CAR_MODEL_LEN {
            return Err(CarError::ModelTooLong {
                max: MAX_CAR_MODEL_LEN,
                actual,
            });
        }

        Ok(Self(trimmed.to_string()))
    }

    /// Возвращает модель без копирования внутренней строки.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Показывает модель в каноническом виде.
impl std::fmt::Display for CarModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Год выпуска автомобиля.
///
/// Год хранится отдельным типом, чтобы в `Car` нельзя было случайно положить
/// `1800` или `9999`. Если год неизвестен, используется `Option<CarYear>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CarYear(u16);

impl CarYear {
    /// Создает год выпуска, если он находится в допустимом диапазоне.
    ///
    /// Алгоритм намеренно простой: включительный диапазон `MIN..=MAX` лучше
    /// читается и меньше подвержен ошибкам на границах, чем две отдельные
    /// проверки.
    pub fn new(value: u16) -> Result<Self, CarError> {
        if !(MIN_CAR_YEAR..=MAX_CAR_YEAR).contains(&value) {
            return Err(CarError::InvalidYear {
                min: MIN_CAR_YEAR,
                max: MAX_CAR_YEAR,
                actual: value,
            });
        }

        Ok(Self(value))
    }

    /// Возвращает числовой год.
    pub fn value(&self) -> u16 {
        self.0
    }
}

/// Показывает год как обычное число.
impl std::fmt::Display for CarYear {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Регистрационный номер автомобиля.
///
/// Номер опционален и нормализуется агрессивнее, чем марка или модель:
/// пользователь может ввести пробелы, дефисы и нижний регистр, но внутри домена
/// номер хранится без разделителей и в верхнем регистре.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LicensePlate(String);

impl LicensePlate {
    /// Нормализует регистрационный номер.
    ///
    /// Алгоритм:
    /// 1. Убираем внешние пробелы.
    /// 2. Одним проходом оставляем только буквенно-цифровые символы. Дефисы,
    ///    пробелы и другие разделители считаются визуальным шумом.
    /// 3. Каждый символ переводим в верхний регистр через `to_uppercase()`.
    ///    Это корректнее, чем ASCII-only uppercase, потому что номер может
    ///    содержать национальные буквы.
    /// 4. Если после очистки ничего не осталось, возвращаем `Ok(None)`: номер
    ///    не указан.
    /// 5. Проверяем длину уже нормализованного значения.
    /// 6. Сохраняем единственный внутренний формат.
    pub fn parse(input: &str) -> Result<Option<Self>, CarError> {
        let normalized: String = input
            .trim()
            .chars()
            .filter(|ch| ch.is_alphanumeric())
            .flat_map(|ch| ch.to_uppercase())
            .collect();

        if normalized.is_empty() {
            return Ok(None);
        }

        let actual = normalized.chars().count();

        if actual > MAX_LICENSE_PLATE_LEN {
            return Err(CarError::PlateTooLong {
                max: MAX_LICENSE_PLATE_LEN,
                actual,
            });
        }

        Ok(Some(Self(normalized)))
    }

    /// Возвращает нормализованный номер без копирования.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Показывает номер в сохраненном каноническом виде.
impl std::fmt::Display for LicensePlate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// VIN автомобиля.
///
/// VIN хранится отдельно от обычной строки, потому что у него есть строгие
/// синтаксические правила: длина 17, только ASCII-буквы и цифры, без букв
/// `I`, `O`, `Q`, которые легко спутать с `1` и `0`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Vin(String);

impl Vin {
    /// Нормализует и проверяет VIN.
    ///
    /// Алгоритм:
    /// 1. Убираем внешние пробелы.
    /// 2. Удаляем все whitespace-символы внутри строки. Это позволяет принять
    ///    VIN, скопированный блоками или перенесенный на несколько строк.
    /// 3. Переводим символы в верхний регистр.
    /// 4. Пустой результат трактуем как отсутствие VIN и возвращаем `Ok(None)`.
    /// 5. Сначала проверяем длину. Так пользователь получает более точную
    ///    ошибку, если ввел слишком мало или слишком много символов.
    /// 6. Затем проверяем допустимый алфавит: ASCII alphanumeric, кроме
    ///    запрещенных `I`, `O`, `Q`.
    pub fn parse(input: &str) -> Result<Option<Self>, CarError> {
        let normalized: String = input
            .trim()
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .flat_map(|ch| ch.to_uppercase())
            .collect();

        if normalized.is_empty() {
            return Ok(None);
        }

        let actual = normalized.chars().count();

        if actual != VIN_LEN {
            return Err(CarError::InvalidVinLength {
                expected: VIN_LEN,
                actual,
            });
        }

        let has_invalid_chars = normalized
            .chars()
            .any(|ch| !ch.is_ascii_alphanumeric() || matches!(ch, 'I' | 'O' | 'Q'));

        if has_invalid_chars {
            return Err(CarError::InvalidVinCharacters);
        }

        Ok(Some(Self(normalized)))
    }

    /// Возвращает нормализованный VIN без копирования.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Показывает VIN в сохраненном каноническом виде.
impl std::fmt::Display for Vin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Заметка по автомобилю.
///
/// Пустая заметка не является доменным значением. Если пользователь ничего не
/// указал или очистил поле, в `Car` хранится `None`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CarNotes(String);

impl CarNotes {
    /// Нормализует заметку по автомобилю.
    ///
    /// Алгоритм:
    /// 1. Убираем внешние пробелы.
    /// 2. Пустой результат превращаем в `Ok(None)`.
    /// 3. Считаем длину в Unicode-символах, чтобы кириллица не считалась по
    ///    UTF-8 байтам.
    /// 4. При превышении лимита возвращаем `NotesTooLong`.
    /// 5. Иначе сохраняем trimmed-текст как `Some(CarNotes)`.
    pub fn parse(input: &str) -> Result<Option<Self>, CarError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(None);
        }

        let actual = trimmed.chars().count();

        if actual > MAX_CAR_NOTES_LEN {
            return Err(CarError::NotesTooLong {
                max: MAX_CAR_NOTES_LEN,
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
impl std::fmt::Display for CarNotes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Ошибка работы с автомобилем и его value objects.
///
/// Ошибки оставлены рядом с типами, потому что именно этот модуль определяет
/// правила валидности марки, модели, года, номера, VIN, заметок и временных
/// меток автомобиля.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CarError {
    /// Марка пустая после удаления пробелов по краям.
    #[error("car make is empty")]
    EmptyMake,

    /// Марка превышает допустимую длину в Unicode-символах.
    #[error("car make is too long: max={max}, actual={actual}")]
    MakeTooLong { max: usize, actual: usize },

    /// Модель пустая после удаления пробелов по краям.
    #[error("car model is empty")]
    EmptyModel,

    /// Модель превышает допустимую длину в Unicode-символах.
    #[error("car model is too long: max={max}, actual={actual}")]
    ModelTooLong { max: usize, actual: usize },

    /// Год выпуска выходит за допустимый диапазон.
    #[error("car year is invalid: min={min}, max={max}, actual={actual}")]
    InvalidYear { min: u16, max: u16, actual: u16 },

    /// Нормализованный номер превышает допустимую длину.
    #[error("license plate is too long: max={max}, actual={actual}")]
    PlateTooLong { max: usize, actual: usize },

    /// VIN имеет длину, отличную от 17 символов.
    #[error("VIN has invalid length: expected={expected}, actual={actual}")]
    InvalidVinLength { expected: usize, actual: usize },

    /// VIN содержит недопустимые символы.
    #[error("VIN contains invalid characters")]
    InvalidVinCharacters,

    /// Заметка превышает допустимую длину в Unicode-символах.
    #[error("car notes are too long: max={max}, actual={actual}")]
    NotesTooLong { max: usize, actual: usize },

    /// Восстановленное или обновленное состояние нарушает временной порядок.
    #[error("car updated_at cannot be earlier than created_at")]
    UpdatedAtBeforeCreatedAt,
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use super::{
        Car, CarError, CarMake, CarModel, CarNotes, CarYear, LicensePlate, Vin, MAX_CAR_MAKE_LEN,
        MAX_CAR_MODEL_LEN, MAX_CAR_NOTES_LEN, MAX_CAR_YEAR, MAX_LICENSE_PLATE_LEN, MIN_CAR_YEAR,
        VIN_LEN,
    };
    use crate::{CarId, ClientId};

    fn fixed_time(seconds: i64) -> chrono::DateTime<Utc> {
        Utc.timestamp_opt(seconds, 0).single().unwrap()
    }

    fn car_id() -> CarId {
        CarId::from_uuid(Uuid::from_u128(1))
    }

    fn client_id() -> ClientId {
        ClientId::from_uuid(Uuid::from_u128(2))
    }

    fn car_make(value: &str) -> CarMake {
        CarMake::parse(value).unwrap()
    }

    fn car_model(value: &str) -> CarModel {
        CarModel::parse(value).unwrap()
    }

    fn car_year(value: u16) -> CarYear {
        CarYear::new(value).unwrap()
    }

    fn license_plate(value: &str) -> LicensePlate {
        LicensePlate::parse(value).unwrap().unwrap()
    }

    fn vin(value: &str) -> Vin {
        Vin::parse(value).unwrap().unwrap()
    }

    fn car_notes(value: &str) -> CarNotes {
        CarNotes::parse(value).unwrap().unwrap()
    }

    fn full_car(now: chrono::DateTime<Utc>) -> Car {
        Car::new(
            car_id(),
            client_id(),
            car_make("Toyota"),
            car_model("Camry"),
            Some(car_year(2020)),
            Some(license_plate("1234 ab-7")),
            Some(vin("1HGCM82633A004352")),
            Some(car_notes("Первичный осмотр")),
            now,
        )
    }

    /// Марка очищается от внешних пробелов и сохраняется в каноническом виде.
    #[test]
    fn car_make_parse_trims_valid_make() {
        let make = CarMake::parse("  Toyota  ").unwrap();

        assert_eq!(make.as_str(), "Toyota");
        assert_eq!(make.to_string(), "Toyota");
    }

    /// Строка из пробелов не должна становиться маркой автомобиля.
    #[test]
    fn car_make_parse_rejects_empty_make_after_trim() {
        let error = CarMake::parse("   ").unwrap_err();

        assert_eq!(error, CarError::EmptyMake);
    }

    /// Лимит марки считается в символах, а не в байтах.
    #[test]
    fn car_make_parse_allows_unicode_make_at_max_length() {
        let input = "Ж".repeat(MAX_CAR_MAKE_LEN);

        let make = CarMake::parse(&input).unwrap();

        assert_eq!(make.as_str().chars().count(), MAX_CAR_MAKE_LEN);
    }

    /// При превышении лимита ошибка возвращает максимум и фактическую длину.
    #[test]
    fn car_make_parse_rejects_too_long_make() {
        let input = "a".repeat(MAX_CAR_MAKE_LEN + 1);

        let error = CarMake::parse(&input).unwrap_err();

        assert_eq!(
            error,
            CarError::MakeTooLong {
                max: MAX_CAR_MAKE_LEN,
                actual: MAX_CAR_MAKE_LEN + 1,
            }
        );
    }

    /// Модель проходит тот же trim-алгоритм, что и марка.
    #[test]
    fn car_model_parse_trims_valid_model() {
        let model = CarModel::parse("  Camry XV70  ").unwrap();

        assert_eq!(model.as_str(), "Camry XV70");
        assert_eq!(model.to_string(), "Camry XV70");
    }

    /// Пустая после trim модель отклоняется.
    #[test]
    fn car_model_parse_rejects_empty_model_after_trim() {
        let error = CarModel::parse("\n\t").unwrap_err();

        assert_eq!(error, CarError::EmptyModel);
    }

    /// Unicode-модель на границе лимита валидна.
    #[test]
    fn car_model_parse_allows_unicode_model_at_max_length() {
        let input = "ю".repeat(MAX_CAR_MODEL_LEN);

        let model = CarModel::parse(&input).unwrap();

        assert_eq!(model.as_str().chars().count(), MAX_CAR_MODEL_LEN);
    }

    /// Слишком длинная модель возвращает структурированную ошибку.
    #[test]
    fn car_model_parse_rejects_too_long_model() {
        let input = "a".repeat(MAX_CAR_MODEL_LEN + 1);

        let error = CarModel::parse(&input).unwrap_err();

        assert_eq!(
            error,
            CarError::ModelTooLong {
                max: MAX_CAR_MODEL_LEN,
                actual: MAX_CAR_MODEL_LEN + 1,
            }
        );
    }

    /// Минимальная и максимальная границы года включены в допустимый диапазон.
    #[test]
    fn car_year_accepts_boundary_values() {
        let min = CarYear::new(MIN_CAR_YEAR).unwrap();
        let max = CarYear::new(MAX_CAR_YEAR).unwrap();

        assert_eq!(min.value(), MIN_CAR_YEAR);
        assert_eq!(max.value(), MAX_CAR_YEAR);
        assert_eq!(max.to_string(), MAX_CAR_YEAR.to_string());
    }

    /// Год ниже допустимой границы отклоняется.
    #[test]
    fn car_year_rejects_value_below_min() {
        let error = CarYear::new(MIN_CAR_YEAR - 1).unwrap_err();

        assert_eq!(
            error,
            CarError::InvalidYear {
                min: MIN_CAR_YEAR,
                max: MAX_CAR_YEAR,
                actual: MIN_CAR_YEAR - 1,
            }
        );
    }

    /// Год выше допустимой границы отклоняется.
    #[test]
    fn car_year_rejects_value_above_max() {
        let error = CarYear::new(MAX_CAR_YEAR + 1).unwrap_err();

        assert_eq!(
            error,
            CarError::InvalidYear {
                min: MIN_CAR_YEAR,
                max: MAX_CAR_YEAR,
                actual: MAX_CAR_YEAR + 1,
            }
        );
    }

    /// Номер очищается от разделителей и переводится в верхний регистр.
    #[test]
    fn license_plate_parse_normalizes_visual_separators_and_case() {
        let plate = LicensePlate::parse("  ab-1234 cd  ").unwrap().unwrap();

        assert_eq!(plate.as_str(), "AB1234CD");
        assert_eq!(plate.to_string(), "AB1234CD");
    }

    /// Если после удаления разделителей ничего не осталось, номер отсутствует.
    #[test]
    fn license_plate_parse_returns_none_for_empty_input() {
        let plate = LicensePlate::parse(" - \n ").unwrap();

        assert!(plate.is_none());
    }

    /// Длина номера проверяется после нормализации.
    #[test]
    fn license_plate_parse_allows_plate_at_max_length() {
        let input = "a".repeat(MAX_LICENSE_PLATE_LEN);

        let plate = LicensePlate::parse(&input).unwrap().unwrap();

        assert_eq!(plate.as_str().chars().count(), MAX_LICENSE_PLATE_LEN);
    }

    /// Слишком длинный нормализованный номер отклоняется.
    #[test]
    fn license_plate_parse_rejects_too_long_plate() {
        let input = "a".repeat(MAX_LICENSE_PLATE_LEN + 1);

        let error = LicensePlate::parse(&input).unwrap_err();

        assert_eq!(
            error,
            CarError::PlateTooLong {
                max: MAX_LICENSE_PLATE_LEN,
                actual: MAX_LICENSE_PLATE_LEN + 1,
            }
        );
    }

    /// VIN приводится к верхнему регистру и сохраняется без внутренних пробелов.
    #[test]
    fn vin_parse_normalizes_case_and_whitespace() {
        let vin = Vin::parse(" 1hg cm82633a004352 ").unwrap().unwrap();

        assert_eq!(vin.as_str(), "1HGCM82633A004352");
        assert_eq!(vin.to_string(), "1HGCM82633A004352");
    }

    /// Пустой VIN представлен как отсутствие значения, а не как ошибка.
    #[test]
    fn vin_parse_returns_none_for_empty_input() {
        let vin = Vin::parse(" \n\t ").unwrap();

        assert!(vin.is_none());
    }

    /// Ошибка длины возвращается до проверки запрещенных символов.
    #[test]
    fn vin_parse_rejects_invalid_length() {
        let error = Vin::parse("1234567890123456").unwrap_err();

        assert_eq!(
            error,
            CarError::InvalidVinLength {
                expected: VIN_LEN,
                actual: VIN_LEN - 1,
            }
        );
    }

    /// VIN не допускает буквы I, O и Q.
    #[test]
    fn vin_parse_rejects_forbidden_letters() {
        let error = Vin::parse("1HGCM82633A00435Q").unwrap_err();

        assert_eq!(error, CarError::InvalidVinCharacters);
    }

    /// VIN должен состоять только из ASCII-букв и цифр.
    #[test]
    fn vin_parse_rejects_non_ascii_characters() {
        let error = Vin::parse("1HGCM82633A00435Я").unwrap_err();

        assert_eq!(error, CarError::InvalidVinCharacters);
    }

    /// Непустая заметка очищается от внешних пробелов.
    #[test]
    fn car_notes_parse_trims_non_empty_notes() {
        let notes = CarNotes::parse("  Проверить тормоза  ").unwrap().unwrap();

        assert_eq!(notes.as_str(), "Проверить тормоза");
        assert_eq!(notes.to_string(), "Проверить тормоза");
    }

    /// Пустая заметка не является ошибкой и возвращается как `None`.
    #[test]
    fn car_notes_parse_returns_none_for_empty_input() {
        let notes = CarNotes::parse("   ").unwrap();

        assert!(notes.is_none());
    }

    /// Заметки считаются в Unicode-символах.
    #[test]
    fn car_notes_parse_allows_unicode_notes_at_max_length() {
        let input = "ю".repeat(MAX_CAR_NOTES_LEN);

        let notes = CarNotes::parse(&input).unwrap().unwrap();

        assert_eq!(notes.as_str().chars().count(), MAX_CAR_NOTES_LEN);
    }

    /// Слишком длинная заметка возвращает структурированную ошибку.
    #[test]
    fn car_notes_parse_rejects_too_long_notes() {
        let input = "a".repeat(MAX_CAR_NOTES_LEN + 1);

        let error = CarNotes::parse(&input).unwrap_err();

        assert_eq!(
            error,
            CarError::NotesTooLong {
                max: MAX_CAR_NOTES_LEN,
                actual: MAX_CAR_NOTES_LEN + 1,
            }
        );
    }

    /// Новый автомобиль получает одинаковые даты создания и обновления.
    #[test]
    fn car_new_sets_initial_state_and_timestamps() {
        let now = fixed_time(1_700_000_000);
        let car = full_car(now);

        assert_eq!(car.id(), car_id());
        assert_eq!(car.client_id(), client_id());
        assert_eq!(car.make().as_str(), "Toyota");
        assert_eq!(car.model().as_str(), "Camry");
        assert_eq!(car.year().unwrap().value(), 2020);
        assert_eq!(car.plate_number().unwrap().as_str(), "1234AB7");
        assert_eq!(car.vin().unwrap().as_str(), "1HGCM82633A004352");
        assert_eq!(car.notes().unwrap().as_str(), "Первичный осмотр");
        assert_eq!(*car.created_at(), now);
        assert_eq!(*car.updated_at(), now);
    }

    /// Restore сохраняет переданные даты, если временной порядок корректен.
    #[test]
    fn car_restore_accepts_valid_persisted_state() {
        let created_at = fixed_time(1_700_000_000);
        let updated_at = fixed_time(1_700_000_500);

        let car = Car::restore(
            car_id(),
            client_id(),
            car_make("BMW"),
            car_model("X5"),
            Some(car_year(2021)),
            None,
            None,
            None,
            created_at,
            updated_at,
        )
        .unwrap();

        assert_eq!(car.make().as_str(), "BMW");
        assert_eq!(car.model().as_str(), "X5");
        assert_eq!(car.year().unwrap().value(), 2021);
        assert!(car.plate_number().is_none());
        assert!(car.vin().is_none());
        assert!(car.notes().is_none());
        assert_eq!(*car.created_at(), created_at);
        assert_eq!(*car.updated_at(), updated_at);
    }

    /// Restore отклоняет поврежденное состояние, где обновление раньше создания.
    #[test]
    fn car_restore_rejects_updated_at_before_created_at() {
        let created_at = fixed_time(1_700_000_500);
        let updated_at = fixed_time(1_700_000_000);

        let error = Car::restore(
            car_id(),
            client_id(),
            car_make("BMW"),
            car_model("X5"),
            None,
            None,
            None,
            None,
            created_at,
            updated_at,
        )
        .unwrap_err();

        assert_eq!(error, CarError::UpdatedAtBeforeCreatedAt);
    }

    /// Обновление идентичности меняет марку, модель, год и дату обновления.
    #[test]
    fn update_identity_replaces_identity_and_touches_car() {
        let created_at = fixed_time(1_700_000_000);
        let updated_at = fixed_time(1_700_000_100);
        let mut car = full_car(created_at);

        car.update_identity(
            car_make("Audi"),
            car_model("A6"),
            Some(car_year(2022)),
            updated_at,
        )
        .unwrap();

        assert_eq!(car.make().as_str(), "Audi");
        assert_eq!(car.model().as_str(), "A6");
        assert_eq!(car.year().unwrap().value(), 2022);
        assert_eq!(*car.created_at(), created_at);
        assert_eq!(*car.updated_at(), updated_at);
    }

    /// При некорректном времени identity-обновление не меняет состояние.
    #[test]
    fn update_identity_rejects_time_before_creation_without_changes() {
        let created_at = fixed_time(1_700_000_000);
        let invalid_time = fixed_time(1_699_999_999);
        let mut car = full_car(created_at);

        let error = car
            .update_identity(car_make("Audi"), car_model("A6"), None, invalid_time)
            .unwrap_err();

        assert_eq!(error, CarError::UpdatedAtBeforeCreatedAt);
        assert_eq!(car.make().as_str(), "Toyota");
        assert_eq!(car.model().as_str(), "Camry");
        assert_eq!(car.year().unwrap().value(), 2020);
        assert_eq!(*car.updated_at(), created_at);
    }

    /// Номер можно заменить или очистить, а `updated_at` должен измениться.
    #[test]
    fn update_plate_number_replaces_and_clears_plate() {
        let created_at = fixed_time(1_700_000_000);
        let first_update = fixed_time(1_700_000_100);
        let second_update = fixed_time(1_700_000_200);
        let mut car = full_car(created_at);

        car.update_plate_number(Some(license_plate("7777 aa-7")), first_update)
            .unwrap();
        assert_eq!(car.plate_number().unwrap().as_str(), "7777AA7");
        assert_eq!(*car.updated_at(), first_update);

        car.update_plate_number(None, second_update).unwrap();
        assert!(car.plate_number().is_none());
        assert_eq!(*car.updated_at(), second_update);
    }

    /// Некорректное время не должно очищать или заменять номер.
    #[test]
    fn update_plate_number_rejects_time_before_creation_without_changes() {
        let created_at = fixed_time(1_700_000_000);
        let invalid_time = fixed_time(1_699_999_999);
        let mut car = full_car(created_at);

        let error = car.update_plate_number(None, invalid_time).unwrap_err();

        assert_eq!(error, CarError::UpdatedAtBeforeCreatedAt);
        assert_eq!(car.plate_number().unwrap().as_str(), "1234AB7");
        assert_eq!(*car.updated_at(), created_at);
    }

    /// VIN можно заменить или очистить, а `updated_at` должен измениться.
    #[test]
    fn update_vin_replaces_and_clears_vin() {
        let created_at = fixed_time(1_700_000_000);
        let first_update = fixed_time(1_700_000_100);
        let second_update = fixed_time(1_700_000_200);
        let mut car = full_car(created_at);

        car.update_vin(Some(vin("3FA6P0H75ER208976")), first_update)
            .unwrap();
        assert_eq!(car.vin().unwrap().as_str(), "3FA6P0H75ER208976");
        assert_eq!(*car.updated_at(), first_update);

        car.update_vin(None, second_update).unwrap();
        assert!(car.vin().is_none());
        assert_eq!(*car.updated_at(), second_update);
    }

    /// Некорректное время не должно менять VIN.
    #[test]
    fn update_vin_rejects_time_before_creation_without_changes() {
        let created_at = fixed_time(1_700_000_000);
        let invalid_time = fixed_time(1_699_999_999);
        let mut car = full_car(created_at);

        let error = car.update_vin(None, invalid_time).unwrap_err();

        assert_eq!(error, CarError::UpdatedAtBeforeCreatedAt);
        assert_eq!(car.vin().unwrap().as_str(), "1HGCM82633A004352");
        assert_eq!(*car.updated_at(), created_at);
    }

    /// Заметку можно заменить, очистить через update и очистить явным методом.
    #[test]
    fn update_notes_and_clear_notes_manage_optional_notes() {
        let created_at = fixed_time(1_700_000_000);
        let first_update = fixed_time(1_700_000_100);
        let second_update = fixed_time(1_700_000_200);
        let third_update = fixed_time(1_700_000_300);
        let mut car = full_car(created_at);

        car.update_notes(Some(car_notes("Повторный визит")), first_update)
            .unwrap();
        assert_eq!(car.notes().unwrap().as_str(), "Повторный визит");
        assert_eq!(*car.updated_at(), first_update);

        car.update_notes(None, second_update).unwrap();
        assert!(car.notes().is_none());
        assert_eq!(*car.updated_at(), second_update);

        car.update_notes(Some(car_notes("Перед выдачей помыть")), third_update)
            .unwrap();
        car.clear_notes(third_update).unwrap();
        assert!(car.notes().is_none());
        assert_eq!(*car.updated_at(), third_update);
    }

    /// Некорректное время не должно менять заметку.
    #[test]
    fn update_notes_rejects_time_before_creation_without_changes() {
        let created_at = fixed_time(1_700_000_000);
        let invalid_time = fixed_time(1_699_999_999);
        let mut car = full_car(created_at);

        let error = car.update_notes(None, invalid_time).unwrap_err();

        assert_eq!(error, CarError::UpdatedAtBeforeCreatedAt);
        assert_eq!(car.notes().unwrap().as_str(), "Первичный осмотр");
        assert_eq!(*car.updated_at(), created_at);
    }

    /// `clear_notes` тоже обязан защищать временной инвариант.
    #[test]
    fn clear_notes_rejects_time_before_creation_without_changes() {
        let created_at = fixed_time(1_700_000_000);
        let invalid_time = fixed_time(1_699_999_999);
        let mut car = full_car(created_at);

        let error = car.clear_notes(invalid_time).unwrap_err();

        assert_eq!(error, CarError::UpdatedAtBeforeCreatedAt);
        assert_eq!(car.notes().unwrap().as_str(), "Первичный осмотр");
        assert_eq!(*car.updated_at(), created_at);
    }
}
