// main.rs
mod commands;
mod config;
mod database; // Keep for now until migration is complete
mod error;
mod events;
mod utils;
mod emoji_reaction;
mod types;
mod services;

use crate::config::Config;
use crate::services::database::DatabaseService;
use crate::services::{LoggingService, MetricsService};
use crate::error::Error;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{ChannelId, CreateMessage, OnlineStatus, ActivityData};
use serenity::GatewayIntents;
use std::sync::Arc;
use tokio::time::{interval, Duration, sleep};
use tracing::{info, warn, error, debug};
use crate::types::ShardManagerContainer;
use crate::types::DataContainer;
use rand::Rng;
use std::time::Instant;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct Data {
    pub config: Arc<Config>,
    pub database: DatabaseService,
    pub logging: Arc<LoggingService>,
    pub metrics: Arc<MetricsService>,
    pub start_time: Arc<Instant>,
}

async fn check_and_send_reminders(ctx: &serenity::Context, data: &Data) -> Result<(), Error> {
    // Use TimedOperation to measure database query duration
    let due_reminders = {
        let _timer = crate::services::TimedOperation::for_db_query(
            "get_due_reminders", 
            data.logging.clone()
        );
        data.database.get_due_reminders().await?
    };

    if !due_reminders.is_empty() {
        data.logging.log_command_execution(
            "check_reminders", 
            None, 
            None
        );
        tracing::info!("Processing {} due reminders", due_reminders.len());
    }

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
    // Log that we're updating presence
    data.logging.log_command_execution("update_presence", None, None);
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
    // Load configuration first
    let config = Config::load().await?;
    
    // Initialize logging system
    let logging_service = Arc::new(LoggingService::new(config.logging.clone()));
    if let Err(e) = logging_service.init() {
        eprintln!("Failed to initialize logging: {}", e);
        return Err(e);
    }
    
    // Log startup information
    info!("Bot starting up");
    info!(version = env!("CARGO_PKG_VERSION"), "Version");
    
    // Initialize database
    let database = DatabaseService::new(&config.database).await?;
    
    // Initialize metrics service
    let metrics_service = Arc::new(MetricsService::new(logging_service.clone()));
    
    // Start metrics logging in the background (every 5 minutes)
    metrics_service.start_metrics_logger(Duration::from_secs(300));
    
    // Run database migrations
    let migrations_path = Path::new("migrations");
    match database.run_migrations(migrations_path).await {
        Ok(count) => {
            info!("Applied {} database migrations", count);
        },
        Err(e) => {
            error!("Failed to run database migrations: {}", e);
            return Err(e);
        }
    }
    
    // Get current schema version
    if let Ok(Some(version)) = database.get_schema_version().await {
        info!("Current database schema version: {}", version);
    } else {
        warn!("Could not determine database schema version");
    }
    
    let start_time = Arc::new(Instant::now());

    let config_clone = config.clone();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::get_commands(),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(config.bot.command_prefix.clone()),
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
                    logging: logging_service.clone(),
                    metrics: metrics_service.clone(),
                    start_time: start_time.clone(),
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
                let data_for_health_check = data.clone();

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
                
                // Spawn the database health check task
                tokio::spawn(async move {
                    let mut interval = interval(Duration::from_secs(30));
                    loop {
                        interval.tick().await;
                        
                        // Time the database health check
                        let health_result = {
                            let _timer = crate::services::TimedOperation::for_db_query(
                                "health_check", 
                                data_for_health_check.logging.clone()
                            );
                            data_for_health_check.database.check_health().await
                        };
                        
                        match health_result {
                            Ok(status) => {
                                // Record metrics
                                data_for_health_check.metrics.record(
                                    &crate::services::format_metric_name(
                                        crate::services::MetricType::DatabaseQuery, 
                                        "pool_size"
                                    ), 
                                    status.size as u64
                                );
                                data_for_health_check.metrics.record(
                                    &crate::services::format_metric_name(
                                        crate::services::MetricType::DatabaseQuery, 
                                        "pool_available"
                                    ), 
                                    status.available as u64
                                );
                                
                                // Log info
                                info!(
                                    size = status.size,
                                    max_size = status.max_size,
                                    available = status.available,
                                    waiting = status.waiting,
                                    "DB Pool Status"
                                );
                                
                                // Log a warning if available connections are low
                                if status.available < 3 && status.waiting > 0 {
                                    warn!(
                                        available = status.available,
                                        waiting = status.waiting,
                                        "Database connection pool pressure"
                                    );
                                    
                                    // Record pressure metric
                                    data_for_health_check.metrics.record(
                                        &crate::services::format_metric_name(
                                            crate::services::MetricType::DatabaseQuery, 
                                            "pool_pressure"
                                        ), 
                                        1
                                    );
                                }
                            },
                            Err(e) => {
                                error!("Database health check failed: {:?}", e);
                                data_for_health_check.logging.log_error_occurrence("database_health_check_failure");
                            }
                        }
                    }
                });

                Ok(data)
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(
        &config.bot.bot_token,
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
