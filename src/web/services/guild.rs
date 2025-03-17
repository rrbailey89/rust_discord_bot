// src/web/services/guild.rs
//! Guild service for database operations related to guilds

use crate::error::Error;
use crate::services::database::DatabaseService;
use crate::web::models::guild::{GuildSettings, UpdateGuildSettingsRequest};
use tracing::{debug, error, info};

/// Guild service for database operations
pub struct GuildService {
    /// Database service
    db: DatabaseService,
}

impl GuildService {
    /// Create a new guild service
    pub fn new(db: DatabaseService) -> Self {
        Self { db }
    }

    /// Get guild settings from the database
    pub async fn get_guild_settings(&self, guild_id: i64) -> Result<GuildSettings, Error> {
        let client = self.db.get_client().await?;

        // Query relevant settings from multiple tables
        let row = client
            .query_one(
                "SELECT 
                    COALESCE((SELECT enabled FROM guild_settings WHERE guild_id = $1 AND setting = 'emoji_reactions'), true) as emoji_reactions_enabled,
                    (SELECT channel_id FROM guild_channels WHERE guild_id = $1 AND channel_type = 'level_up') as level_up_channel_id,
                    (SELECT channel_id FROM guild_channels WHERE guild_id = $1 AND channel_type = 'warn') as warn_channel_id,
                    (SELECT rule FROM guild_settings WHERE guild_id = $1 AND setting = 'url_rule') as url_rule,
                    (SELECT channel_id FROM guild_channels WHERE guild_id = $1 AND channel_type = 'delete_log') as delete_log_channel_id,
                    (SELECT channel_id FROM guild_channels WHERE guild_id = $1 AND channel_type = 'reaction_log') as reaction_log_channel_id",
                &[&guild_id],
            )
            .await?;

        // Map the result to a GuildSettings object
        let settings = GuildSettings {
            emoji_reactions_enabled: row.get::<_, bool>(0),
            level_up_channel_id: row.get::<_, Option<String>>(1),
            warn_channel_id: row.get::<_, Option<String>>(2),
            url_rule: row.get::<_, Option<String>>(3),
            delete_log_channel_id: row.get::<_, Option<String>>(4),
            reaction_log_channel_id: row.get::<_, Option<String>>(5),
        };

        Ok(settings)
    }

    /// Update guild settings in the database
    pub async fn update_guild_settings(
        &self,
        guild_id: i64,
        request: &UpdateGuildSettingsRequest,
    ) -> Result<(), Error> {
        let client = self.db.get_client().await?;
        
        // Update emoji reactions setting if provided
        if let Some(emoji_reactions_enabled) = request.emoji_reactions_enabled {
            client
                .execute(
                    "INSERT INTO guild_settings (guild_id, setting, enabled) 
                    VALUES ($1, 'emoji_reactions', $2)
                    ON CONFLICT (guild_id, setting) 
                    DO UPDATE SET enabled = $2",
                    &[&guild_id, &emoji_reactions_enabled],
                )
                .await?;

            debug!(
                "Updated emoji reactions setting for guild {}: {}",
                guild_id, emoji_reactions_enabled
            );
        }

        // Update URL rule setting if provided
        if let Some(url_rule) = &request.url_rule {
            client
                .execute(
                    "INSERT INTO guild_settings (guild_id, setting, rule) 
                    VALUES ($1, 'url_rule', $2)
                    ON CONFLICT (guild_id, setting) 
                    DO UPDATE SET rule = $2",
                    &[&guild_id, url_rule],
                )
                .await?;

            debug!(
                "Updated URL rule setting for guild {}: {}",
                guild_id, url_rule
            );
        }

        // Update level up channel if provided
        if let Some(channel_id) = &request.level_up_channel_id {
            // Parse channel ID to ensure it's valid
            let _channel_id_i64 = channel_id.parse::<i64>().map_err(|_| {
                Error::Unknown(format!("Invalid channel ID format: {}", channel_id))
            })?;
            
            client
                .execute(
                    "INSERT INTO guild_channels (guild_id, channel_type, channel_id) 
                    VALUES ($1, $2, $3)
                    ON CONFLICT (guild_id, channel_type) 
                    DO UPDATE SET channel_id = $3",
                    &[&guild_id, &"level_up", channel_id],
                )
                .await?;
                
            debug!("Updated level_up channel for guild {} to {}", guild_id, channel_id);
        }
        
        // Update warn channel if provided
        if let Some(channel_id) = &request.warn_channel_id {
            // Parse channel ID to ensure it's valid
            let _channel_id_i64 = channel_id.parse::<i64>().map_err(|_| {
                Error::Unknown(format!("Invalid channel ID format: {}", channel_id))
            })?;
            
            client
                .execute(
                    "INSERT INTO guild_channels (guild_id, channel_type, channel_id) 
                    VALUES ($1, $2, $3)
                    ON CONFLICT (guild_id, channel_type) 
                    DO UPDATE SET channel_id = $3",
                    &[&guild_id, &"warn", channel_id],
                )
                .await?;
                
            debug!("Updated warn channel for guild {} to {}", guild_id, channel_id);
        }
        
        // Update delete log channel if provided
        if let Some(channel_id) = &request.delete_log_channel_id {
            // Parse channel ID to ensure it's valid
            let _channel_id_i64 = channel_id.parse::<i64>().map_err(|_| {
                Error::Unknown(format!("Invalid channel ID format: {}", channel_id))
            })?;
            
            client
                .execute(
                    "INSERT INTO guild_channels (guild_id, channel_type, channel_id) 
                    VALUES ($1, $2, $3)
                    ON CONFLICT (guild_id, channel_type) 
                    DO UPDATE SET channel_id = $3",
                    &[&guild_id, &"delete_log", channel_id],
                )
                .await?;
                
            debug!("Updated delete_log channel for guild {} to {}", guild_id, channel_id);
        }
        
        // Update reaction log channel if provided
        if let Some(channel_id) = &request.reaction_log_channel_id {
            // Parse channel ID to ensure it's valid
            let _channel_id_i64 = channel_id.parse::<i64>().map_err(|_| {
                Error::Unknown(format!("Invalid channel ID format: {}", channel_id))
            })?;
            
            client
                .execute(
                    "INSERT INTO guild_channels (guild_id, channel_type, channel_id) 
                    VALUES ($1, $2, $3)
                    ON CONFLICT (guild_id, channel_type) 
                    DO UPDATE SET channel_id = $3",
                    &[&guild_id, &"reaction_log", channel_id],
                )
                .await?;
                
            debug!("Updated reaction_log channel for guild {} to {}", guild_id, channel_id);
        }

        info!("Successfully updated settings for guild {}", guild_id);
        Ok(())
    }

    /// Check if the bot is in a specific guild
    pub async fn is_bot_in_guild(&self, guild_id: i64) -> Result<bool, Error> {
        let client = self.db.get_client().await?;

        // Query to check if the guild exists in our database
        let row = client
            .query_opt(
                "SELECT 1 FROM guild_info WHERE guild_id = $1 LIMIT 1",
                &[&guild_id],
            )
            .await?;

        Ok(row.is_some())
    }
}
