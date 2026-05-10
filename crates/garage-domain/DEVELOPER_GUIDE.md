# garage-domain — Developer Guide

> Автор: DeathCrushZzZ
> Крейт: `crates/garage-domain`  
> Версия: workspace 0.1.0

---

## 1. Архитектурный контекст

`garage-domain` — это **чистый доменный слой** в классической слоистой архитектуре:

```
┌─────────────────────────┐
│   apps/garage-telegram  │  ← UI / Telegram handlers
├─────────────────────────┤
│     crates/garage-app   │  ← Application services, use cases
├─────────────────────────┤
│    crates/garage-infra  │  ← Repositories, DB, external APIs
├─────────────────────────┤
│   crates/garage-domain  │  ← Entities, Value Objects, Domain Errors
└─────────────────────────┘
```

**Ключевое правило:** `garage-domain` не зависит ни от одного другого крейта проекта.  
Зависимости строго утилитарные: `uuid`, `chrono`, `serde`, `thiserror`.

---

## 2. Структура модулей

```
src/
 ├── lib.rs          — pub mod + pub use * (плоский re-export)
 ├── ids.rs          — типобезопасные идентификаторы (newtype over Uuid)
 ├── money.rs        — Money, SignedMoney, Currency
 ├── phone.rs        — PhoneNumber (value object с нормализацией)
 ├── client.rs       — Client + ClientName + ClientNotes
 ├── car.rs          — Car + CarMake, CarModel, CarYear, LicensePlate, Vin, CarNotes
 ├── booking.rs      — Booking + BookingReason + BookingNotes + BookingStatus
 ├── repair.rs       — Repair + RepairDescription + RepairNotes + RepairStatus + PaymentStatus
 ├── part.rs         — Part + PartName + PartSku + PartQuantity + PartNotes
 └── part_supply.rs  — PartSupply + PartSupplier + PartSupplyNotes + PartSupplyStatus
```

---

## 3. Фундаментальные паттерны

### 3.1 Parse, don't validate

**Принцип:** данные нормализуются и проверяются **один раз** при создании value object. После этого весь остальной код работает с уже валидным типом.

```rust
// ❌ Антипаттерн: строка везде
fn create_client(name: String, phone: String) {
    if name.is_empty() { /* ... */ }
    if phone.len() != 12 { /* ... */ }
    // повторяется в каждом сервисе
}

// ✅ Паттерн из кодовой базы
let name = ClientName::parse("  Иван Петрович  ")?;  // → "Иван Петрович"
let phone = PhoneNumber::parse("+375 (29) 123-45-67")?;  // → "+375291234567"
// Дальше — только валидные типы
let client = Client::new(id, name, phone, notes, Utc::now());
```

**Алгоритм parse для обязательных полей:**
1. `trim()` — убираем внешний шум из Telegram/форм
2. Проверка пустоты (`Empty` ошибка)
3. `chars().count()` — длина в Unicode-символах (не байтах!)
4. Проверка лимита (`TooLong { max, actual }` ошибка)
5. Сохраняем trimmed-строку

**Алгоритм parse для опциональных полей** (notes, supplier и т.д.):
1. `trim()`
2. Если пусто → `Ok(None)` (не ошибка!)
3. Если непусто → проверка лимита → `Ok(Some(T))`

```rust
// Опциональные поля никогда не хранят пустую строку
let notes = ClientNotes::parse("")?;  // → Ok(None)
let notes = ClientNotes::parse("  ")?;  // → Ok(None)
let notes = ClientNotes::parse("VIP клиент")?;  // → Ok(Some(...))
```

---

### 3.2 Newtype-идентификаторы

Каждая сущность имеет **свой тип идентификатора**, несовместимый с остальными:

```rust
pub struct ClientId(Uuid);
pub struct CarId(Uuid);
pub struct BookingId(Uuid);
// ...
```

**Три метода — три сценария:**

| Метод | Сценарий |
|---|---|
| `XxxId::new()` | Создание новой сущности внутри домена |
| `XxxId::from_uuid(uuid)` | Восстановление из БД / входящего API |
| `id.as_uuid()` | Передача в БД / API / логи |

