#[derive(Debug, Clone)]
pub struct Config {
    pub bot_token: String,
    pub database_url: String,
    pub timezone_offset_hours: i32,
    pub owner_chat_id: Option<i64>,
}

impl Config {
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
