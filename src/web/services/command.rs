// src/web/services/command.rs
//! Command service for database operations related to commands

use std::collections::HashMap;
use crate::error::Error;
use crate::services::database::DatabaseService;
use tracing::{debug, info, warn, error};
use crate::web::models::command::{
    CommandInfo, CommandDetails, CommandSettings, UpdateCommandSettingsRequest,
    ConfigSchema, ConfigOption, EnumValue
};
use serde_json::Value;

use std::sync::Arc;
use crate::web::services::discord::{DiscordService, DiscordApplicationCommand, DiscordApplicationCommandOption, DiscordApplicationCommandOptionChoice};

/// Command service for database operations
pub struct CommandService {
    /// Database service
    db: DatabaseService,
    /// Discord service (optional, for command sync)
    discord: Option<DiscordService>,
    /// Cache service (optional for caching operations)
    cache: Option<Arc<crate::services::cache::CacheService>>,
}

impl CommandService {
    /// Create a new command service
    pub fn new(db: DatabaseService) -> Self {
        Self { 
            db,
            discord: None,
            cache: None,
        }
    }
    
    /// Create a new command service with cache
    pub fn new_with_cache(db: DatabaseService, cache: Arc<crate::services::cache::CacheService>) -> Self {
        Self { 
            db,
            discord: None,
            cache: Some(cache),
        }
    }
    
    /// Set Discord service for command synchronization
    pub fn with_discord_service(mut self, discord: DiscordService) -> Self {
        self.discord = Some(discord);
        self
    }

    /// Sync command names from Discord API to database
    pub async fn sync_command_names_from_discord(&self) -> Result<(), Error> {
        // Check if Discord service is available
        let discord = match &self.discord {
            Some(service) => service,
            None => {
                debug!("No Discord service configured, skipping command name sync");
                return Ok(());
            }
        };
        
        info!("Starting command name synchronization from Discord API");
        
        // Get client for database operations
        let client = self.db.get_client().await?;
        
        // Retrieve all available commands from our database first
        let our_commands = self.get_available_commands().await?;
        
        // Create a map of our command IDs to command info
        let mut command_map: HashMap<String, &CommandInfo> = HashMap::new();
        for cmd in &our_commands {
            command_map.insert(cmd.id.clone(), cmd);
        }
        
        // Optionally retrieve global commands
        let global_commands = match discord.get_global_commands().await {
            Ok(cmds) => {
                info!("Retrieved {} global commands from Discord", cmds.len());
                cmds
            },
            Err(e) => {
                warn!("Failed to retrieve global commands: {}", e);
                Vec::new()
            }
        };
        
        // Track which command names we've updated
        let mut updated_commands = 0;
        
        // Process global commands first if any
        for cmd in &global_commands {
            // Find a match in our commands by comparing names case-insensitively
            let mut matched_id = None;
            
            for our_cmd in &our_commands {
                if our_cmd.name.to_lowercase() == cmd.name.to_lowercase() {
                    matched_id = Some(our_cmd.id.clone());
                    break;
                }
            }
            
            if let Some(id) = matched_id {
                debug!("Updating command {} with Discord global name: {}", id, cmd.name);
                
                client
                    .execute(
                        "UPDATE commands SET discord_name = $1 WHERE command_id = $2",
                        &[&cmd.name, &id],
                    )
                    .await?;
                
                updated_commands += 1;
            }
        }
        
        // Get list of guilds the bot is in from database
        let guild_rows = client
            .query(
                "SELECT guild_id FROM guilds WHERE bot_joined = true",
                &[],
            )
            .await?;
        
        let guild_ids: Vec<i64> = guild_rows.iter().map(|row| row.get(0)).collect();
        
        // For each guild, get guild commands and update our database
        for guild_id in guild_ids {
            info!("Checking commands for guild {}", guild_id);
            
            let guild_commands = match discord.get_guild_commands(&guild_id.to_string()).await {
                Ok(cmds) => {
                    info!("Retrieved {} commands for guild {}", cmds.len(), guild_id);
                    cmds
                },
                Err(e) => {
                    warn!("Failed to retrieve commands for guild {}: {}", guild_id, e);
                    continue; // Skip to next guild
                }
            };
            
            // Process each guild command
            for cmd in guild_commands {
                // Skip empty names (shouldn't happen, but just to be safe)
                if cmd.name.is_empty() {
                    continue;
                }
                
                // Find a match in our commands by comparing names case-insensitively
                let mut matched_id = None;
                
                for our_cmd in &our_commands {
                    if our_cmd.name.to_lowercase() == cmd.name.to_lowercase() {
                        matched_id = Some(our_cmd.id.clone());
                        break;
                    }
                }
                
                if let Some(id) = matched_id {
                    debug!("Updating command {} with Discord guild name: {}", id, cmd.name);
                    
                    client
                        .execute(
                            "UPDATE commands SET discord_name = $1 WHERE command_id = $2",
                            &[&cmd.name, &id],
                        )
                        .await?;
                    
                    updated_commands += 1;
                }
            }
        }
        
        info!("Command name synchronization complete, updated {} commands", updated_commands);
        Ok(())
    }

