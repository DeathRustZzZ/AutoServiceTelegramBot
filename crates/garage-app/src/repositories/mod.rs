//! Порты репозиториев для прикладных сервисов.
//!
//! Traits в этом модуле описывают, какие операции хранения нужны сценариям.
//! Они не описывают SQL-схему и не зависят от SQLx. PostgreSQL-реализации
//! должны жить в `garage-infra`, а тесты могут использовать in-memory fake.

mod booking;
mod car;
mod client;
mod part;
mod part_supply;
mod payment;
mod repair;
mod repair_part;
mod stock_movement;

pub use booking::*;
pub use car::*;
pub use client::*;
pub use part::*;
pub use part_supply::*;
pub use payment::*;
pub use repair::*;
pub use repair_part::*;
pub use stock_movement::*;
