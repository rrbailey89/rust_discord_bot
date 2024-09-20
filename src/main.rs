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
use poise::serenity_prelude::{ChannelId, CreateMessage, OnlineStatus, ActivityData};
use serenity::GatewayIntents;
use std::sync::Arc;
use tokio::time::{interval, Duration, sleep};
use tracing::Level;
use crate::types::ShardManagerContainer;
use crate::types::DataContainer;
use rand::Rng;

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

async fn update_presence(ctx: serenity::Context, data: Data) -> Result<(), Error> {

    loop {
        let activity = ActivityData::custom("Use /help to learn more");
        ctx.set_presence(Some(activity), OnlineStatus::Online);

        let sleep_duration = {
            let mut rng = rand::thread_rng();
            Duration::from_secs(rng.gen_range(600..=900))
        };
        sleep(sleep_duration).await;

        let blame_count = data.database.get_blame_count().await?;
        let activity = ActivityData::custom(format!("Serena's blame count: {}", blame_count));
        ctx.set_presence(Some(activity), OnlineStatus::Online);

        let sleep_duration = {
            let mut rng = rand::thread_rng();
            Duration::from_secs(rng.gen_range(600..=900))
        };
        sleep(sleep_duration).await;
    }
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
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(config.command_prefix.clone()),
                edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                    Duration::from_secs(3600)
                ))),
                case_insensitive_commands: true,
                ..Default::default()
            },
            event_handler: |ctx, event, framework, data| {
                Box::pin(events::handle_event(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            let config_clone = config_clone.clone();
            let database = database.clone();

            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                let data = Data {
                    config: Arc::new(config_clone),
                    database: database.clone(),
                };

                // Insert Data into TypeMap
                {
                    let mut data_map = ctx.data.write().await;
                    data_map.insert::<DataContainer>(data.clone());
                }

                // Clone Context and Data for the spawned tasks
                let ctx_for_reminder = ctx.clone();
                let ctx_for_presence = ctx.clone();
                let data_for_reminder = data.clone();
                let data_for_presence = data.clone();

                tokio::spawn(async move {
                    let mut interval = interval(Duration::from_secs(15));
                    loop {
                        interval.tick().await;
                        if let Err(e) = check_and_send_reminders(&ctx_for_reminder, &data_for_reminder).await {
                            eprintln!("Error sending reminders: {:?}", e);
                        }
                    }
                });

                // Spawn the presence update task
                tokio::spawn(async move {
                    if let Err(e) = update_presence(ctx_for_presence, data_for_presence).await {
                        eprintln!("Error updating presence: {:?}", e);
                    }
                });

                Ok(data)
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(
        &config.bot_token,
        GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT,
    )
        .framework(framework)
        .await?;

    {
        let mut data_map = client.data.write().await;
        data_map.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        // Data is inserted via the setup closure
    }

    client.cache.set_max_messages(1000);

    client.start_autosharded().await.map_err(Error::from)
}