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

        // Query relevant settings directly from guild_settings table
        let row_opt = client
            .query_opt(
                "SELECT
                    prefix,                     -- 0
                    mod_role_id,                -- 1
                    admin_role_id,              -- 2
                    settings,                   -- 3: Fetch the entire JSONB settings object
                    level_up_channel_id,        -- 4: Fetch directly from new column
                    warn_channel_id,            -- 5: Fetch directly from new column
                    delete_log_channel_id,      -- 6: Fetch directly from new column
                    reaction_log_channel_id     -- 7: Fetch directly from new column
                 FROM guild_settings
                 WHERE guild_id = $1",
                &[&guild_id],
            )
            .await?; // Propagate other DB errors

        // Extract data if row exists, otherwise use defaults
        let (prefix, mod_role, admin_role, settings_json_opt, lvl_chan, warn_chan, del_log_chan, react_log_chan) = match row_opt {
            Some(row) => {
                let pfx: Option<String> = row.get(0);
                let mod_r: Option<i64> = row.get(1);
                let admin_r: Option<i64> = row.get(2);
                let json_val: Option<JsonValue> = row.get(3);
                let lvl: Option<i64> = row.get(4);
                let warn_ch: Option<i64> = row.get(5);
                let del_log: Option<i64> = row.get(6);
                let react_log: Option<i64> = row.get(7);
                // Use empty JSON object if settings column is NULL
                (pfx, mod_r, admin_r, json_val, lvl, warn_ch, del_log, react_log)
            },
            None => {
                // Guild not found in guild_settings, return defaults
                debug!("No settings found for guild_id {}. Returning defaults.", guild_id);
                (None, None, None, None, None, None, None, None)
            }
        };

        // Use empty JSON if settings column was NULL or not an object
        let settings_json = settings_json_opt.unwrap_or_else(|| serde_json::json!({}));

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
            settings: Some(settings_json), // Store the potentially defaulted JSONB
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
        let mut tx = client.transaction().await?; // Start transaction

        // --- Ensure the row exists first ---
        // Use INSERT ... ON CONFLICT DO NOTHING to safely create the row if it doesn't exist.
        tx.execute(
                "INSERT INTO guild_settings (guild_id) VALUES ($1) ON CONFLICT (guild_id) DO NOTHING",
                &[&guild_id],
            )
            .await?;
        debug!("Ensured guild_settings row exists for guild {}", guild_id);

        // --- Prepare fields to update ---
        let mut updates: Vec<String> = Vec::new();
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
        params.push(&guild_id); // $1 is always guild_id

        // Helper to add field update
        let mut add_update = |field_name: &str, value: &(dyn tokio_postgres::types::ToSql + Sync)| {
            updates.push(format!("{} = ${}", field_name, params.len() + 1));
            params.push(value);
        };

        // Fetch current settings JSONB to merge updates
        let current_settings_row = tx
            .query_opt("SELECT settings FROM guild_settings WHERE guild_id = $1", &[&guild_id])
            .await?;

        let mut current_settings_json: JsonValue = current_settings_row
            .and_then(|row| row.get::<_, Option<JsonValue>>(0)) // Get Option<JsonValue>
            .unwrap_or_else(|| serde_json::json!({})); // Default to empty JSON object if NULL

        // Ensure current_settings_json is a map
        let settings_map = match current_settings_json.as_object_mut() {
            Some(map) => map,
            None => {
                warn!("Guild {} settings column was not a JSON object. Resetting.", guild_id);
                current_settings_json = serde_json::json!({});
                current_settings_json.as_object_mut().unwrap() // Should be safe now
            }
        };

        // Merge incoming nested settings
        if let Some(incoming_settings) = &request.settings {
             if let Some(incoming_map) = incoming_settings.as_object() {
                 debug!("Merging incoming nested settings for guild {}", guild_id);
                 for (key, value) in incoming_map {
                     // Avoid overwriting specific top-level fields managed separately if needed
                     if key != "emoji_reactions_enabled" && key != "url_rule" {
                          settings_map.insert(key.clone(), value.clone());
                     }
                 }
             } else {
                 warn!("Incoming settings for guild {} was not a JSON object, skipping merge.", guild_id);
             }
         }

        // Update specific fields within the JSON map if provided in the request
        if let Some(enabled) = request.emoji_reactions_enabled {
            settings_map.insert("emoji_reactions_enabled".to_string(), serde_json::json!(enabled));
        }
        if let Some(rule) = &request.url_rule {
            settings_map.insert("url_rule".to_string(), serde_json::json!(rule));
        } else {
             // If url_rule is explicitly None in request, remove it? Or handle null?
             // Current frontend sends null if empty, let's store null.
             settings_map.insert("url_rule".to_string(), JsonValue::Null);
        }
        // Add the potentially modified settings JSONB to the update list
        add_update("settings", &current_settings_json);


        // Add other top-level fields if present in the request
        if let Some(prefix) = &request.prefix {
            add_update("prefix", prefix);
        } else {
             // Handle explicit NULL setting for prefix if needed, e.g., if request sends null
             // For now, assume None means "no change" unless explicitly handled
             // add_update("prefix", &Option::<String>::None); // Example if you want to set NULL
        }

        // Handle Option<i64> fields correctly for NULL
        match request.mod_role_id {
            Some(id) => add_update("mod_role_id", &id),
            None => {
                updates.push(format!("mod_role_id = ${}", params.len() + 1));
                params.push(&Option::<i64>::None); // Explicitly push NULL
            }
        }
         match request.admin_role_id {
            Some(id) => add_update("admin_role_id", &id),
            None => {
                updates.push(format!("admin_role_id = ${}", params.len() + 1));
                params.push(&Option::<i64>::None); // Explicitly push NULL
            }
        }

        // Helper function to parse channel ID string to Option<i64>
        let parse_channel_id = |id_str: &Option<String>, field_name: &str| -> Result<Option<i64>, Error> {
            match id_str {
                Some(s) if !s.is_empty() => s.parse::<i64>().map(Some).map_err(|_| {
                    Error::Unknown(format!("Invalid {} format: {}", field_name, s))
                }),
                _ => Ok(None), // Treat empty string or None as NULL
            }
        };

        // Add channel IDs
        let level_up_id = parse_channel_id(&request.level_up_channel_id, "level_up_channel_id")?;
        updates.push(format!("level_up_channel_id = ${}", params.len() + 1));
        params.push(&level_up_id);

        let warn_id = parse_channel_id(&request.warn_channel_id, "warn_channel_id")?;
        updates.push(format!("warn_channel_id = ${}", params.len() + 1));
        params.push(&warn_id);

        let delete_log_id = parse_channel_id(&request.delete_log_channel_id, "delete_log_channel_id")?;
        updates.push(format!("delete_log_channel_id = ${}", params.len() + 1));
        params.push(&delete_log_id);

        let reaction_log_id = parse_channel_id(&request.reaction_log_channel_id, "reaction_log_channel_id")?;
        updates.push(format!("reaction_log_channel_id = ${}", params.len() + 1));
        params.push(&reaction_log_id);


        // --- Execute the dynamic UPDATE statement ---
        if !updates.is_empty() {
            updates.push("updated_at = NOW()".to_string()); // Always update timestamp
            let update_query = format!(
                "UPDATE guild_settings SET {} WHERE guild_id = $1",
                updates.join(", ")
            );
            debug!("Executing update query: {} with params: {:?}", update_query, params.len());
            tx.execute(&update_query, &params[..]).await?;
            debug!("Successfully updated guild_settings for guild {}", guild_id);
        } else {
            debug!("No fields to update for guild {}", guild_id);
        }

        tx.commit().await?; // Commit transaction
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