    /// Get all available commands
    pub async fn get_available_commands(&self) -> Result<Vec<CommandInfo>, Error> {
        let client = self.db.get_client().await?;

        // Query the commands table to get all available commands
        let rows = client
            .query(
                "SELECT 
                    command_id, 
                    name, 
                    description, 
                    category, 
                    requires_admin,
                    coalesce(options IS NOT NULL, false) as has_config,
                    discord_name
                FROM commands
                ORDER BY category, name",
                &[],
            )
            .await?;

        // Map rows to CommandInfo objects
        let commands = rows
            .iter()
            .map(|row| CommandInfo {
                id: row.get(0),
                name: row.get(1),
                description: row.get(2),
                category: row.get(3),
                requires_admin: row.get(4),
                has_config: row.get(5),
                discord_name: row.get(6),
            })
            .collect();

        Ok(commands)
    }

    /// Get command settings for a guild
    pub async fn get_command_settings(
        &self,
        guild_id: i64,
        command_id: &str,
    ) -> Result<CommandSettings, Error> {
        let client = self.db.get_client().await?;

        // Query the guild_command_settings table
        let row = client
            .query_opt(
                "SELECT enabled, settings, discord_command_id
                FROM guild_command_settings
                WHERE guild_id = $1 AND command_id = $2",
                &[&guild_id, &command_id],
            )
            .await?;

        // If no settings found, return default settings (enabled = true, no custom settings)
        let (enabled, settings, discord_command_id) = match row {
            Some(r) => (r.get(0), r.get::<_, Option<Value>>(1), r.get::<_, Option<String>>(2)),
            None => (true, None, None),
        };

        Ok(CommandSettings {
            command_id: command_id.to_string(),
            enabled,
            settings: settings.map(|v| {
                serde_json::from_value(v).unwrap_or_else(|_| HashMap::new())
            }),
            discord_command_id,
        })
    }

