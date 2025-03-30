// main.rs
mod commands;
mod config;
mod error;
mod events;
mod utils;
mod emoji_reaction;
mod types;
mod services;
mod web;

use crate::config::Config;
use crate::services::database::DatabaseService;
use crate::services::{LoggingService, MetricsService, TaskManager, RateLimiter};
use crate::services::cache::CacheService;
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
    pub cache: Arc<CacheService>,
    pub api: Arc<crate::services::api::ApiService>,
    pub start_time: Arc<Instant>,
    pub task_manager: Arc<crate::services::task_manager::TaskManager>,
    pub rate_limiter: Arc<crate::services::rate_limiter::RateLimiter>,
}

async fn check_and_send_reminders(ctx: &serenity::Context, data: &Data) -> Result<(), Error> {
    let start_time = std::time::Instant::now();
    let mut reminders_sent = 0;
    
    // Use TimedOperation to measure database query duration
    let due_reminders = {
        let _timer = crate::services::TimedOperation::for_db_query(
            "get_due_reminders", 
            data.logging.clone()
        );
        data.database.get_due_reminders().await?
    };

    let reminder_count = due_reminders.len();
    
    if reminder_count > 0 {
        data.logging.log_command_execution(
            "check_reminders", 
            None, 
            None
        );
        tracing::info!("Processing {} due reminders", reminder_count);
    }

    for reminder in due_reminders {
        let channel = ChannelId::new(reminder.channel_id as u64);
        let content = reminder.message.clone();

        match channel.send_message(&ctx.http, CreateMessage::new().content(&content)).await {
            Ok(_) => {
                data.database.update_reminder_last_sent(reminder.id).await?;
                reminders_sent += 1;
            },
            Err(e) => {
                tracing::warn!("Failed to send reminder to channel {}: {}", 
                     reminder.channel_id, e);
            }
        }
    }
    
    // Log metrics
    let elapsed = start_time.elapsed();
    if reminders_sent > 0 || reminder_count > 0 {
        tracing::info!("Reminder check completed: {}/{} reminders sent in {:?}", 
             reminders_sent, reminder_count, elapsed);
    } else {
        tracing::debug!("Reminder check completed: No due reminders found ({:?})", elapsed);
    }

    Ok(())
}

// Helper function to check and send reminders using just the HTTP client
async fn check_and_send_reminders_http(http: &serenity::Http, data: &Data) -> Result<(), Error> {
    let start_time = std::time::Instant::now();
    let mut reminders_sent = 0;
    
    // Use TimedOperation to measure database query duration
    let due_reminders = {
        let _timer = crate::services::TimedOperation::for_db_query(
            "get_due_reminders", 
            data.logging.clone()
        );
        data.database.get_due_reminders().await?
    };

    let reminder_count = due_reminders.len();
    
    if reminder_count > 0 {
        data.logging.log_command_execution(
            "check_reminders", 
            None, 
            None
        );
        tracing::info!("Processing {} due reminders", reminder_count);
    }

    for reminder in due_reminders {
        let channel = ChannelId::new(reminder.channel_id as u64);
        let content = reminder.message.clone();

        match channel.send_message(&http, CreateMessage::new().content(&content)).await {
            Ok(_) => {
                data.database.update_reminder_last_sent(reminder.id).await?;
                reminders_sent += 1;
            },
            Err(e) => {
                tracing::warn!("Failed to send reminder to channel {}: {}", 
                     reminder.channel_id, e);
            }
        }
    }
    
    // Log metrics
    let elapsed = start_time.elapsed();
    if reminders_sent > 0 || reminder_count > 0 {
        tracing::info!("Reminder check completed: {}/{} reminders sent in {:?}", 
             reminders_sent, reminder_count, elapsed);
    } else {
        tracing::debug!("Reminder check completed: No due reminders found ({:?})", elapsed);
    }

    Ok(())
}

