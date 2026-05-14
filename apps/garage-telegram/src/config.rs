#[derive(Debug, Clone)]
pub struct Config {
    pub bot_token: String,
}

impl Config {
    pub fn from_env() -> Self {
        let bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
            .or_else(|_| std::env::var("BOT_TOKEN"))
            .or_else(|_| std::env::var("TELOXIDE_TOKEN"))
            .expect("TELEGRAM_BOT_TOKEN, BOT_TOKEN or TELOXIDE_TOKEN must be set");

        Self { bot_token }
    }
}
