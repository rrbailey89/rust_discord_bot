// main.rs
mod commands;
mod config;
mod database;
mod error;
mod events;
mod utils;
mod emoji_reaction;
mod types;

use std::sync::Arc;
use chrono::Utc;
use crate::config::Config;
use crate::database::Database;
use crate::error::Error;
use poise::serenity_prelude as serenity;
use serenity::GatewayIntents;
use tracing::Level;
use tokio::time::{interval, Duration};
use poise::serenity_prelude::{ChannelId, CreateMessage};
use tracing::{info, debug, error};
use chrono_tz::America::Los_Angeles;

#[derive(Clone)]
pub struct Data {
    pub config: Arc<Config>,
    pub database: Database,
}

async fn check_and_send_reminders(ctx: &serenity::Context, data: &Data) -> Result<(), Error> {
    info!("Starting check_and_send_reminders");

    let now = Utc::now().with_timezone(&Los_Angeles);
    info!("Current time (Los Angeles): {}", now.format("%Y-%m-%d %H:%M:%S %Z"));

    let due_reminders = data.database.get_due_reminders().await?;
    debug!("Found {} due reminders", due_reminders.len());

    for reminder in due_reminders {
        debug!("Processing reminder: id={}, channel_id={}, message={:?}",
               reminder.id, reminder.channel_id, reminder.message);

        let channel = ChannelId::new(reminder.channel_id as u64);
        let content = reminder.message.clone();

        debug!("Attempting to send reminder to channel {}", channel);

        match channel.send_message(&ctx.http, CreateMessage::new().content(&content)).await {
            Ok(_) => {
                info!("Successfully sent reminder: id={}", reminder.id);

                debug!("Updating last_sent time for reminder id={}", reminder.id);
                match data.database.update_reminder_last_sent(reminder.id).await {
                    Ok(_) => debug!("Successfully updated last_sent time for reminder id={}", reminder.id),
                    Err(e) => error!("Failed to update last_sent time for reminder id={}: {:?}", reminder.id, e),
                }
            },
            Err(e) => error!("Failed to send reminder id={}: {:?}", reminder.id, e),
        }
    }

    info!("Finished check_and_send_reminders");
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
                    let mut interval = interval(Duration::from_secs(15)); // Check every minute
                    loop {
                        interval.tick().await;
                        debug!("Tick: About to check and send reminders");
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