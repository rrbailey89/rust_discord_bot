// main.rs
mod commands;
mod config;
mod database;
mod error;
mod events;
mod utils;

use crate::config::Config;
use crate::database::Database;
use crate::error::Error;
use poise::serenity_prelude as serenity;
use serenity::GatewayIntents;

struct Data {
    config: Config,
    database: Database,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let config = Config::load().await?;
    let database = Database::connect(&config.database_url).await?;

    let config_clone = config.clone();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::get_commands(),
            event_handler: |ctx, event, framework, data| {
                Box::pin(events::handle_event(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { config, database })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(&config.bot_token, GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT)
        .framework(framework)
        .await?;

    client.start().await.map_err(Error::from)
}