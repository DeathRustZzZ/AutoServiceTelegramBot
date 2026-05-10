//! Application services and use cases.
//!
//! Services coordinate repositories and domain entities. They do not know
//! anything about Telegram, SQLx or PostgreSQL.

mod common;

mod booking;
mod car;
mod client;
mod part;
mod part_supply;
mod repair;
mod statistics;

pub use booking::*;
pub use car::*;
pub use client::*;
pub use part::*;
pub use part_supply::*;
pub use repair::*;
pub use statistics::*;
