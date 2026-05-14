use crate::config::Config;

#[derive(Debug, Clone)]
pub struct AppContainer {
    config: Config,
}

impl AppContainer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn bot_token(&self) -> &str {
        &self.config.bot_token
    }
}
