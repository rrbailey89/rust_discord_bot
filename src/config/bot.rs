// config/bot.rs
use serde::Deserialize;
use std::env;
use crate::error::Error;

#[derive(Clone, Deserialize, Debug)]
pub struct BotConfig {
    pub bot_token: String,
    pub command_prefix: String,
    pub serena_user_id: String,
    #[serde(default)]
    pub application_id: Option<String>,
}

impl BotConfig {
    pub fn load() -> Result<Self, Error> {
        let application_id = env::var("APPLICATION_ID").ok();
        
        if application_id.is_none() {
            tracing::warn!("APPLICATION_ID not set, Discord command sync will be disabled");
        }
        
        Ok(Self {
            bot_token: env::var("BOT_TOKEN").unwrap_or_else(|_| String::from("")),
            command_prefix: env::var("COMMAND_PREFIX").unwrap_or_else(|_| String::from("!")),
            serena_user_id: env::var("SERENA_USER_ID").unwrap_or_else(|_| {
                // Set Serena's ID as default
                tracing::warn!("SERENA_USER_ID not set, using hardcoded default");
                String::from("803867382447079485")  // Serena's ID
            }),
            application_id,
        })
    }
}
