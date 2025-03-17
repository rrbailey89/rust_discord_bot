// src/web/models/command.rs
//! Command data models for the web API

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Command information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandInfo {
    /// Command ID
    pub id: String,
    /// Command name
    pub name: String,
    /// Command description
    pub description: String,
    /// Command category
    pub category: String,
    /// Whether the command requires admin permissions
    pub requires_admin: bool,
    /// Whether the command has configuration options
    pub has_config: bool,
}

/// Detailed command information with configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDetails {
    /// Command ID
    pub id: String,
    /// Command name
    pub name: String,
    /// Command description
    pub description: String,
    /// Command category
    pub category: String,
    /// Whether the command requires admin permissions
    pub requires_admin: bool,
    /// Command configuration schema
    pub config_schema: Option<ConfigSchema>,
    /// Current command configuration
    pub current_config: Option<HashMap<String, serde_json::Value>>,
}

/// Command configuration schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSchema {
    /// Configuration options
    pub options: Vec<ConfigOption>,
}

/// Command configuration option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOption {
    /// Option name
    pub name: String,
    /// Option description
    pub description: String,
    /// Option type (string, number, boolean, etc.)
    pub option_type: String,
    /// Whether the option is required
    pub required: bool,
    /// Default value
    pub default: Option<serde_json::Value>,
    /// Possible enum values (for select options)
    pub enum_values: Option<Vec<EnumValue>>,
}

/// Enum value for select options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumValue {
    /// Value
    pub value: String,
    /// Display name
    pub label: String,
}

/// Command settings for a guild
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSettings {
    /// Command ID
    pub command_id: String,
    /// Whether the command is enabled
    pub enabled: bool,
    /// Command configuration
    pub settings: Option<HashMap<String, serde_json::Value>>,
}

/// Request to update command settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCommandSettingsRequest {
    /// Whether the command is enabled
    pub enabled: Option<bool>,
    /// Command configuration
    pub settings: Option<HashMap<String, serde_json::Value>>,
}

/// Response for command operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    /// Whether the operation was successful
    pub success: bool,
    /// Message describing the result
    pub message: String,
}
