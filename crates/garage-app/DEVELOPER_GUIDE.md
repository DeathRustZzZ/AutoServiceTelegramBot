# garage-app — Developer Guide

> Крейт: `crates/garage-app`  
> Роль: application layer / use cases / repository ports  
> Текущая цель: MVP Telegram-бота без привязки к Telegram и PostgreSQL

---

## 1. Архитектурный контекст

`garage-app` находится между чистым доменом и внешними адаптерами:

```text
┌─────────────────────────┐
│   apps/garage-telegram  │  UI, handlers, buttons, dialogues
├─────────────────────────┤
│     crates/garage-app   │  use cases, services, repository traits
├─────────────────────────┤
│    crates/garage-infra  │  PostgreSQL, SQLx, migrations, repository impls
├─────────────────────────┤
│   crates/garage-domain  │  entities, value objects, domain errors
└─────────────────────────┘
```

Разрешенные зависимости:

- `garage-app -> garage-domain`
- `garage-app -> chrono / uuid / thiserror / async-trait`

Запрещенные зависимости:

- `garage-app -> garage-infra`
- `garage-app -> garage-telegram`
- `garage-app -> sqlx`
- `garage-app -> teloxide`

`garage-app` не знает, откуда пришла команда: Telegram, CLI, HTTP API или тест. Он получает уже подготовленные value objects и вызывает доменные методы.

---

## 2. Структура модулей

```text
src/
 ├── lib.rs
 ├── error.rs
 ├── repositories/
 │   ├── mod.rs
 │   ├── client.rs
 │   ├── car.rs
 │   ├── booking.rs
 │   ├── part.rs
 │   ├── part_supply.rs
 │   └── repair.rs
 └── services/
     ├── mod.rs
     ├── common.rs
     ├── client.rs
     ├── car.rs
     ├── booking.rs
     ├── part.rs
     ├── part_supply.rs
     ├── repair.rs
     ├── statistics.rs
     └── tests.rs
```

Публичный API остается плоским:

```rust
use garage_app::{
    BookingRepository,
    BookingService,
    ClientRepository,
    ClientService,
    PartService,
    RepairService,
    StatisticsService,
};
```

Подмодули `repositories::*` и `services::*` закрыты наружу. Наружу экспортируются сами traits, services и DTO app-layer вроде `ProfitSummary`.

---

## 3. Repository Ports

Repository traits — это порты application layer. Они описывают потребности сценариев, а не SQL-схему.

Пример:

```rust
#[async_trait]
pub trait ClientRepository: Send + Sync {
    async fn get(&self, id: ClientId) -> AppResult<Option<Client>>;
    async fn save(&self, client: &Client) -> AppResult<()>;
}
```

Правила:

1. `get` возвращает `Option<T>`.
   Отсутствие агрегата — нормальный бизнес-исход. Service превращает `None` в `AppError::*NotFound`.

2. `save` принимает агрегат целиком.
   Инфраструктура не должна обновлять отдельные поля в обход доменных методов.

3. Traits не содержат SQLx-типы.
   Ошибки PostgreSQL, transactions, rows и query builders остаются в `garage-infra`.

4. `Arc<T>` поддерживается прямо в app-layer.
   Это позволяет Telegram handlers держать `Arc<dyn Repository>` без wrapper-кода.

5. Методы выборки добавляются от сценариев, а не от структуры БД.
   Например, `BookingRepository::list_scheduled_between` нужен Telegram MVP для расписания на период.

---

## 4. Application Services

Application service — это orchestration, а не CRUD.

Типичный алгоритм:

1. Загрузить нужные агрегаты через repository ports.
2. Проверить cross-aggregate связи.
3. Вызвать доменные методы.
4. Сохранить измененные агрегаты.

Пример: создание booking.

```text
schedule_booking
 ├─ require_client(client_id)
 ├─ require_car(car_id)
 ├─ ensure_car_belongs_to_client(car, client_id)
 ├─ Booking::new(...)
 └─ bookings.save(...)
```

Домен проверяет внутренние инварианты сущности. App-layer проверяет связи между агрегатами:

- `Car -> Client`
- `Booking -> Client`
- `Booking -> Car`
- `PartSupply -> Part`

---

## 5. MVP-сценарии

### ClientService

- `create_client`
- `rename_client`
- `change_phone`
- `update_notes`

### CarService

- `create_car`
- `update_identity`
- `list_client_cars`

`list_client_cars` сначала проверяет существование клиента, затем запрашивает машины клиента.

### BookingService

- `schedule_booking`
- `reschedule_booking`
- `complete_booking`
- `cancel_booking`
- `mark_no_show`
- `list_client_bookings`
- `list_car_bookings`
- `list_bookings_between`

`list_bookings_between` принимает `DateTime<Utc>` диапазон. Локальные today/tomorrow границы должен посчитать UI/adaptor layer.

