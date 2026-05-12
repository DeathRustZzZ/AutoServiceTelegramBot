//! Application services and use cases.
//!
//! Services coordinate repositories and domain entities. They do not know
//! anything about Telegram, SQLx or PostgreSQL.

mod common;

mod booking;
mod car;
mod client;
mod part;
mod part_query;
mod part_supply;
mod payment;
mod payment_transactional;
mod repair;
mod repair_part;
mod repair_part_transactional;
mod repair_query;
mod statistics;

pub use booking::*;
pub use car::*;
pub use client::*;
pub use part::*;
pub use part_query::*;
pub use part_supply::*;
pub use payment::*;
pub use payment_transactional::*;
pub use repair::*;
pub use repair_part::*;
pub use repair_part_transactional::*;
pub use repair_query::*;
pub use statistics::*;

#[cfg(test)]
mod tests;
