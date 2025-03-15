// config/bot.rs
use serde::Deserialize;
use std::env;
use crate::error::Error;

#[derive(Clone, Deserialize, Debug)]
pub struct BotConfig {
    pub bot_token: String,
    pub command_prefix: String,
    pub serena_user_id: String,
}

impl BotConfig {
    pub fn load() -> Result<Self, Error> {
        Ok(Self {
            bot_token: env::var("BOT_TOKEN").unwrap_or_else(|_| String::from("")),
            command_prefix: env::var("COMMAND_PREFIX").unwrap_or_else(|_| String::from("!")),
            serena_user_id: env::var("SERENA_USER_ID").unwrap_or_else(|_| String::from("")),
        })
    }
}
