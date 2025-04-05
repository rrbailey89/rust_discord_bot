// src/web/services/guild.rs
//! Guild service for database operations related to guilds

use crate::error::Error;
use crate::services::database::DatabaseService;
use crate::web::models::guild::{GuildSettings, UpdateGuildSettingsRequest};
use crate::web::models::auth::Guild;
use serde_json::{Value as JsonValue, Map}; // Import serde_json Value and Map
use tokio_postgres::error::SqlState; // Import SqlState for error handling
use tracing::{debug, error, info, warn}; // Added warn

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

        // Query relevant settings from guild_settings and guild_channels
        // Query relevant settings from guild_settings and join/subquery guild_channels
        let row_opt = client
            .query_opt(
                "SELECT
                    gs.prefix,                     -- Fetch prefix
                    gs.mod_role_id,                -- Fetch mod_role_id
                    gs.admin_role_id,              -- Fetch admin_role_id
                    gs.settings,                   -- Fetch the entire JSONB settings object
                    (SELECT channel_id FROM guild_channels WHERE guild_id = $1 AND channel_type = 'level_up') as level_up_channel_id,
                    (SELECT channel_id FROM guild_channels WHERE guild_id = $1 AND channel_type = 'warn') as warn_channel_id,
                    (SELECT channel_id FROM guild_channels WHERE guild_id = $1 AND channel_type = 'delete_log') as delete_log_channel_id,
                    (SELECT channel_id FROM guild_channels WHERE guild_id = $1 AND channel_type = 'reaction_log') as reaction_log_channel_id
                 FROM guild_settings gs
                 WHERE gs.guild_id = $1",
                &[&guild_id],
            )
            .await?; // Propagate other DB errors

        // Extract data if row exists, otherwise use defaults
        let (prefix, mod_role, admin_role, settings_json, lvl_chan, warn_chan, del_log_chan, react_log_chan) = match row_opt {
            Some(row) => {
                let pfx = row.get::<_, Option<String>>(0);
                let mod_r = row.get::<_, Option<i64>>(1);
                let admin_r = row.get::<_, Option<i64>>(2);
                let json_val = row.get::<_, Option<JsonValue>>(3).unwrap_or_else(|| serde_json::json!({}));
                let lvl = row.get::<_, Option<i64>>(4);
                let warn_ch = row.get::<_, Option<i64>>(5);
                let del_log = row.get::<_, Option<i64>>(6);
                let react_log = row.get::<_, Option<i64>>(7);
                (pfx, mod_r, admin_r, json_val, lvl, warn_ch, del_log, react_log)
            },
            None => {
                // Guild not found in guild_settings, return defaults
                debug!("No settings found for guild_id {}. Returning defaults.", guild_id);
                (None, None, None, serde_json::json!({}), None, None, None, None)
            }
        };

        // Extract emoji_reactions_enabled, default to true if missing or not a boolean
        let emoji_reactions_enabled = settings_json
            .get("emoji_reactions_enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true); // Default to true

        // Extract url_rule from settings JSONB
        let url_rule = settings_json
            .get("url_rule")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Map the result to a GuildSettings object
        let guild_settings_result = GuildSettings {
            guild_id,
            prefix, // Use fetched value
            mod_role_id: mod_role, // Use fetched value
            admin_role_id: admin_role, // Use fetched value
            settings: Some(settings_json), // Store the raw JSONB
            emoji_reactions_enabled: Some(emoji_reactions_enabled), // Use extracted value
            // Assign the correctly typed channel variables
            level_up_channel_id: lvl_chan,
            warn_channel_id: warn_chan,
            url_rule, // Use extracted value from JSONB
            delete_log_channel_id: del_log_chan,
            reaction_log_channel_id: react_log_chan,
        };

        Ok(guild_settings_result)
    }

    /// Update guild settings in the database
    pub async fn update_guild_settings(
        &self,
        guild_id: i64,
        request: &UpdateGuildSettingsRequest,
    ) -> Result<(), Error> {
        debug!("Received update request for guild {}: {:?}", guild_id, request); // Log incoming request
        let client = self.db.get_client().await?;

        // Fetch current settings JSONB to merge updates
        debug!("Fetching current settings for guild {}", guild_id);
        let current_settings_row = client
            .query_opt("SELECT settings FROM guild_settings WHERE guild_id = $1", &[&guild_id])
            .await?;

        let mut current_settings: JsonValue = current_settings_row
            .map(|row| row.get::<_, Option<JsonValue>>(0).unwrap_or_else(|| serde_json::json!({})))
            .unwrap_or_else(|| serde_json::json!({}));

        // Ensure current_settings is a map
        let settings_map = match current_settings.as_object_mut() {
            Some(map) => map,
            None => {
                // If it's not an object (e.g., null or other type), reset to empty object
                warn!("Guild {} settings column was not a JSON object. Resetting.", guild_id);
                current_settings = serde_json::json!({});
                current_settings.as_object_mut().unwrap() // Should be safe now
            }
        };

        // Merge incoming nested settings first (if provided)
        if let Some(incoming_settings) = &request.settings {
            if let Some(incoming_map) = incoming_settings.as_object() {
                debug!("Merging incoming nested settings for guild {}", guild_id);
                for (key, value) in incoming_map {
                    // Avoid overwriting emoji_reactions_enabled if it's explicitly set at top level
                    if key != "emoji_reactions_enabled" {
                         settings_map.insert(key.clone(), value.clone());
                    }
                }
            } else {
                warn!("Incoming settings for guild {} was not a JSON object, skipping merge.", guild_id);
            }
        }

        // Update top-level settings within the JSON map *after* potential merge
        // This ensures the explicit top-level values take precedence

        // Update emoji reactions setting if provided
        if let Some(enabled) = request.emoji_reactions_enabled {
            settings_map.insert("emoji_reactions_enabled".to_string(), serde_json::json!(enabled));
            debug!("Set emoji reactions in JSON for guild {}: {}", guild_id, enabled);
        }

        // Update URL rule setting if provided
        if let Some(rule) = &request.url_rule {
            settings_map.insert("url_rule".to_string(), serde_json::json!(rule));
             debug!("Prepared URL rule update for guild {}: {}", guild_id, rule);
        }

        // --- Persist updated settings JSONB and top-level fields ---
        // Use INSERT ... ON CONFLICT for initial row creation or updating settings JSONB
        // We handle other top-level fields separately for clarity
        client
            .execute(
                "INSERT INTO guild_settings (guild_id, settings)
                 VALUES ($1, $2)
                 ON CONFLICT (guild_id) DO UPDATE
                 SET settings = $2", // Update the entire settings object
                &[&guild_id, &current_settings], // Use the potentially merged current_settings
            )
            .await?;
        debug!("Persisted updated settings JSONB for guild {}: {:?}", guild_id, &current_settings); // Log the JSON being saved

        // Update top-level fields individually if provided
        if let Some(prefix) = &request.prefix {
            client
                .execute(
                    "UPDATE guild_settings SET prefix = $2 WHERE guild_id = $1",
                    &[&guild_id, prefix],
                )
                .await?;
            debug!("Updated prefix for guild {} to '{}'", guild_id, prefix);
        }
        if let Some(mod_role_id) = request.mod_role_id {
             client
                .execute(
                    "UPDATE guild_settings SET mod_role_id = $2 WHERE guild_id = $1",
                    &[&guild_id, &mod_role_id],
                )
                .await?;
             debug!("Updated mod_role_id for guild {}", guild_id);
        }
        if let Some(admin_role_id) = request.admin_role_id {
             client
                .execute(
                    "UPDATE guild_settings SET admin_role_id = $2 WHERE guild_id = $1",
                    &[&guild_id, &admin_role_id],
                )
                .await?;
             debug!("Updated admin_role_id for guild {}", guild_id);
        }


        // --- Update Channel IDs in guild_channels table ---

        // Update level up channel if provided
        if let Some(channel_id_str) = &request.level_up_channel_id {
            // Parse channel ID to ensure it's valid
            let channel_id_i64 = channel_id_str.parse::<i64>().map_err(|_| {
                Error::Unknown(format!("Invalid level_up_channel_id format: {}", channel_id_str))
            })?;

            client
                .execute(
                    "INSERT INTO guild_channels (guild_id, channel_type, channel_id)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (guild_id, channel_type)
                    DO UPDATE SET channel_id = $3",
                    &[&guild_id, &"level_up", &channel_id_i64], // Use the parsed i64
                )
                .await?;

            debug!("Updated level_up channel for guild {} to {}", guild_id, channel_id_str);
        }

        // Update warn channel if provided
        if let Some(channel_id_str) = &request.warn_channel_id {
            // Parse channel ID to ensure it's valid
            let channel_id_i64 = channel_id_str.parse::<i64>().map_err(|_| {
                Error::Unknown(format!("Invalid warn_channel_id format: {}", channel_id_str))
            })?;

            client
                .execute(
                    "INSERT INTO guild_channels (guild_id, channel_type, channel_id)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (guild_id, channel_type)
                    DO UPDATE SET channel_id = $3",
                    &[&guild_id, &"warn", &channel_id_i64], // Use the parsed i64
                )
                .await?;

            debug!("Updated warn channel for guild {} to {}", guild_id, channel_id_str);
        }

        // Update delete log channel if provided
        if let Some(channel_id_str) = &request.delete_log_channel_id {
            // Parse channel ID to ensure it's valid
            let channel_id_i64 = channel_id_str.parse::<i64>().map_err(|_| {
                Error::Unknown(format!("Invalid delete_log_channel_id format: {}", channel_id_str))
            })?;

            client
                .execute(
                    "INSERT INTO guild_channels (guild_id, channel_type, channel_id)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (guild_id, channel_type)
                    DO UPDATE SET channel_id = $3",
                    &[&guild_id, &"delete_log", &channel_id_i64], // Use the parsed i64
                )
                .await?;

            debug!("Updated delete_log channel for guild {} to {}", guild_id, channel_id_str);
        }

        // Update reaction log channel if provided
        if let Some(channel_id_str) = &request.reaction_log_channel_id {
            // Parse channel ID to ensure it's valid
            let channel_id_i64 = channel_id_str.parse::<i64>().map_err(|_| {
                Error::Unknown(format!("Invalid reaction_log_channel_id format: {}", channel_id_str))
            })?;

            client
                .execute(
                    "INSERT INTO guild_channels (guild_id, channel_type, channel_id)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (guild_id, channel_type)
                    DO UPDATE SET channel_id = $3",
                    &[&guild_id, &"reaction_log", &channel_id_i64], // Use the parsed i64
                )
                .await?;

            debug!("Updated reaction_log channel for guild {} to {}", guild_id, channel_id_str);
        }

        info!("Successfully updated settings for guild {}", guild_id);
        Ok(())
    }

    /// Check if the bot is in a specific guild
    pub async fn is_bot_in_guild(&self, guild_id: i64) -> Result<bool, Error> {
        let client = self.db.get_client().await?;

        // First check if the guild exists in guild_info
        let guild_exists = client
            .query_opt(
                "SELECT 1 FROM guild_info WHERE guild_id = $1 LIMIT 1",
                &[&guild_id],
            )
            .await?
            .is_some();

        // Then check if we have any members for this guild
        let has_members = client
            .query_opt(
                "SELECT 1 FROM guild_members WHERE guild_id = $1 LIMIT 1",
                &[&guild_id],
            )
            .await?
            .is_some();

        // The bot is considered to be in the guild if both conditions are true
        Ok(guild_exists && has_members)
    }
    
    /// Get the member count for a specific guild
    pub async fn get_guild_member_count(&self, guild_id: i64) -> Result<i32, Error> {
        let client = self.db.get_client().await?;
        
        // Use a simple count query with error handling
        match client
            .query_opt(
                "SELECT COUNT(*) FROM guild_members WHERE guild_id = $1",
                &[&guild_id],
            )
            .await {
                Ok(Some(row)) => {
                    // Successfully got the count
                    let count: i64 = row.get(0);
                    Ok(count as i32)
                },
                Ok(None) => {
                    // This shouldn't happen with COUNT(*), but handle it anyway
                    debug!("No count result returned for guild {}", guild_id);
                    Ok(0)
                },
                Err(e) => {
                    // Handle database errors by logging and returning 0
                    error!("Error counting members for guild {}: {}", guild_id, e);
                    Ok(0) // Return 0 instead of propagating the error
                }
            }
    }

    /// Get all guilds where the bot is a member
    pub async fn get_guilds(&self) -> Result<Vec<Guild>, Error> {
        let client = self.db.get_client().await?;
        
        // Query all guilds from the database where the bot is a member
        let rows = client
            .query(
                "SELECT 
                    g.guild_id, 
                    g.name, 
                    g.icon_hash,
                    g.owner_id
                FROM guild_info g
                WHERE EXISTS (
                    SELECT 1 FROM guild_members 
                    WHERE guild_id = g.guild_id 
                    LIMIT 1
                )
                ORDER BY g.name",
                &[],
            )
            .await?;
        
        // Manually create the guild vector
        let mut guilds = Vec::new();
        
        for row in rows.iter() {
            let guild_id = row.get::<_, i64>(0);
            let guild_name = row.get::<_, String>(1);
            let icon_hash = row.get::<_, Option<String>>(2);
            let owner_id = row.get::<_, Option<i64>>(3);
            
            guilds.push(Guild {
                id: guild_id.to_string(),
                name: guild_name,
                icon: icon_hash,
                owner: owner_id.is_some(),
                permissions: 0, // Default permissions 
                bot_joined: true,
            });
        }
        
        info!("Retrieved {} guilds where bot is a member", guilds.len());
        Ok(guilds)
    }
}