    /// Update command settings for a guild
    pub async fn update_command_settings(
        &self,
        guild_id: i64,
        command_id: &str,
        request: &UpdateCommandSettingsRequest,
    ) -> Result<(), Error> {
        let client = self.db.get_client().await?;

        // Check if the command exists
        let command_row = client
            .query_opt(
                "SELECT name, description, discord_name FROM commands WHERE command_id = $1 LIMIT 1",
                &[&command_id],
            )
            .await?;

        let command_info = match command_row {
            Some(row) => {
                let name: String = row.get(0);
                let description: String = row.get(1);
                let discord_name: Option<String> = row.get(2);
                (name, description, discord_name)
            },
            None => return Err(Error::Unknown(format!("Command not found: {}", command_id))),
        };

        // If both enabled and settings are None, there's nothing to update
        if request.enabled.is_none() && request.settings.is_none() {
            return Ok(());
        }

        // Get current settings to merge with new ones if needed
        let current = self.get_command_settings(guild_id, command_id).await?;

        // Determine what to update
        let enabled = request.enabled.unwrap_or(current.enabled);
        
        let settings = match (&request.settings, &current.settings) {
            // If new settings provided, use them
            (Some(new_settings), _) => {
                let json = serde_json::to_value(new_settings)
                    .map_err(|e| Error::Unknown(format!("Failed to serialize settings: {}", e)))?;
                Some(json)
            },
            // Otherwise keep current settings
            (None, Some(_)) => {
                let json = serde_json::to_value(current.settings.unwrap())
                    .map_err(|e| Error::Unknown(format!("Failed to serialize current settings: {}", e)))?;
                Some(json)
            },
            // No settings at all
            (None, None) => None,
        };

        // Get cooldown setting from settings if it exists
        let cooldown_secs = if let Some(settings_value) = &settings {
            settings_value.get("cooldown")
                .and_then(|v| v.as_u64())
                .or_else(|| {
                    // Get from command config if not in settings
                    let configs = crate::commands::get_command_config();
                    configs
                        .get(command_id)
                        .and_then(|c| c.cooldown)
                })
        } else {
            // If no settings provided, get default from command config
            let configs = crate::commands::get_command_config();
            configs
                .get(command_id)
                .and_then(|c| c.cooldown)
        };

        // Check for Discord integration
        let discord_service = self.discord.as_ref();

        // Variable to hold new Discord command ID if we register one
        let mut new_discord_command_id: Option<String> = None;

        // If the enabled state changed and Discord integration is available, update Discord
        let enabled_changed = request.enabled.is_some() && request.enabled != Some(current.enabled);
        
        if enabled_changed && discord_service.is_some() {
            let discord = discord_service.unwrap();
            
            if enabled {
                // Command is being enabled - register it with Discord
                debug!("Registering command {} with Discord for guild {}", command_id, guild_id);
                
                // Use discord_name if available, otherwise use the command's name
                let cmd_name = command_info.2.unwrap_or_else(|| command_info.0.clone());
                
                // Create the command structure
                let command_details = self.get_command_details(guild_id, command_id).await?;
                
                // Convert the options recursively if the command has them
                let options = if let Some(config_schema) = &command_details.config_schema {
                    // Call the helper function correctly
                    convert_options_recursive(&config_schema.options)? 
                } else {
                    Vec::new()
                };
                
                // Create the command for Discord
                let app_command = crate::web::services::discord::DiscordApplicationCommand {
                    id: None,
                    name: cmd_name,
                    description: command_info.1.clone(),
                    options,
                };
                
                // Register with Discord
                match discord.add_guild_command(&guild_id.to_string(), app_command).await {
                    Ok(registered_command) => {
                        // Save the Discord assigned ID
                        if let Some(id) = registered_command.id {
                            new_discord_command_id = Some(id);
                            debug!("Command registered with Discord, assigned ID: {:?}", new_discord_command_id);
                        }
                    },
                    Err(e) => {
                        error!("Failed to register command with Discord: {}", e);
                        return Err(Error::Unknown(format!("Failed to register command with Discord: {}", e)));
                    }
                }
            } else if let Some(discord_cmd_id) = &current.discord_command_id {
                // Command is being disabled - remove it from Discord
                debug!("Removing command {} from Discord for guild {}", command_id, guild_id);
                
                match discord.delete_guild_command(&guild_id.to_string(), discord_cmd_id).await {
                    Ok(_) => {
                        debug!("Command successfully removed from Discord");
                    },
                    Err(e) => {
                        // Log the error but don't fail the whole operation
                        warn!("Failed to remove command from Discord: {}. Will update database anyway.", e);
                    }
                }
                
                // We're removing the command, so set the Discord ID to null
                new_discord_command_id = None;
            }
        }

        // Update in database with the new Discord command ID if applicable
        client
            .execute(
                "INSERT INTO guild_command_settings (guild_id, command_id, enabled, settings, discord_command_id)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (guild_id, command_id) 
                DO UPDATE SET 
                    enabled = EXCLUDED.enabled,
                    settings = EXCLUDED.settings,
                    discord_command_id = COALESCE(EXCLUDED.discord_command_id, guild_command_settings.discord_command_id)",
                &[&guild_id, &command_id, &enabled, &settings, &new_discord_command_id],
            )
            .await?;

        info!("Updated command settings for guild {} command {}", guild_id, command_id);
        
        Ok(())
    }
    
