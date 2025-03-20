// src/web/models/guild.rs
//! Guild data models for the web API

use serde::{Deserialize, Serialize};

/// Guild information returned by the API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildInfo {
    /// Guild ID
    pub id: String,
    /// Guild name
    pub name: String,
    /// Guild icon URL
    pub icon_url: Option<String>,
    /// Whether the user is the owner of the guild
    pub owner: bool,
    /// User's permissions in the guild
    pub permissions: u64,
    /// Whether the bot is in this guild
    #[serde(rename = "botJoined")]
    pub bot_joined: bool,
    /// Number of members in the guild
    #[serde(rename = "memberCount")]
    pub member_count: Option<i32>,
}

/// Detailed guild information including channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildDetails {
    /// Guild ID
    pub id: String,
    /// Guild name
    pub name: String,
    /// Guild icon URL
    pub icon_url: Option<String>,
    /// Whether the user is the owner of the guild
    pub owner: bool,
    /// User's permissions in the guild
    pub permissions: u64,
    /// Whether the bot is in this guild
    #[serde(rename = "botJoined")]
    pub bot_joined: bool,
    /// Number of members in the guild
    #[serde(rename = "memberCount")]
    pub member_count: Option<i32>,
    /// List of channels in the guild
    pub channels: Vec<ChannelInfo>,
}

/// Channel information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelInfo {
    /// Channel ID
    pub id: String,
    /// Channel name
    pub name: String,
    /// Channel type (0 = text, 2 = voice, etc.)
    pub channel_type: i32,
    /// Channel position
    pub position: i32,
    /// Channel topic
    pub topic: Option<String>,
}

/// Guild settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildSettings {
    /// Whether emoji reactions are enabled
    pub emoji_reactions_enabled: bool,
    /// Channel ID for level up notifications
    pub level_up_channel_id: Option<String>,
    /// Channel ID for warning notifications
    pub warn_channel_id: Option<String>,
    /// URL rule setting for the guild
    pub url_rule: Option<String>,
    /// Channel ID for delete log
    pub delete_log_channel_id: Option<String>,
    /// Channel ID for reaction log
    pub reaction_log_channel_id: Option<String>,
}

/// Request to update guild settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGuildSettingsRequest {
    /// Whether emoji reactions are enabled
    pub emoji_reactions_enabled: Option<bool>,
    /// Channel ID for level up notifications
    pub level_up_channel_id: Option<String>,
    /// Channel ID for warning notifications
    pub warn_channel_id: Option<String>,
    /// URL rule setting for the guild
    pub url_rule: Option<String>,
    /// Channel ID for delete log
    pub delete_log_channel_id: Option<String>,
    /// Channel ID for reaction log
    pub reaction_log_channel_id: Option<String>,
}

/// Response for guild operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildResponse {
    /// Whether the operation was successful
    pub success: bool,
    /// Message describing the result
    pub message: String,
}
