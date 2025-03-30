// src/web/state.rs
//! Shared state for the web application

use crate::Data;
use std::sync::Arc;

/// WebAppState holds shared data between the web server and the Discord bot
#[derive(Clone)]
pub struct WebAppState {
    /// Shared bot data
    pub bot_data: Arc<Data>,
}

impl WebAppState {
    /// Create a new WebAppState
    pub fn new(bot_data: Arc<Data>) -> Self {
        Self { bot_data }
    }
    
    /// Get a reference to the database service
    pub fn database(&self) -> &crate::services::database::DatabaseService {
        &self.bot_data.database
    }
    
    /// Get a reference to the config
    pub fn config(&self) -> &Arc<crate::config::Config> {
        &self.bot_data.config
    }
    
    /// Get a reference to the logging service
    pub fn logging(&self) -> &Arc<crate::services::LoggingService> {
        &self.bot_data.logging
    }
    
    /// Get a reference to the cache service
    pub fn cache(&self) -> &Arc<crate::services::cache::CacheService> {
        &self.bot_data.cache
    }
    
    /// Get a new authentication service
    pub fn auth_service(&self) -> crate::web::services::auth::AuthService {
        crate::web::services::auth::AuthService::new(
            self.database().clone(),
            self.config().web.clone().into()
        )
    }
    
    /// Get a new Discord service with user token and caching enabled
    pub fn discord_service(&self, token: impl Into<String>) -> crate::web::services::discord::DiscordService {
        crate::web::services::discord::DiscordService::new_with_cache(
            token,
            self.cache().clone()
        )
    }
    
    /// Get a new Discord service with bot token and caching enabled
    pub fn discord_bot_service(&self) -> crate::web::services::discord::DiscordService {
        // Use the bot token from config
        crate::web::services::discord::DiscordService::new_bot_with_cache(
            self.config().bot.bot_token.clone(),
            self.cache().clone(),
            self.config().bot.application_id.clone()
        )
    }
}
