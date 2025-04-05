// src/web/services/guild.rs
//! Guild service for database operations related to guilds

use crate::error::Error;
use crate::services::database::DatabaseService;
use crate::web::models::guild::{GuildSettings, UpdateGuildSettingsRequest};
use crate::web::models::auth::Guild;
use serde_json::{Value as JsonValue};
 // Import ToSql trait
use tracing::{debug, error, info, warn};

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
            .await?;

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
                (pfx, mod_r, admin_r, json_val, lvl, warn_ch, del_log, react_log)
            },
            None => {
                debug!("No settings found for guild_id {}. Returning defaults.", guild_id);
                (None, None, None, None, None, None, None, None)
            }
        };

        let settings_json = settings_json_opt.unwrap_or_else(|| serde_json::json!({}));

        let emoji_reactions_enabled = settings_json
            .get("emoji_reactions_enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let url_rule = settings_json
            .get("url_rule")
            .and_then(|v| v.as_str())
            .map(String::from);

        let guild_settings_result = GuildSettings {
            guild_id,
            prefix,
            mod_role_id: mod_role,
            admin_role_id: admin_role,
            settings: Some(settings_json),
            emoji_reactions_enabled: Some(emoji_reactions_enabled),
            level_up_channel_id: lvl_chan,
            warn_channel_id: warn_chan,
            url_rule,
            delete_log_channel_id: del_log_chan,
            reaction_log_channel_id: react_log_chan,
        };

        Ok(guild_settings_result)
    }

    /// Update guild settings in the database using individual UPDATE statements within a transaction
    pub async fn update_guild_settings(
        &self,
        guild_id: i64,
        request: &UpdateGuildSettingsRequest,
    ) -> Result<(), Error> {
        debug!("Received update request for guild {}: {:?}", guild_id, request);
        let mut client = self.db.get_client().await?;
        let tx = client.transaction().await?;

        // --- Ensure the row exists first ---
        tx.execute(
            "INSERT INTO guild_settings (guild_id) VALUES ($1) ON CONFLICT (guild_id) DO NOTHING",
            &[&guild_id],
        ).await?;
        debug!("Ensured guild_settings row exists for guild {}", guild_id);

        // --- Update fields individually ---

        // Fetch and merge settings JSONB
        let current_settings_row = tx
            .query_opt("SELECT settings FROM guild_settings WHERE guild_id = $1", &[&guild_id])
            .await?;
        let mut current_settings_json: JsonValue = current_settings_row
            .and_then(|row| row.get::<_, Option<JsonValue>>(0))
            .unwrap_or_else(|| serde_json::json!({}));

        if !current_settings_json.is_object() {
            warn!("Guild {} settings column was not a JSON object. Resetting.", guild_id);
            current_settings_json = serde_json::json!({});
        }

        let mut settings_changed = false;
        if let Some(settings_map) = current_settings_json.as_object_mut() {
            if let Some(incoming_settings) = &request.settings {
                if let Some(incoming_map) = incoming_settings.as_object() {
                    for (key, value) in incoming_map {
                        if key != "emoji_reactions_enabled" && key != "url_rule" {
                            if settings_map.get(key) != Some(value) {
                                settings_map.insert(key.clone(), value.clone());
                                settings_changed = true;
                            }
                        }
                    }
                }
            }
            if let Some(enabled) = request.emoji_reactions_enabled {
                 if settings_map.get("emoji_reactions_enabled") != Some(&serde_json::json!(enabled)) {
                    settings_map.insert("emoji_reactions_enabled".to_string(), serde_json::json!(enabled));
                    settings_changed = true;
                 }
            }
            let url_rule_json = request.url_rule.as_ref().map_or(JsonValue::Null, |r| serde_json::json!(r));
            if settings_map.get("url_rule") != Some(&url_rule_json) {
                settings_map.insert("url_rule".to_string(), url_rule_json);
                settings_changed = true;
            }
        }

        if settings_changed {
            tx.execute(
                "UPDATE guild_settings SET settings = $2, updated_at = NOW() WHERE guild_id = $1",
                &[&guild_id, &current_settings_json],
            ).await?;
            debug!("Updated settings JSONB for guild {}", guild_id);
        }

        // Update prefix if provided
        if request.prefix.is_some() {
             // We need to bind the Option<String> itself, not just the inner String
            tx.execute(
                "UPDATE guild_settings SET prefix = $2, updated_at = NOW() WHERE guild_id = $1",
                &[&guild_id, &request.prefix],
            ).await?;
             debug!("Updated prefix for guild {}", guild_id);
        }
        // Note: If request.prefix is None, we don't update, preserving the existing value.
        // If you want None to explicitly set the DB field to NULL, you'd need:
        // else {
        //     tx.execute("UPDATE guild_settings SET prefix = NULL, updated_at = NOW() WHERE guild_id = $1", &[&guild_id]).await?;
        // }


        // Update mod_role_id (Option<i64> directly maps to nullable BIGINT)
        tx.execute(
            "UPDATE guild_settings SET mod_role_id = $2, updated_at = NOW() WHERE guild_id = $1",
            &[&guild_id, &request.mod_role_id],
        ).await?;
        debug!("Updated mod_role_id for guild {} to {:?}", guild_id, request.mod_role_id);

        // Update admin_role_id
        tx.execute(
            "UPDATE guild_settings SET admin_role_id = $2, updated_at = NOW() WHERE guild_id = $1",
            &[&guild_id, &request.admin_role_id],
        ).await?;
        debug!("Updated admin_role_id for guild {} to {:?}", guild_id, request.admin_role_id);


        // Helper function to parse channel ID string to Option<i64>
        let parse_channel_id = |id_str: &Option<String>, field_name: &str| -> Result<Option<i64>, Error> {
            match id_str {
                Some(s) if !s.is_empty() => s.parse::<i64>().map(Some).map_err(|_| {
                    Error::Unknown(format!("Invalid {} format: {}", field_name, s))
                }),
                _ => Ok(None), // Treat empty string or None as NULL
            }
        };

        // Update level_up_channel_id
        let level_up_id = parse_channel_id(&request.level_up_channel_id, "level_up_channel_id")?;
        tx.execute(
            "UPDATE guild_settings SET level_up_channel_id = $2, updated_at = NOW() WHERE guild_id = $1",
            &[&guild_id, &level_up_id], // Pass Option<i64> directly
        ).await?;
        debug!("Updated level_up_channel_id for guild {} to {:?}", guild_id, level_up_id);

        // Update warn_channel_id
        let warn_id = parse_channel_id(&request.warn_channel_id, "warn_channel_id")?;
        tx.execute(
            "UPDATE guild_settings SET warn_channel_id = $2, updated_at = NOW() WHERE guild_id = $1",
            &[&guild_id, &warn_id],
        ).await?;
        debug!("Updated warn_channel_id for guild {} to {:?}", guild_id, warn_id);

        // Update delete_log_channel_id
        let delete_log_id = parse_channel_id(&request.delete_log_channel_id, "delete_log_channel_id")?;
        tx.execute(
            "UPDATE guild_settings SET delete_log_channel_id = $2, updated_at = NOW() WHERE guild_id = $1",
            &[&guild_id, &delete_log_id],
        ).await?;
        debug!("Updated delete_log_channel_id for guild {} to {:?}", guild_id, delete_log_id);

        // Update reaction_log_channel_id
        let reaction_log_id = parse_channel_id(&request.reaction_log_channel_id, "reaction_log_channel_id")?;
        tx.execute(
            "UPDATE guild_settings SET reaction_log_channel_id = $2, updated_at = NOW() WHERE guild_id = $1",
            &[&guild_id, &reaction_log_id],
        ).await?;
        debug!("Updated reaction_log_channel_id for guild {} to {:?}", guild_id, reaction_log_id);

        tx.commit().await?; // Commit transaction
        info!("Successfully updated settings for guild {}", guild_id);
        Ok(())
    }


    /// Check if the bot is in a specific guild
    pub async fn is_bot_in_guild(&self, guild_id: i64) -> Result<bool, Error> {
        let client = self.db.get_client().await?;

        let guild_exists = client
            .query_opt(
                "SELECT 1 FROM guild_info WHERE guild_id = $1 LIMIT 1",
                &[&guild_id],
            )
            .await?
            .is_some();

        let has_members = client
            .query_opt(
                "SELECT 1 FROM guild_members WHERE guild_id = $1 LIMIT 1",
                &[&guild_id],
            )
            .await?
            .is_some();

        Ok(guild_exists && has_members)
    }

    /// Get the member count for a specific guild
    pub async fn get_guild_member_count(&self, guild_id: i64) -> Result<i32, Error> {
        let client = self.db.get_client().await?;

        match client
            .query_opt(
                "SELECT COUNT(*) FROM guild_members WHERE guild_id = $1",
                &[&guild_id],
            )
            .await {
                Ok(Some(row)) => {
                    let count: i64 = row.get(0);
                    Ok(count as i32)
                },
                Ok(None) => {
                    debug!("No count result returned for guild {}", guild_id);
                    Ok(0)
                },
                Err(e) => {
                    error!("Error counting members for guild {}: {}", guild_id, e);
                    Ok(0)
                }
            }
    }

    /// Get all guilds where the bot is a member
    pub async fn get_guilds(&self) -> Result<Vec<Guild>, Error> {
        let client = self.db.get_client().await?;

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
                permissions: 0,
                bot_joined: true,
            });
        }

        info!("Retrieved {} guilds where bot is a member", guilds.len());
        Ok(guilds)
    }
}
