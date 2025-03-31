// src/web/services/settings.rs
//! Settings service for database operations related to settings

use std::collections::HashMap;
use crate::error::Error;
use crate::services::database::DatabaseService;
use crate::web::models::settings::{
    GlobalSettings, UserPreferences, 
    UpdateGlobalSettingsRequest, UpdateUserPreferencesRequest
};
use tracing::info;
use serde_json::Value;

/// Settings service for database operations
pub struct SettingsService {
    /// Database service
    db: DatabaseService,
}

impl SettingsService {
    /// Create a new settings service
    pub fn new(db: DatabaseService) -> Self {
        Self { db }
    }

    /// Get global bot settings
    pub async fn get_global_settings(&self) -> Result<GlobalSettings, Error> {
        let client = self.db.get_client().await?;

        // Query global settings
        let row = client
            .query_one(
                "SELECT 
                    default_prefix, 
                    analytics_enabled, 
                    max_message_fetches_per_day, 
                    config
                FROM global_settings
                WHERE id = 1", // We use a single row for global settings
                &[],
            )
            .await?;

        // Parse the settings
        let settings = GlobalSettings {
            default_prefix: row.get(0),
            analytics_enabled: row.get(1),
            max_message_fetches_per_day: row.get(2),
            config: row.get::<_, Option<Value>>(3)
                .map(|v| serde_json::from_value(v).unwrap_or_default())
                .unwrap_or_default(),
        };

        Ok(settings)
    }

    /// Update global bot settings
    pub async fn update_global_settings(
        &self,
        request: &UpdateGlobalSettingsRequest,
    ) -> Result<(), Error> {
        let client = self.db.get_client().await?;

        // Get current settings
        let current = self.get_global_settings().await?;

        // Determine what to update
        let default_prefix = request.default_prefix.as_ref().unwrap_or(&current.default_prefix);
        let analytics_enabled = request.analytics_enabled.unwrap_or(current.analytics_enabled);
        let max_message_fetches = request.max_message_fetches_per_day.unwrap_or(current.max_message_fetches_per_day);
        
        // Handle config, which is a HashMap
        let config = match &request.config {
            Some(new_config) => {
                let json = serde_json::to_value(new_config)
                    .map_err(|e| Error::Unknown(format!("Failed to serialize config: {}", e)))?;
                json
            },
            None => {
                serde_json::to_value(current.config)
                    .map_err(|e| Error::Unknown(format!("Failed to serialize config: {}", e)))?
            },
        };

        // Update settings
        client
            .execute(
                "INSERT INTO global_settings 
                (id, default_prefix, analytics_enabled, max_message_fetches_per_day, config)
                VALUES (1, $1, $2, $3, $4)
                ON CONFLICT (id) DO UPDATE SET
                    default_prefix = EXCLUDED.default_prefix,
                    analytics_enabled = EXCLUDED.analytics_enabled,
                    max_message_fetches_per_day = EXCLUDED.max_message_fetches_per_day,
                    config = EXCLUDED.config",
                &[default_prefix, &analytics_enabled, &max_message_fetches, &config],
            )
            .await?;

        info!("Updated global settings");
        Ok(())
    }

    /// Get user preferences
    pub async fn get_user_preferences(&self, user_id: i64) -> Result<UserPreferences, Error> {
        let client = self.db.get_client().await?;

        // Query user preferences
        let row = client
            .query_opt(
                "SELECT 
                    user_id, 
                    theme, 
                    language, 
                    notifications_enabled, 
                    preferences
                FROM user_preferences
                WHERE user_id = $1",
                &[&user_id],
            )
            .await?;

        // If no preferences exist yet, return defaults
        match row {
            Some(row) => {
                // Parse the preferences
                let prefs = UserPreferences {
                    user_id: row.get(0),
                    theme: row.get(1),
                    language: row.get(2),
                    notifications_enabled: row.get(3),
                    preferences: row.get::<_, Option<Value>>(4)
                        .map(|v| serde_json::from_value(v).unwrap_or_default())
                        .unwrap_or_default(),
                };

                Ok(prefs)
            },
            None => {
                // Return default preferences
                Ok(UserPreferences {
                    user_id,
                    theme: "light".to_string(),
                    language: "en".to_string(),
                    notifications_enabled: true,
                    preferences: HashMap::new(),
                })
            }
        }
    }

    /// Update user preferences
    pub async fn update_user_preferences(
        &self,
        user_id: i64,
        request: &UpdateUserPreferencesRequest,
    ) -> Result<(), Error> {
        let client = self.db.get_client().await?;

        // Get current preferences or defaults
        let current = self.get_user_preferences(user_id).await?;

        // Determine what to update
        let theme = request.theme.as_ref().unwrap_or(&current.theme);
        let language = request.language.as_ref().unwrap_or(&current.language);
        let notifications_enabled = request.notifications_enabled.unwrap_or(current.notifications_enabled);
        
        // Handle preferences, which is a HashMap
        let preferences = match &request.preferences {
            Some(new_prefs) => {
                let json = serde_json::to_value(new_prefs)
                    .map_err(|e| Error::Unknown(format!("Failed to serialize preferences: {}", e)))?;
                json
            },
            None => {
                serde_json::to_value(current.preferences)
                    .map_err(|e| Error::Unknown(format!("Failed to serialize preferences: {}", e)))?
            },
        };

        // Update preferences
        client
            .execute(
                "INSERT INTO user_preferences 
                (user_id, theme, language, notifications_enabled, preferences)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (user_id) DO UPDATE SET
                    theme = EXCLUDED.theme,
                    language = EXCLUDED.language,
                    notifications_enabled = EXCLUDED.notifications_enabled,
                    preferences = EXCLUDED.preferences",
                &[&user_id, theme, language, &notifications_enabled, &preferences],
            )
            .await?;

        info!("Updated preferences for user {}", user_id);
        Ok(())
    }
}