```rust
// Создание
let client_id = ClientId::new();

// Восстановление из БД (репозиторий)
let client_id = ClientId::from_uuid(row.id);

// Запись в БД
stmt.bind(client_id.as_uuid());
```

**Почему не `impl From<Uuid>`?**  
`from_uuid` более явный: в коде сразу видно, что происходит пересечение слоёв.

**Улучшение:** рассмотрите добавление `Display` и `FromStr` для ID на границах системы:

```rust
impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

---

### 3.3 Сущности: new / restore / touch

Каждая доменная сущность следует одному контракту:

#### `new(...)` — создание
- Принимает **уже валидные value objects** (не строки)
- Устанавливает начальный статус жизненного цикла
- `created_at == updated_at == now` (один момент времени)
- Возвращает `Self` без `Result` (типы уже гарантируют корректность)

```rust
// new не может получить невалидный ClientName — тип уже проверен
let client = Client::new(id, name, phone, notes, Utc::now());
```

#### `restore(...)` — восстановление из хранилища
- Принимает **исторические даты** из внешнего слоя
- Обязательно проверяет временной инвариант: `created_at <= updated_at`
- Для `Repair` — расширенная проверка: `started_at <= created_at <= updated_at`, `completed_at` только у `Completed`
- Возвращает `Result<Self, XxxError>`

```rust
// Репозиторий восстанавливает value objects, затем вызывает restore
let client = Client::restore(id, name, phone, notes, created_at, updated_at)?;
```

#### `touch(&mut self, now)` — приватный метод обновления времени
- Проверяет `now >= created_at`
- Обновляет `updated_at`
- Вызывается **первым** во всех мутирующих методах → при ошибке состояние не меняется

```rust
// Атомарность: сначала touch, потом изменение поля
pub fn rename(&mut self, name: ClientName, now: DateTime<Utc>) -> Result<(), ClientError> {
    self.touch(now)?;   // если время некорректно — сразу ошибка, поле не меняется
    self.name = name;
    Ok(())
}
```

---

### 3.4 State machines с финальными статусами

Статусы описывают **жизненный цикл**, а не хранимое поле напрямую.

```
BookingStatus:
    Scheduled ──→ Completed
              ──→ Cancelled
              ──→ NoShow
    (финальные статусы не переходят в другие)

RepairStatus:
    InProgress ──→ Completed
               ──→ Cancelled

PartSupplyStatus:
    Expected ──→ Received
             ──→ Cancelled
```

**Защита от мутации финальных сущностей:**

```rust
fn ensure_scheduled_for_modification(&self) -> Result<(), BookingError> {
    if self.is_terminal() {
        return Err(BookingError::TerminalStatus { status: self.status });
    }
    Ok(())
}

pub fn reschedule(&mut self, at: DateTime<Utc>, now: DateTime<Utc>) -> Result<(), BookingError> {
    self.ensure_scheduled_for_modification()?;  // ← первая проверка
    self.touch(now)?;
    self.scheduled_at = at;
    Ok(())
}
```

**PaymentStatus — вычисляемый, не хранимый:**
```rust
// Не хранится как поле — вычисляется из paid_amount vs total_price
// Это исключает противоречивые состояния (Paid при оплате 50%)
pub fn payment_status(&self) -> PaymentStatus { ... }
```

---

### 3.5 Деньги как целые числа

**Никогда не используйте f32/f64 для денег!**

```rust
// ✅ Хранение в минимальных единицах
// 10.50 BYN → amount_minor = 1050
// 25.99 USD → amount_minor = 2599

let price = Money::byn_minor(1050)?;  // 10.50 BYN
let zero  = Money::zero(Currency::Byn);

// Арифметика только внутри одной валюты
let total = labor.checked_add(parts)?;        // Ok или CurrencyMismatch
let rest  = total.checked_sub(paid)?;         // Ok или NegativeAmount / Overflow