async fn update_presence(ctx: serenity::Context, data: Arc<Data>) -> Result<(), Error> {
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
    
    // Initialize cache service
    let cache_service = Arc::new(CacheService::new(Arc::new(config.cache.clone()), metrics_service.clone()));
    CacheService::start_cleanup_task(cache_service.clone());
    info!("Cache service initialized with {} max entries", config.cache.max_size);
    
    // Initialize API service with cache
    let api_service = Arc::new(
        crate::services::api::ApiService::new(Arc::new(config.api.clone()))
            .with_cache(cache_service.clone())
    );
    info!("API service initialized with caching enabled");
    
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
    
    // Prepare database statements after migrations are applied
    let mut database_mut = database.clone();
    if let Err(e) = database_mut.prepare_statements().await {
        error!("Failed to prepare database statements: {}", e);
        return Err(e);
    }
    // Use the prepared version for all further operations
    let database = database_mut;
    
    let start_time = Arc::new(Instant::now());

    // Initialize task manager
    let task_manager = Arc::new(crate::services::TaskManager::new());
    info!("Task manager initialized");
    
    // Initialize rate limiter
    let rate_limiter = Arc::new(crate::services::RateLimiter::new());
    info!("Rate limiter initialized");
    
    // Create shared app data wrapped in Arc
    let app_data = Arc::new(Data {
        config: Arc::new(config.clone()),
        database: database.clone(),
        logging: logging_service.clone(),
        metrics: metrics_service.clone(),
        cache: cache_service.clone(),
        api: api_service.clone(),
        start_time: start_time.clone(),
        task_manager: task_manager.clone(),
        rate_limiter: rate_limiter.clone(),
    });
    
    // Set up global data access
    let _ = crate::types::DATA.set(app_data.clone());
    info!("Global data reference initialized");
    
    // Start Discord bot in a background task
    let bot_app_data = app_data.clone();
    tokio::task::spawn(async move {
        info!("Starting Discord bot in background task");
        if let Err(e) = start_discord_bot(bot_app_data).await {
            error!("Discord bot error: {}", e);
        }
    });
    
    // Initialize web module
    web::init().await;
    
    // Default web server port
    let web_port = std::env::var("WEB_SERVER_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);
    
    // Start the web server in the main thread
    info!("Starting web server on port {}", web_port);
    web::start_server(app_data.clone(), web_port).await?;
    
    // We'll only get here if the web server stops normally
    info!("Web server stopped normally, shutting down application");
    
    Ok(())
}

/// Start the Discord bot
async fn start_discord_bot(app_data: Arc<Data>) -> Result<(), Error> {
    info!("Starting Discord bot");
    
    let config_clone = app_data.config.as_ref().clone();
    let app_data_for_setup = app_data.clone();
    // Create a cloned Data for framework setup
    let data_for_framework = (*app_data).clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::get_commands(),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(config_clone.bot.command_prefix.clone()),
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
            let app_data_clone = app_data_for_setup.clone();
            
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                // Insert Data into TypeMap
                {
                    let mut data_map = ctx.data.write().await;
                    data_map.insert::<DataContainer>(app_data_clone);
                }

                // Return the Data instance, not the Arc<Data>
                Ok(data_for_framework)
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(
        &config_clone.bot.bot_token,
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
    
    // Create Arc references for use in the reminder checker task
    let http = client.http.clone();
    let task_data = app_data.clone();
    
    // Register the reminder checker task with high priority
    info!("Scheduling reminder checker task");
    if let Err(e) = app_data.task_manager.spawn_task_with_priority(
        "reminder_checker",
        crate::services::task_manager::TaskPriority::High,
        async move {
            info!("Reminder checker task started");
            let check_interval = Duration::from_secs(60); // Check every minute
            let mut interval = tokio::time::interval(check_interval);
            
            // For tracking consecutive errors
            let mut consecutive_errors = 0;
            const MAX_CONSECUTIVE_ERRORS: u32 = 5;
            
            loop {
                interval.tick().await;
                debug!("Checking for due reminders");
                
                match check_and_send_reminders_http(&http, &task_data).await {
                    Ok(_) => {
                        // Reset error counter on success
                        if consecutive_errors > 0 {
                            consecutive_errors = 0;
                            info!("Reminder checker recovered after previous errors");
                        }
                    },
                    Err(e) => {
                        consecutive_errors += 1;
                        error!("Error checking reminders (attempt {}): {}", 
                            consecutive_errors, e);
                        
                        // If we've had too many consecutive errors, back off temporarily
                        if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                            warn!("Too many consecutive errors, backing off for 5 minutes");
                            tokio::time::sleep(Duration::from_secs(300)).await;
                            consecutive_errors = 0;
                        }
                    }
                }
            }
        }
    ).await {
        error!("Failed to schedule reminder checker task: {}", e);
    } else {
        info!("Reminder checker task scheduled successfully");
    }

    client.start_autosharded().await.map_err(Error::from)
}
