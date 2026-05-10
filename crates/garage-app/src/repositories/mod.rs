//! Repository ports for application services.
//!
//! These traits describe persistence needs of use cases. Concrete PostgreSQL
//! implementations belong to `garage-infra`.

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
