use poise::serenity_prelude;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Serenity error: {0}")]
    Serenity(#[from] serenity_prelude::Error),

    #[error("Database error: {0}")]
    Database(#[from] tokio_postgres::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] toml::de::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("OpenAI error: {0}")]
    OpenAI(#[from] async_openai::error::OpenAIError),

    #[error("Chrono parse error: {0}")]
    ChronoParse(#[from] chrono::ParseError),

    #[error("Timezone parse error: {0}")]
    TimezoneParse(#[from] chrono_tz::ParseError),

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}