    /// Sync all enabled commands with Discord for a guild
    pub async fn sync_guild_commands(&self, guild_id: i64) -> Result<(), Error> {
        // If no Discord service is configured, just return
        let discord = match &self.discord {
            Some(service) => service,
            None => {
                debug!("No Discord service configured, skipping command sync");
                return Ok(());
            }
        };
        
        // Get all enabled guild commands
        let all_settings = self.get_all_command_settings(guild_id).await?;
        let enabled_commands: Vec<_> = all_settings
            .into_iter()
            .filter(|cmd| cmd.enabled)
            .collect();
            
        // Store the count for logging later
        let enabled_count = enabled_commands.len();
        
        // Get command details for all enabled commands
        let mut app_commands = Vec::new();
        
        for cmd in enabled_commands {
            let details = self.get_command_details(guild_id, &cmd.command_id).await?;
            
            // Convert to Discord application command format
            let mut options = Vec::new();
            
            // Add options if the command has them
            if let Some(config_schema) = details.config_schema {
                for opt in config_schema.options {
                    let discord_option_type = match opt.option_type.as_str() {
                        "string" => 3, // STRING
                        "integer" => 4, // INTEGER
                        "boolean" => 5, // BOOLEAN
                        "user" => 6,    // USER
                        "channel" => 7, // CHANNEL
                        "role" => 8,    // ROLE
                        _ => 3, // Default to STRING
                    };
                    
                    // Create choices for enum type
                    let choices = if let Some(enum_values) = opt.enum_values {
                        enum_values.iter()
                            .map(|ev| {
                                crate::web::services::discord::DiscordApplicationCommandOptionChoice {
                                    name: ev.label.clone(),
                                    value: serde_json::Value::String(ev.value.clone()),
                                }
                            })
                            .collect()
                    } else {
                        Vec::new()
                    };
                    
                    options.push(DiscordApplicationCommandOption {
                        option_type: discord_option_type,
                        name: opt.name,
                        description: opt.description,
                        required: opt.required,
                        choices,
                        options: Vec::new(),
                    });
                }
            }
            
            // Use discord_name if available, otherwise use name
            let command_name = details.discord_name.unwrap_or_else(|| {
                debug!("Using original name for command {}: no discord_name available", details.id);
                details.name.clone()
            });
            
            app_commands.push(DiscordApplicationCommand {
                id: None,
                name: command_name,
                description: details.description,
                options,
            });
        }
        
        // Sync commands with Discord
        discord.sync_guild_commands(&guild_id.to_string(), app_commands).await?;
        
        info!("Synced {} commands with Discord for guild {}", 
            enabled_count, guild_id);
        
        Ok(())
    }

    /// Get detailed command information including configuration schema
    pub async fn get_command_details(
        &self,
        guild_id: i64,
        command_id: &str,
    ) -> Result<CommandDetails, Error> {
        let client = self.db.get_client().await?;

        // Get basic command info
        let command_row = client
            .query_opt(
                "SELECT 
                    name, 
                    description, 
                    category, 
                    requires_admin,
                    options as config_schema,  -- Map options to config_schema
                    discord_name
                FROM commands
                WHERE command_id = $1",
                &[&command_id],
            )
            .await?;

        let command_row = match command_row {
            Some(row) => row,
            None => return Err(Error::Unknown(format!("Command not found: {}", command_id))),
        };

        // Get command settings for this guild
        let settings = self.get_command_settings(guild_id, command_id).await?;

        // Parse config schema if available
        let config_schema: Option<ConfigSchema> = command_row
            .get::<_, Option<Value>>(4)
            .map(|v| {
                serde_json::from_value(v).unwrap_or_else(|_| {
                    ConfigSchema { options: Vec::new() }
                })
            });

        // Build the detailed command info
        let details = CommandDetails {
            id: command_id.to_string(),
            name: command_row.get(0),
            description: command_row.get(1),
            category: command_row.get(2),
            requires_admin: command_row.get(3),
            config_schema,
            current_config: settings.settings,
            discord_name: command_row.get(5),
        };

        Ok(details)
    }

