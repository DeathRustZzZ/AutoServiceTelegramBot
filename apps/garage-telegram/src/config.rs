//! Конфигурация Telegram-приложения из переменных окружения.
//!
//! Модуль оставляет валидацию простой и ранней: обязательные значения приводят
//! к `expect` при старте, а опциональные настройки имеют безопасные значения
//! по умолчанию.

/// Runtime-настройки, необходимые Telegram-адаптеру.
#[derive(Debug, Clone)]
pub struct Config {
    /// Токен Telegram bot API.
    pub bot_token: String,
    /// PostgreSQL DSN для инфраструктурного слоя.
    pub database_url: String,
    /// Смещение локального времени автосервиса относительно UTC в часах.
    pub timezone_offset_hours: i32,
    /// Единственный разрешенный владелец чата/пользователь, если задан.
    pub owner_chat_id: Option<i64>,
}

impl Config {
    /// Загружает конфигурацию из окружения.
    ///
    /// Для токена поддержаны несколько имен переменных, чтобы не ломать
    /// стандартные ожидания teloxide и существующие `.env` файлы.
    pub fn from_env() -> Self {
        let bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
            .or_else(|_| std::env::var("BOT_TOKEN"))
            .or_else(|_| std::env::var("TELOXIDE_TOKEN"))
            .expect("TELEGRAM_BOT_TOKEN, BOT_TOKEN or TELOXIDE_TOKEN must be set");
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for garage-telegram");
        let timezone_offset_hours = std::env::var("APP_TIMEZONE_OFFSET_HOURS")
            .or_else(|_| std::env::var("LOCAL_TIMEZONE_OFFSET_HOURS"))
            .ok()
            .and_then(|value| value.parse::<i32>().ok())
            .unwrap_or(0);
        let owner_chat_id = std::env::var("OWNER_CHAT_ID").ok().and_then(|value| {
            let value = value.trim();
            (!value.is_empty())
                .then(|| value.parse::<i64>().ok())
                .flatten()
        });

        Self {
            bot_token,
            database_url,
            timezone_offset_hours,
            owner_chat_id,
        }
    }
}
