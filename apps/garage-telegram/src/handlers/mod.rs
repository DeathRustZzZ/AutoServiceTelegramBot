//! Handler'ы пользовательских сценариев Telegram.
//!
//! Handler слой переводит Telegram-события в вызовы `garage-app`: собирает
//! черновики, парсит пользовательский ввод в domain value objects, вызывает
//! сервисы и отдает результат в `messages`/`keyboards`. Доменные правила здесь
//! не дублируются.

pub mod bookings;
pub mod cars;
pub mod clients;
pub mod errors;
pub mod parts;
pub mod repairs;
pub mod start;