### PartService

- `create_part`
- `set_stock`
- `search_parts`
- `list_low_stock`

`search_parts` не решает, искать по name, sku или другой индексной стратегии. Это ответственность реализации `PartRepository`.

### PartSupplyService

- `create_supply`
- `receive_supply`
- `cancel_supply`

`receive_supply` меняет два агрегата:

```text
PartSupply::mark_received(now)
Part::increase_stock(supply.quantity(), now)
```

PostgreSQL-реализация должна сохранить оба изменения в одной транзакции.

### RepairService

- `start_repair`
- `record_payment`
- `complete_repair`
- `cancel_repair`

`start_repair` проверяет клиента, автомобиль и опциональный booking. Финансовые правила остаются в `Repair`.

### StatisticsService

- `profit_summary`

Статистика строится по `Repair`, не по `Booking`, потому что только repair содержит цены, себестоимость и оплаты.

---

## 6. Ошибки

Все public methods возвращают:

```rust
pub type AppResult<T> = Result<T, AppError>;
```

`AppError` делится на три группы:

1. Not found:
   - `ClientNotFound`
   - `CarNotFound`
   - `BookingNotFound`
   - `PartNotFound`
   - `PartSupplyNotFound`
   - `RepairNotFound`

2. Cross-aggregate consistency:
   - `CarDoesNotBelongToClient`
   - `BookingDoesNotBelongToClient`
   - `BookingDoesNotBelongToCar`

3. Wrapped domain/repository errors:
   - `Client`
   - `Car`
   - `Booking`
   - `Part`
   - `PartSupply`
   - `Repair`
   - `Money`
   - `PhoneNumber`
   - `Repository(String)`

`Repository(String)` пока оставлен простым, чтобы не тащить SQLx в app-layer.

---

## 7. Транзакционные границы

`garage-app` описывает порядок операций, но не управляет SQL-транзакциями напрямую.

Сценарии с одним агрегатом:

- `ClientService::rename_client`
- `BookingService::complete_booking`
- `RepairService::record_payment`

Сценарии с несколькими агрегатами:

- `PartSupplyService::receive_supply`
- будущие сценарии списания запчастей на ремонт
- будущие сценарии возврата/коррекции оплаты

Для multi-aggregate сценариев `garage-infra` должен обеспечить атомарность. На текущем этапе это можно сделать через repository implementation или будущий Unit of Work.

---

## 8. Тестирование

Тесты `garage-app` используют in-memory `Store`, который реализует все repository traits.

Это проверяет именно application orchestration:

- not-found guards;
- ownership checks;
- списки клиента/машины;
- расписание booking по диапазону;
- поиск и low-stock запчастей;
- получение поставки с обновлением склада;
- старт ремонта из booking;
- оплаты, completion/cancellation;
- статистику прибыли по валюте.

Команды:

```bash
cargo test -p garage-app
cargo test
```

---

## 9. Coverage

В окружении проекта может не быть `cargo-llvm-cov` или `cargo-tarpaulin`. Coverage можно проверить системными LLVM-инструментами:

```bash
mkdir -p /tmp/garage-app-coverage/profraw

CARGO_TARGET_DIR=/tmp/garage-app-coverage \
RUSTFLAGS="-Cinstrument-coverage" \
LLVM_PROFILE_FILE="/tmp/garage-app-coverage/profraw/garage-app-%p-%m.profraw" \
cargo test -p garage-app

llvm-profdata merge -sparse \
  /tmp/garage-app-coverage/profraw/*.profraw \
  -o /tmp/garage-app-coverage/garage-app.profdata

llvm-cov report \
  /tmp/garage-app-coverage/debug/deps/garage_app-<hash> \
  --instr-profile=/tmp/garage-app-coverage/garage-app.profdata \
  --ignore-filename-regex='/.cargo/registry|rustc|crates/garage-domain|services/tests.rs|rust/library/std'
```

Текущий результат для production-кода `garage-app`:

```text
Regions: 82.64%
Functions: 92.86%
Lines: 93.22%
```

---

## 10. Что не добавлять в garage-app

Не добавлять:

- Telegram handlers;
- keyboard/button builders;
- SQLx queries;
- PostgreSQL transactions;
- migration code;
- DTO базы данных;
- timezone today/tomorrow логику;
- HTTP clients для курсов валют.

Если сценарию нужна внешняя система, добавляется port в `garage-app`, а реализация остается в `garage-infra`.

---

## 11. Следующие шаги

1. Реализовать repository traits в `garage-infra`.
2. Добавить PostgreSQL migrations.
3. Решить транзакционный подход для multi-aggregate сценариев.
4. Подключить `garage-telegram` к services через `Arc<dyn Repository>`.
5. Позже добавить provider курса валют для BYN/USD статистики.
