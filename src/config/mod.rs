// config/mod.rs
pub mod bot;
pub mod database;
pub mod logging;
pub mod api;

use dotenv::dotenv;
use crate::error::Error;

// Re-export configuration components
pub use bot::BotConfig;
pub use database::DatabaseConfig;
pub use logging::LoggingConfig;
pub use api::ApiConfig;

#[derive(Clone, Debug)]
pub struct Config {
    pub bot: BotConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub api: ApiConfig,
}

impl Config {
    pub async fn load() -> Result<Self, Error> {
        // Load .env file if present
        dotenv().ok();
        
        Ok(Self {
            bot: BotConfig::load()?,
            database: DatabaseConfig::load()?,
            logging: LoggingConfig::load()?,
            api: ApiConfig::load()?,
        })
    }
}
