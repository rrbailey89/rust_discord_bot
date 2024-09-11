// config.rs
use serde::Deserialize;
use tokio::io::AsyncReadExt;


#[derive(Clone, Deserialize)]
pub struct Config {
    pub bot_token: String,
    pub database_url: String,
    pub openai_api_key: String,
    pub openweather_api_key: String,
}


impl Config {
    pub async fn load() -> Result<Self, crate::error::Error> {
        let mut file = tokio::fs::File::open("config.toml").await?;
        let mut contents = String::new();
        AsyncReadExt::read_to_string(&mut file, &mut contents).await?;
        toml::from_str(&contents).map_err(Into::into)
    }
}