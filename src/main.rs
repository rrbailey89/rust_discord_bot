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
use crate::services::{LoggingService, MetricsService};
use crate::services::cache::CacheService;
use crate::error::Error;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{ChannelId, CreateMessage, OnlineStatus, ActivityData, Command as SerenityCommand}; // Added SerenityCommand
use serenity::GatewayIntents;
use std::sync::Arc;
use tokio::time::{Duration, sleep};
use tracing::{info, warn, error, debug};
use crate::types::ShardManagerContainer;
use crate::types::DataContainer;
use rand::{Rng, thread_rng}; // Use thread_rng directly
use std::time::Instant;
use std::path::Path;
use crate::web::services::AnalyticsService;
use crate::web::models::analytics::LogEventRequest;
use std::collections::{HashMap, HashSet}; // Added HashSet
use poise::Command as PoiseCommand; // Alias Poise Command
use serenity::builder::CreateCommand; // Added CreateCommand

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
    pub command_registry: Arc<crate::services::command_registry::CommandRegistryService>,
    pub command_cooldown: Arc<crate::services::command_cooldown::CommandCooldownService>,
}

// Consolidated function to send reminders
async fn send_due_reminders_internal(http: &serenity::Http, data: &Data) -> Result<(), Error> {
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
        // Prefix unused result
        let _ = data.logging.log_command_execution(
            "check_reminders", // Consider renaming this log identifier if needed
            None,
            None
        );
        tracing::info!("Processing {} due reminders", reminder_count);
    }

    for reminder in due_reminders {
        let channel = ChannelId::new(reminder.channel_id as u64);
        let content = reminder.message.clone();

        match channel.send_message(http, CreateMessage::new().content(&content)).await { // Use the passed http client
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

    // Synchronize command names with Discord
    info!("Synchronizing command names with Discord...");
    if let (Some(token), Some(app_id)) = (
        Some(config.bot.bot_token.clone()),
        config.bot.application_id.clone()
    ) {
        let command_service = web::services::command::CommandService::new(database.clone());
        let discord_service = web::services::discord::DiscordService::new_bot(
            token,
            Some(app_id)
        );
        let command_service_with_discord = command_service.with_discord_service(discord_service);

        // Run the sync operation
        if let Err(e) = command_service_with_discord.sync_command_names_from_discord().await {
            error!("Failed to sync command names from Discord: {}", e);
        } else {
            info!("Successfully synchronized command names with Discord");
        }
    }

    let start_time = Arc::new(Instant::now());

    // Initialize task manager
    let task_manager = Arc::new(crate::services::TaskManager::new());
    info!("Task manager initialized");

    // Initialize rate limiter
    let rate_limiter = Arc::new(crate::services::RateLimiter::new());
    info!("Rate limiter initialized");

    // Initialize command registry service
    let command_registry = Arc::new(
        crate::services::command_registry::CommandRegistryService::new(
            database.clone(),
            Arc::new(serenity::Http::new(&config.bot.bot_token.clone())),
        )
    );
    info!("Command registry service initialized");

    // Initialize command cooldown service
    let command_cooldown = Arc::new(
        crate::services::command_cooldown::CommandCooldownService::new(
            database.clone(),
        )
    );
    info!("Command cooldown service initialized");

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
        command_registry: command_registry.clone(),
        command_cooldown: command_cooldown.clone(),
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
    // Clone the bot token separately to avoid move issues later
    let bot_token_clone = config_clone.bot.bot_token.clone();
    let app_data_for_setup = app_data.clone();
    // Create a cloned Data for framework setup
    let data_for_framework = (*app_data).clone();

    // We'll use the existing registry and cooldown services from app_data
    let _command_registry = app_data.command_registry.clone(); // Prefix unused
    let _command_cooldown = app_data.command_cooldown.clone(); // Prefix unused

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
            on_error: |error| {
                Box::pin(async move {
                    tracing::error!("Command error: {}", error);
                    if let poise::FrameworkError::Command { error, ctx, .. } = error {
                        let _ = ctx.say(format!("Error running command: {}", error)).await;
                    }
                })
            },
            pre_command: |ctx| {
                Box::pin(async move {
                    let command_name = ctx.command().qualified_name.clone();
                    let user_id = ctx.author().id.get() as i64;
                    let data = ctx.data(); // Get data once at the start

                    // Only apply cooldowns for guild commands
                    if let Some(guild_id) = ctx.guild_id() {
                        let guild_id = guild_id.get() as i64;

                        // Check if the command is on cooldown
                        if let Ok(Some(remaining)) = data.command_cooldown.is_on_cooldown(guild_id, &command_name, user_id).await {
                            // Command is on cooldown, respond to the user
                            let seconds = remaining.as_secs();
                            let _ = ctx.say(format!("This command is on cooldown. Please wait {} more second{} before using it again.",
                                seconds, if seconds == 1 { "" } else { "s" })).await;

                            // Return early to prevent execution
                            return;
                        }

                        // Command is not on cooldown, record this usage
                        // This method returns () (unit), not a Result, so don't try to handle errors
                        data.command_cooldown.record_command_usage(guild_id, &command_name, user_id).await;

                        // Log analytics event for command usage
                        let analytics_service = AnalyticsService::new(data.database.clone());
                        let mut event_data = HashMap::new();
                        event_data.insert("command_name".to_string(), serde_json::Value::String(command_name.clone()));

                        let log_request = LogEventRequest {
                            event_type: "command_used".to_string(),
                            user_id: Some(user_id),
                            guild_id: Some(guild_id),
                            event_data: Some(event_data),
                        };

                        // Spawn a task to log the event without blocking the command
                        tokio::spawn(async move {
                            if let Err(e) = analytics_service.log_event(&log_request).await {
                                error!("Failed to log command_used analytics event: {}", e);
                            }
                         });
                     } else {
                          // Log global command usage (no guild_id)
                         // Use the 'data' variable obtained at the start of the hook
                         let analytics_service = AnalyticsService::new(data.database.clone());
                         let mut event_data = HashMap::new();
                         event_data.insert("command_name".to_string(), serde_json::Value::String(command_name.clone()));

                         let log_request = LogEventRequest {
                             event_type: "command_used".to_string(),
                             user_id: Some(user_id),
                             guild_id: None, // No guild ID for global commands
                             event_data: Some(event_data),
                         };

                         tokio::spawn(async move {
                             if let Err(e) = analytics_service.log_event(&log_request).await {
                                 error!("Failed to log global command_used analytics event: {}", e);
                             }
                          });
                     }
                     // No explicit return needed as the function returns ()
                 })
             },
            event_handler: |ctx, event, framework, data| {
                Box::pin(async move {
                    // Handle the Result returned by the event handler
                    if let Err(e) = events::handle_event(ctx, event, framework, data).await {
                        error!("Error in event handler: {}", e);
                    }
                    Ok(()) // Add Ok(()) to match the expected return type
                })
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, _framework| {
            let app_data_clone = app_data_for_setup.clone();

            Box::pin(async move {
                info!("Starting global command registration check...");

                // 1. Get local command definitions intended to be global
                let all_local_commands: Vec<PoiseCommand<Data, Error>> = commands::get_commands();
                let command_configs = commands::get_command_config();
                let local_global_commands: Vec<PoiseCommand<Data, Error>> = all_local_commands.into_iter()
                    .filter(|cmd| {
                        command_configs.get(cmd.name.as_str())
                            .map_or(false, |config| config.scope == commands::CommandScope::Global)
                    })
                    .collect();
                let local_global_command_names: HashSet<&str> = local_global_commands.iter().map(|cmd| cmd.name.as_str()).collect();
                info!("Found {} local commands intended to be global: {:?}", local_global_command_names.len(), local_global_command_names);

                // 2. Fetch existing global commands from Discord
                let existing_commands = match ctx.http.get_global_commands().await {
                    Ok(cmds) => {
                        info!("Fetched {} existing global commands from Discord.", cmds.len());
                        cmds
                    },
                    Err(e) => {
                        error!("Failed to fetch existing global commands: {}", e);
                        // Proceed without comparison, potentially re-registering everything
                        // Or return error depending on desired behavior
                        return Err(e.into()); // Return error for now
                    }
                };
                let existing_commands_map: HashMap<String, SerenityCommand> = existing_commands.into_iter()
                    .map(|cmd| (cmd.name.clone(), cmd))
                    .collect();

                // 3. Compare and determine actions
                let mut commands_to_create: Vec<CreateCommand> = Vec::new();
                // Correct tuple definition: (ID, Builder, Name)
                let mut commands_to_edit: Vec<(serenity::CommandId, CreateCommand, String)> = Vec::new();
                let mut command_ids_to_delete: Vec<serenity::CommandId> = Vec::new();

                // Check local commands against existing ones
                for local_cmd in &local_global_commands {
                    let cmd_name = local_cmd.name.clone(); // Clone name for logging
                    // Handle Option returned by create_as_slash_command
                    if let Some(local_builder) = local_cmd.create_as_slash_command() {
                        if let Some(existing_cmd) = existing_commands_map.get(&cmd_name) {
                            // Command exists, check if it needs update
                            // Pass local_cmd too for comparison data
                            if builders_differ(local_cmd, &local_builder, existing_cmd) {
                                info!("Command '{}' definition differs, scheduling edit.", cmd_name);
                                // Store name along for logging in the edit loop
                                commands_to_edit.push((existing_cmd.id, local_builder, cmd_name.clone()));
                            } else {
                                info!("Command '{}' definition matches, skipping.", cmd_name);
                            }
                        } else {
                            // Command doesn't exist, needs creation
                            info!("Command '{}' not found on Discord, scheduling creation.", cmd_name);
                            commands_to_create.push(local_builder); // Push the builder itself
                        }
                    } else {
                        warn!("Could not create slash command builder for local command '{}'. Skipping.", local_cmd.name);
                    }
                }

                // Check existing commands against local ones for deletion
                for (name, existing_cmd) in &existing_commands_map {
                    if !local_global_command_names.contains(name.as_str()) {
                        info!("Existing command '{}' not found locally, scheduling deletion.", name);
                        command_ids_to_delete.push(existing_cmd.id);
                    }
                }

                // 4. Execute API calls
                let mut changes_made = false;

                if !commands_to_create.is_empty() {
                    info!("Creating {} new global command(s)...", commands_to_create.len());
                    for builder in commands_to_create { // Takes ownership
                        // Log using the name from the builder if possible (might require storing name separately)
                        // For now, use a generic log message
                        if let Err(e) = ctx.http.create_global_command(&builder).await { // Pass by reference
                            error!("Failed to create a new global command: {}", e);
                        } else {
                            info!("Successfully created a new global command.");
                            changes_made = true;
                        }
                    }
                }

                if !commands_to_edit.is_empty() {
                    info!("Editing {} existing global command(s)...", commands_to_edit.len());
                    // Correctly destructure the tuple (CommandId, CreateCommand, String)
                    for (id, builder, name) in commands_to_edit { // Takes ownership
                        if let Err(e) = ctx.http.edit_global_command(id, &builder).await { // Pass builder by reference
                            error!("Failed to edit global command '{}' (ID {}): {}", name, id, e);
                        } else {
                            info!("Successfully edited global command '{}' (ID {})", name, id);
                            changes_made = true;
                        }
                    }
                }

                if !command_ids_to_delete.is_empty() {
                    info!("Deleting {} obsolete global command(s)...", command_ids_to_delete.len());
                    for id in command_ids_to_delete {
                        if let Err(e) = ctx.http.delete_global_command(id).await {
                            error!("Failed to delete global command ID {}: {}", id, e);
                        } else {
                            info!("Successfully deleted global command ID {}", id);
                            changes_made = true;
                        }
                    }
                }

                if !changes_made {
                    info!("No changes needed for global commands.");
                }

                // Remove the old unconditional registration
                // poise::builtins::register_globally(ctx, &global_commands).await?;
                // info!("Registered {} global commands with Discord (ping, help)", global_commands.len());

                // Insert Data into TypeMap
                {
                    let mut data_map = ctx.data.write().await;
                    data_map.insert::<DataContainer>(app_data_clone);
                }

                Ok(data_for_framework)
            })
        })
        .build();

    // Helper function to compare local PoiseCommand with existing SerenityCommand
    fn builders_differ(
        local_cmd: &PoiseCommand<Data, Error>,
        _builder: &CreateCommand, // Keep reference, maybe needed later for more complex checks
        existing: &SerenityCommand
    ) -> bool {
        // 1. Compare Name (Redundant check, but safe)
        if local_cmd.name != existing.name {
            warn!("Name mismatch during diff check? Local: '{}', Existing: '{}'", local_cmd.name, existing.name);
            return true;
        }

        // 2. Compare Description (Handle Option<String> vs String)
        let local_desc = local_cmd.description.as_deref();
        let existing_desc = Some(existing.description.as_str()).filter(|s| !s.is_empty()); // Treat empty string as None
        if local_desc != existing_desc {
            // Consider localized description if primary differs
            if existing.description_localizations.as_ref().and_then(|loc| loc.get("en-US").map(|s| s.as_str())) != local_desc {
                 return true;
            }
        }

        // 3. Compare Options Length (Basic check - poise::Command uses `parameters`)
        if local_cmd.parameters.len() != existing.options.len() {
            return true;
        }
        // TODO: Implement more robust option comparison using local_cmd.parameters vs existing.options

        // 4. Compare Default Member Permissions (poise::Command uses `required_permissions`)
        // Handle Option vs non-Option: If existing is None, local must be empty. If existing is Some, they must match.
        match existing.default_member_permissions {
            None => {
                if !local_cmd.required_permissions.is_empty() { return true; }
            }
            Some(existing_perms) => {
                if local_cmd.required_permissions != existing_perms { return true; }
            }
        }


        // 5. Compare NSFW status (poise::Command uses `nsfw_only`)
        if local_cmd.nsfw_only != existing.nsfw {
            return true;
        }

        // TODO: Compare contexts, integration_types, dm_permission using local_cmd fields

        false // Assume same if basic checks pass
    }

    let mut client = serenity::ClientBuilder::new(
        &bot_token_clone,
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

                // Call the consolidated internal function
                match send_due_reminders_internal(&http, &task_data).await {
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
