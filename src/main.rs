// main.rs
mod commands;
mod config;
mod database;
mod error;
mod events;
mod utils;
mod emoji_reaction;
mod types;

use crate::config::Config;
use crate::database::Database;
use crate::error::Error;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{ChannelId, CreateMessage};
use serenity::GatewayIntents;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::Level;

#[derive(Clone)]
pub struct Data {
    pub config: Arc<Config>,
    pub database: Database,
}

async fn check_and_send_reminders(ctx: &serenity::Context, data: &Data) -> Result<(), Error> {
    let due_reminders = data.database.get_due_reminders().await?;

    for reminder in due_reminders {
        let channel = ChannelId::new(reminder.channel_id as u64);
        let content = reminder.message.clone();

        if let Ok(_) = channel.send_message(&ctx.http, CreateMessage::new().content(&content)).await {
            data.database.update_reminder_last_sent(reminder.id).await?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

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

                let ctx_clone = ctx.clone();
                let data_clone = Data {
                    config: config_clone.clone().into(),
                    database: database.clone(),
                };

                tokio::spawn(async move {
                    let mut interval = interval(Duration::from_secs(15));
                    loop {
                        interval.tick().await;
                        if let Err(e) = check_and_send_reminders(&ctx_clone, &data_clone).await {
                            eprintln!("Error sending reminders: {:?}", e);
                        }
                    }
                });

                Ok(Data {
                    config: config_clone.into(),
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