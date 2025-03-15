// config/database.rs
use serde::Deserialize;
use std::env;
use crate::error::Error;

#[derive(Clone, Deserialize, Debug)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: usize,
    pub statement_timeout_ms: Option<u64>,
}

impl DatabaseConfig {
    pub fn load() -> Result<Self, Error> {
        Ok(Self {
            url: env::var("DATABASE_URL").unwrap_or_else(|_| String::from("")),
            max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| String::from("10"))
                .parse()
                .unwrap_or(10),
            statement_timeout_ms: env::var("DATABASE_STATEMENT_TIMEOUT_MS")
                .ok()
                .and_then(|v| v.parse().ok()),
        })
    }
}
