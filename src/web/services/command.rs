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

/// Command service for database operations
pub struct CommandService {
    /// Database service
    db: DatabaseService,
}

impl CommandService {
    /// Create a new command service
    pub fn new(db: DatabaseService) -> Self {
        Self { db }
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
                    has_config
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
        let command_exists = client
            .query_one(
                "SELECT 1 FROM commands WHERE command_id = $1 LIMIT 1",
                &[&command_id],
            )
            .await
            .is_ok();

        if !command_exists {
            return Err(Error::Unknown(format!("Command not found: {}", command_id)));
        }

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
                    config_schema
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
}
