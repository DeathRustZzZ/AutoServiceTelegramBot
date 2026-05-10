//! Порты репозиториев для application services.
//!
//! Traits в этом модуле описывают, какие операции хранения нужны use case'ам.
//! Они не описывают SQL-схему и не зависят от SQLx. PostgreSQL-реализации
//! должны жить в `garage-infra`, а тесты могут использовать in-memory fake.

mod booking;
mod car;
mod client;
mod part;
mod part_supply;
mod repair;

pub use booking::*;
pub use car::*;
pub use client::*;
pub use part::*;
pub use part_supply::*;
pub use repair::*;