// Для аналитики (прибыль может быть отрицательной)
let profit: SignedMoney = revenue.checked_sub(cost)?;
```

**Правила для новых финансовых операций:**
1. Все поля одной сущности — **одна валюта** (проверяется в `new` и `restore`)
2. Используйте `checked_add` / `checked_sub` — никогда не `+` / `-` напрямую
3. `Money` для цен и оплат, `SignedMoney` для прибыли/убытка

---

## 4. Соглашения по написанию кода

### 4.1 Unicode-aware длина строк

```rust
// ❌ Неверно: считает байты, кириллица даёт неверный результат
if value.len() > MAX_LEN { ... }

// ✅ Верно: считает символы
if value.chars().count() > MAX_LEN { ... }
```

### 4.2 Геттеры: Copy vs ссылка

```rust
// Copy-типы (Id, Status, Money, Currency, DateTime) — возвращаем по значению
pub fn id(&self) -> ClientId { self.id }
pub fn status(&self) -> BookingStatus { self.status }
pub fn labor_price(&self) -> Money { self.labor_price }

// Строковые value objects — возвращаем ссылку
pub fn name(&self) -> &ClientName { &self.name }

// Опциональные поля — Option<&T>
pub fn notes(&self) -> Option<&ClientNotes> { self.notes.as_ref() }
```

### 4.3 Именование методов мутации

| Действие | Шаблон | Пример |
|---|---|---|
| Изменить поле | `set_field` / `update_field` / `change_field` | `change_phone`, `update_notes` |
| Очистить опциональное | `clear_field` | `clear_notes` |
| Перевести статус | бизнес-имя | `reschedule`, `complete`, `cancel` |

### 4.4 Ошибки: один enum на модуль

```rust
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ClientError {
    #[error("client name is empty")]
    EmptyName,

    #[error("client name is too long: max={max}, actual={actual}")]
    NameTooLong { max: usize, actual: usize },

    #[error("client updated_at cannot be earlier than created_at")]
    UpdatedAtBeforeCreatedAt,
}
```

**Правила:**
- `#[derive(Debug, Clone, PartialEq, Eq)]` — обязательно (для тестов и сравнений)
- `thiserror::Error` — для сообщений
- Структурные варианты `{ max, actual }` — вместо строк, чтобы прикладной слой мог форматировать сам
- Ошибки живут в том же модуле, что и типы

---

## 5. Написание тестов

### 5.1 Вспомогательные функции

```rust
#[cfg(test)]
mod tests {
    // Фиксированное время для детерминированных тестов
    fn fixed_time(seconds: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(seconds, 0).single().unwrap()
    }

    // Фиксированные ID через from_uuid(Uuid::from_u128(N))
    fn client_id() -> ClientId {
        ClientId::from_uuid(Uuid::from_u128(1))
    }

    // Unwrap-хелперы для известно-валидных значений
    fn client_name(value: &str) -> ClientName {
        ClientName::parse(value).unwrap()
    }
}
```

### 5.2 Структура тестов

Каждый тест проверяет **один конкретный инвариант**:

```rust
/// Тест описывает поведение в docstring — что проверяем и почему
#[test]
fn client_name_parse_allows_unicode_name_at_max_length() {
    let input = "Я".repeat(MAX_CLIENT_NAME_LEN);
    let name = ClientName::parse(&input).unwrap();
    assert_eq!(name.as_str().chars().count(), MAX_CLIENT_NAME_LEN);
}

// Тест ошибки — используем unwrap_err()
#[test]
fn parse_rejects_empty_input() {
    let error = PhoneNumber::parse("   ").unwrap_err();
    assert_eq!(error, PhoneNumberError::Empty);
}
```

### 5.3 Что обязательно тестировать

Для каждого value object:
- [ ] Валидный ввод с пробелами по краям (trim)
- [ ] Кириллица на максимальной длине (chars vs bytes)
- [ ] Пустой ввод → ожидаемая ошибка
- [ ] Ввод на 1 символ больше лимита → TooLong с корректными max/actual
- [ ] `Display` возвращает канонический вид

