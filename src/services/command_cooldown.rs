// services/command_cooldown.rs
use crate::error::Error;
use crate::services::database::DatabaseService;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Key for cooldown tracking: (guild_id, command_id, user_id)
type CooldownKey = (i64, String, i64);

/// Service for managing command cooldowns
#[derive(Debug)]
pub struct CommandCooldownService {
    /// Database service for persistent settings
    db: DatabaseService,
    /// In-memory cache of cooldowns for performance
    cooldowns: Arc<RwLock<HashMap<CooldownKey, Instant>>>,
}

impl CommandCooldownService {
    /// Create a new CommandCooldownService
    pub fn new(db: DatabaseService) -> Self {
        Self {
            db,
            cooldowns: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if a command is on cooldown
    /// 
    /// Returns Some(Duration) with the remaining cooldown time if on cooldown,
    /// or None if the command can be used.
    pub async fn is_on_cooldown(
        &self,
        guild_id: i64,
        command_id: &str,
        user_id: i64,
    ) -> Result<Option<Duration>, Error> {
        // Check in-memory cache first
        let key = (guild_id, command_id.to_string(), user_id);

        let cooldowns = self.cooldowns.read().await;
        if let Some(last_used) = cooldowns.get(&key) {
            // Get command settings
            let command_service = crate::web::services::CommandService::new(self.db.clone());
            let settings = command_service.get_command_settings(guild_id, command_id).await?;

            // Get cooldown from settings or default
            let cooldown_secs = settings
                .settings
                .and_then(|s| s.get("cooldown").cloned())
                .and_then(|v| v.as_u64())
                .or_else(|| {
                    // Get from command config if not in settings
                    let configs = crate::commands::get_command_config();
                    configs
                        .get(command_id)
                        .and_then(|c| c.cooldown)
                })
                .unwrap_or(0);

            if cooldown_secs > 0 {
                let cooldown = Duration::from_secs(cooldown_secs);
                let elapsed = last_used.elapsed();

                if elapsed < cooldown {
                    return Ok(Some(cooldown - elapsed));
                }
            }
        }

        Ok(None)
    }

    /// Record command usage for cooldown tracking
    pub async fn record_command_usage(
        &self,
        guild_id: i64,
        command_id: &str,
        user_id: i64,
    ) {
        let key = (guild_id, command_id.to_string(), user_id);
        let now = Instant::now();

        let mut cooldowns = self.cooldowns.write().await;
        cooldowns.insert(key, now);

        // Prune old entries if the map is getting too large
        if cooldowns.len() > 10000 {
            debug!("Pruning old cooldown entries");
            let five_minutes_ago = now - Duration::from_secs(300);
            
            cooldowns.retain(|_, timestamp| *timestamp > five_minutes_ago);
            info!("Pruned cooldown cache to {} entries", cooldowns.len());
        }
    }

    /// Clear cooldown for a user's command
    pub async fn clear_cooldown(&self, guild_id: i64, command_id: &str, user_id: i64) {
        let key = (guild_id, command_id.to_string(), user_id);
        let mut cooldowns = self.cooldowns.write().await;
        cooldowns.remove(&key);
    }

    /// Clear all cooldowns for a user
    pub async fn clear_user_cooldowns(&self, user_id: i64) {
        let mut cooldowns = self.cooldowns.write().await;
        cooldowns.retain(|(_, _, uid), _| *uid != user_id);
    }

    /// Clear all cooldowns in a guild
    pub async fn clear_guild_cooldowns(&self, guild_id: i64) {
        let mut cooldowns = self.cooldowns.write().await;
        cooldowns.retain(|(gid, _, _), _| *gid != guild_id);
    }
}
