// config/api.rs
use serde::Deserialize;
use std::env;
use crate::error::Error;

#[derive(Clone, Deserialize, Debug)]
pub struct ApiConfig {
    pub openai_api_key: String,
    pub openweather_api_key: String,
    pub api_ninjas_key: String,
    pub bfl_api_key: String,
}

impl ApiConfig {
    pub fn load() -> Result<Self, Error> {
        Ok(Self {
            openai_api_key: env::var("OPENAI_API_KEY").unwrap_or_else(|_| String::from("")),
            openweather_api_key: env::var("OPENWEATHER_API_KEY").unwrap_or_else(|_| String::from("")),
            api_ninjas_key: env::var("API_NINJAS_KEY").unwrap_or_else(|_| String::from("")),
            bfl_api_key: env::var("BFL_API_KEY").unwrap_or_else(|_| String::from("")),
        })
    }
}
