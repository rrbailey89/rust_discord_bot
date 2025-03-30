// src/web/services/command.rs
//! Command service for database operations related to commands

use std::collections::HashMap;
use crate::error::Error;
use crate::services::database::DatabaseService;
use crate::web::models::command::{
    CommandInfo, CommandDetails, CommandSettings, UpdateCommandSettingsRequest, CommandResponse,
    ConfigSchema, ConfigOption, EnumValue
};
use tracing::{debug, error, info};
use serde_json::Value;

use std::sync::Arc;
use crate::services::cache::CacheService;
use crate::web::services::discord::{DiscordService, DiscordApplicationCommand, DiscordApplicationCommandOption};

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
                    coalesce(options IS NOT NULL, false) as has_config
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
                "SELECT enabled, settings
                FROM guild_command_settings
                WHERE guild_id = $1 AND command_id = $2",
                &[&guild_id, &command_id],
            )
            .await?;

        // If no settings found, return default settings (enabled = true, no custom settings)
        let (enabled, settings) = match row {
            Some(r) => (r.get(0), r.get::<_, Option<Value>>(1)),
            None => (true, None),
        };

        Ok(CommandSettings {
            command_id: command_id.to_string(),
            enabled,
            settings: settings.map(|v| {
                serde_json::from_value(v).unwrap_or_else(|_| HashMap::new())
            }),
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
                "SELECT name, description FROM commands WHERE command_id = $1 LIMIT 1",
                &[&command_id],
            )
            .await?;

        let command_info = match command_row {
            Some(row) => {
                let name: String = row.get(0);
                let description: String = row.get(1);
                (name, description)
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

        // Update in database
        client
            .execute(
                "INSERT INTO guild_command_settings (guild_id, command_id, enabled, settings)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (guild_id, command_id) 
                DO UPDATE SET 
                    enabled = EXCLUDED.enabled,
                    settings = EXCLUDED.settings",
                &[&guild_id, &command_id, &enabled, &settings],
            )
            .await?;

        info!("Updated command settings for guild {} command {}", guild_id, command_id);
        
        // If the enabled state changed or we have an explicit rate limit, sync with Discord
        let enabled_changed = request.enabled.is_some() && request.enabled != Some(current.enabled);
        
        if enabled_changed && self.discord.is_some() {
            // Sync guild commands with Discord
            self.sync_guild_commands(guild_id).await?;
        }
        
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
            
            app_commands.push(DiscordApplicationCommand {
                id: None,
                name: details.name,
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
                    options as config_schema  -- Map options to config_schema
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
                info!("Initializing settings for command {} in guild {}", command.id, guild_id);
                client
                    .execute(
                        "INSERT INTO guild_command_settings 
                         (guild_id, command_id, enabled, settings)
                         VALUES ($1, $2, $3, $4)
                         ON CONFLICT (guild_id, command_id) DO NOTHING",
                        &[&guild_id, &command.id, &true, &None::<serde_json::Value>],
                    )
                    .await?;
            }
        }
        
        info!("Initialized command settings for guild {}", guild_id);
        Ok(())
    }
}
