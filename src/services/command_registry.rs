// services/command_registry.rs
use crate::commands::{CommandConfig, CommandScope};
use crate::error::Error;
use crate::services::database::DatabaseService;
use crate::Data;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::GuildId;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Service for managing command registration at the guild level
#[derive(Clone)]
pub struct CommandRegistryService {
    db: DatabaseService,
    command_configs: HashMap<String, CommandConfig>,
}

// Custom Debug implementation to avoid potential issues
impl fmt::Debug for CommandRegistryService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommandRegistryService")
            .field("command_configs_count", &self.command_configs.len())
            .finish()
    }
}

impl CommandRegistryService {
    /// Create a new CommandRegistryService
    pub fn new(
        db: DatabaseService,
        _http: Arc<serenity::Http>,
    ) -> Self {
        let command_configs = crate::commands::get_command_config()
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();

        Self {
            db,
            command_configs,
        }
    }

    /// Get a list of command names that should be global
    pub fn get_global_command_names(&self) -> Vec<String> {
        self.command_configs
            .iter()
            .filter_map(|(name, config)| {
                if let CommandScope::Global = config.scope {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get a list of command names that should be guild-specific
    pub fn get_guild_command_names(&self) -> Vec<String> {
        self.command_configs
            .iter()
            .filter_map(|(name, config)| {
                if let CommandScope::Guild = config.scope {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check if a command should be registered globally
    pub fn is_global_command(&self, command_name: &str) -> bool {
        if let Some(config) = self.command_configs.get(command_name) {
            matches!(config.scope, CommandScope::Global)
        } else {
            false
        }
    }

    /// Check if a command should be registered per-guild
    pub fn is_guild_command(&self, command_name: &str) -> bool {
        if let Some(config) = self.command_configs.get(command_name) {
            matches!(config.scope, CommandScope::Guild)
        } else {
            false
        }
    }

    /// Get list of enabled commands for a guild
    pub async fn get_enabled_guild_commands(&self, guild_id: i64) -> Result<Vec<String>, Error> {
        let command_service = crate::web::services::CommandService::new(self.db.clone());
        let guild_commands = command_service
            .get_all_command_settings(guild_id)
            .await?;

        // Filter to only enabled guild commands
        let enabled_commands = guild_commands
            .into_iter()
            .filter(|cmd| {
                // Only include enabled guild-scoped commands
                if cmd.enabled {
                    if let Some(config) = self.command_configs.get(&cmd.command_id) {
                        return matches!(config.scope, CommandScope::Guild);
                    }
                }
                false
            })
            .map(|cmd| cmd.command_id)
            .collect();

        Ok(enabled_commands)
    }
}
