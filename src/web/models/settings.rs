// src/web/models/settings.rs
//! Settings data models for the web API

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Global bot settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSettings {
    /// Default prefix for commands
    pub default_prefix: String,
    /// Whether analytics collection is enabled
    pub analytics_enabled: bool,
    /// Maximum number of message fetches per day
    pub max_message_fetches_per_day: i32,
    /// Additional configuration options
    pub config: HashMap<String, serde_json::Value>,
}

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// User ID
    pub user_id: i64,
    /// Theme preference
    pub theme: String,
    /// Language preference
    pub language: String,
    /// Notification settings
    pub notifications_enabled: bool,
    /// Additional preferences
    pub preferences: HashMap<String, serde_json::Value>,
}

/// Request to update global settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGlobalSettingsRequest {
    /// Default prefix for commands (optional)
    pub default_prefix: Option<String>,
    /// Whether analytics collection is enabled (optional)
    pub analytics_enabled: Option<bool>,
    /// Maximum number of message fetches per day (optional)
    pub max_message_fetches_per_day: Option<i32>,
    /// Additional configuration options (optional)
    pub config: Option<HashMap<String, serde_json::Value>>,
}

/// Request to update user preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserPreferencesRequest {
    /// Theme preference (optional)
    pub theme: Option<String>,
    /// Language preference (optional)
    pub language: Option<String>,
    /// Notification settings (optional)
    pub notifications_enabled: Option<bool>,
    /// Additional preferences (optional)
    pub preferences: Option<HashMap<String, serde_json::Value>>,
}

/// Response for settings operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsResponse {
    /// Whether the operation was successful
    pub success: bool,
    /// Message describing the result
    pub message: String,
}
