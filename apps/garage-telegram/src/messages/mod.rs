//! Тексты Telegram UI.
//!
//! Эти функции форматируют пользовательские сообщения из доменных агрегатов и
//! черновиков. Они не ходят в репозитории и не принимают решений по сценариям:
//! handler выбирает, какой текст показать, а message-модуль отвечает только за
//! стабильную формулировку и формат карточки.

pub mod bookings;
pub mod cars;
pub mod clients;
pub mod errors;
pub mod format;
pub mod main;
pub mod parts;
pub mod repairs;