    /// Get all command settings for a guild
    pub async fn get_all_command_settings(
        &self,
        guild_id: i64,
    ) -> Result<Vec<CommandSettings>, Error> {
        let client = self.db.get_client().await?;

        // Get all commands
        let all_commands = self.get_available_commands().await?;
        let mut result = Vec::new();

        // For each command, get its settings
        for command in all_commands {
            let settings = self.get_command_settings(guild_id, &command.id).await?;
            result.push(settings);
        }

        Ok(result)
    }
    
    /// Initialize command settings for a guild
    /// This ensures that all commands have entries in the guild_command_settings table
    pub async fn initialize_guild_command_settings(&self, guild_id: i64) -> Result<(), Error> {
        let client = self.db.get_client().await?;
        
        // Get all available commands
        let commands = self.get_available_commands().await?;
        
        // For each command, check if settings exist, if not insert defaults
        for command in commands {
            let exists = client
                .query_one(
                    "SELECT 1 FROM guild_command_settings 
                     WHERE guild_id = $1 AND command_id = $2 LIMIT 1",
                    &[&guild_id, &command.id],
                )
                .await.is_ok();
                
            if !exists {
                // Determine the default enabled state
                let default_enabled = command.id == "ping" || command.id == "help";
                
                info!("Initializing settings for command {} in guild {} (enabled: {})", command.id, guild_id, default_enabled);
                client
                    .execute(
                        "INSERT INTO guild_command_settings 
                         (guild_id, command_id, enabled, settings, discord_command_id)
                         VALUES ($1, $2, $3, $4, $5)
                         ON CONFLICT (guild_id, command_id) DO NOTHING",
                        &[&guild_id, &command.id, &default_enabled, &None::<serde_json::Value>, &None::<String>],
                    )
                    .await?;
            }
        }
        
        info!("Initialized command settings for guild {}", guild_id);
        Ok(())
    }
}

/// Helper function to recursively convert ConfigOption to DiscordApplicationCommandOption
fn convert_options_recursive(options: &[ConfigOption]) -> Result<Vec<DiscordApplicationCommandOption>, Error> {
    let mut discord_options = Vec::new();

    for opt in options {
        let discord_option_type = match opt.option_type.as_str() {
            "sub_command" => 1,
            "sub_command_group" => 2,
            "string" => 3,
            "integer" => 4,
            "boolean" => 5,
            "user" => 6,
            "channel" => 7,
            "role" => 8,
            "mentionable" => 9,
            "number" => 10, // Represents float/double
            "attachment" => 11,
            _ => {
                warn!("Unknown option type '{}', defaulting to string", opt.option_type);
                3 // Default to STRING
            }
        };

        // Convert enum choices if present
        let choices = if let Some(enum_values) = &opt.enum_values {
            enum_values.iter()
                .map(|ev| DiscordApplicationCommandOptionChoice {
                    name: ev.label.clone(),
                    value: serde_json::Value::String(ev.value.clone()),
                })
                .collect()
        } else {
            Vec::new()
        };

        // Recursively convert nested options for subcommands/groups
        let nested_options = if discord_option_type == 1 || discord_option_type == 2 {
            if let Some(nested) = &opt.options {
                convert_options_recursive(nested)?
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        discord_options.push(DiscordApplicationCommandOption {
            option_type: discord_option_type,
            name: opt.name.clone(),
            description: opt.description.clone(),
            required: opt.required,
            choices,
            options: nested_options,
        });
    }

    Ok(discord_options)
}
