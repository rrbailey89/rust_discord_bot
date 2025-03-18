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
    
    /// Get a new authentication service
    pub fn auth_service(&self) -> crate::web::services::auth::AuthService {
        crate::web::services::auth::AuthService::new(
            self.database().clone(),
            self.config().web.clone().into()
        )
    }
}