Для каждой сущности:
- [ ] `new` → `created_at == updated_at`
- [ ] `restore` с `updated_at < created_at` → ошибка
- [ ] Мутирующий метод на финальной сущности → ошибка
- [ ] `touch` с `now < created_at` → ошибка без изменения состояния

---

## 6. Как добавить новую доменную сущность

**Чеклист:**

1. **Идентификатор** в `ids.rs`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ServiceOrderId(Uuid);

impl ServiceOrderId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
    pub fn from_uuid(value: Uuid) -> Self { Self(value) }
    pub fn as_uuid(&self) -> Uuid { self.0 }
}
```

2. **Value objects** — отдельный `parse` для каждого поля с инвариантами

3. **Enum ошибок** — один `XxxError` с `#[derive(Debug, Clone, PartialEq, Eq, Error)]`

4. **Сущность**:
   - `new(...)` без Result (типы гарантируют корректность)
   - `restore(...)` с проверкой временного порядка → Result
   - Приватный `touch(now)` → вызывается первым в мутирующих методах
   - Только `pub` геттеры (Copy по значению, остальное по ссылке)

5. **Экспорт** в `lib.rs`:
```rust
pub mod service_order;
pub use service_order::*;
```

6. **Тесты** — `#[cfg(test)] mod tests` в том же файле

---

## 7. Известные ограничения и точки роста

### 7.1 Glob-реэкспорт в lib.rs

```rust
// Текущий подход: pub use booking::*;
// Проблема: не очевидно, что именно экспортируется без открытия каждого модуля
```

**Альтернатива** — явный реэкспорт ключевых типов:
```rust
pub use booking::{Booking, BookingError, BookingReason, BookingNotes, BookingStatus};
```

### 7.2 IDs без Display/FromStr

Для логов и HTTP-маршрутов полезно добавить:
```rust
impl std::fmt::Display for ClientId { ... }
impl std::str::FromStr for ClientId { ... }
```

### 7.3 IDs без Default

Если где-то нужен `Default` (например, для derive-макросов):
```rust
impl Default for ClientId {
    fn default() -> Self { Self::new() }
}
```

### 7.4 Serde не подключён к доменным типам

Это **правильное решение**: домен не знает о сериализации.  
Конвертация в DTO происходит в `garage-app` или `garage-infra`.

### 7.5 Только белорусские номера в PhoneNumber

При расширении на другие страны — менять алгоритм `parse`, тесты покрывают контракт.

### 7.6 Money: нет форматирования для UI

`Money` хранит только minor units. Форматирование (`"10.50 BYN"`) — ответственность `garage-app` или слоя представления.

---

## 8. Быстрая справка: типичные сценарии

### Создание нового клиента
```rust
let id = ClientId::new();
let name = ClientName::parse(input_name)?;
let phone = PhoneNumber::parse(input_phone)?;
let notes = ClientNotes::parse(input_notes)?;  // Option<ClientNotes>
let client = Client::new(id, name, phone, notes, Utc::now());
```

### Восстановление клиента из БД (репозиторий)
```rust
let client = Client::restore(
    ClientId::from_uuid(row.id),
    ClientName::parse(&row.name)?,
    PhoneNumber::parse(&row.phone)?,
    row.notes.as_deref().map(ClientNotes::parse).transpose()?.flatten(),
    row.created_at,
    row.updated_at,
)?;
```

### Изменение имени
```rust
let new_name = ClientName::parse(new_input)?;
client.rename(new_name, Utc::now())?;
// Теперь client.updated_at() изменился
```

### Работа с деньгами
```rust
let labor = Money::byn_minor(5000)?;   // 50.00 BYN
let parts = Money::byn_minor(2000)?;   // 20.00 BYN
let total = labor.checked_add(parts)?; // 70.00 BYN

// Для отображения: total.amount_minor() / 100 = 70, остаток = total.amount_minor() % 100
```

### Перевод записи в финальный статус
```rust
if booking.is_terminal() {
    return Err(/* уже закрыта */);
}
booking.complete(Utc::now())?;
```
