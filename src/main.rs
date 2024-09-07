// main.rs
mod commands;
mod config;
mod database;
mod error;
mod events;
mod utils;
mod emoji_reaction;

use crate::config::Config;
use crate::database::Database;
use crate::error::Error;
use poise::serenity_prelude as serenity;
use serenity::GatewayIntents;
use tracing::Level;

struct Data {
    config: Config,
    database: Database,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let config = Config::load().await?;
    let database = Database::connect(&config.database_url).await?;

    let config_clone = config.clone(); // Clone config here

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
                Ok(Data {
                    config: config_clone, // Use the cloned config
                    database,
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(&config.bot_token, GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT)
        .framework(framework)
        .await?;

    client.cache.set_max_messages(1000);

    client.start_autosharded().await.map_err(Error::from)
}