// src/web/models/analytics.rs
//! Analytics data models for the web API

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Analytics event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    /// Event ID
    pub id: i32,
    /// Event type
    pub event_type: String,
    /// User ID (optional)
    pub user_id: Option<i64>,
    /// Guild ID (optional)
    pub guild_id: Option<i64>,
    /// Event data
    pub event_data: Option<HashMap<String, serde_json::Value>>,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
}

/// Analytics summary for a guild
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildAnalyticsSummary {
    /// Guild ID
    pub guild_id: i64,
    /// Time period (e.g., "day", "week", "month")
    pub period: String,
    /// Number of active users
    pub active_users: i32,
    /// Number of commands used
    pub commands_used: i32,
    /// Most used commands
    pub top_commands: Vec<CommandUsage>,
    /// Message activity
    pub message_count: i32,
    /// Event counts by type
    pub events_by_type: HashMap<String, i32>,
}

/// Command usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandUsage {
    /// Command ID
    pub command_id: String,
    /// Command name
    pub command_name: String,
    /// Number of times used
    pub count: i32,
}

/// User activity summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivitySummary {
    /// User ID
    pub user_id: i64,
    /// User name
    pub username: String,
    /// Time period (e.g., "day", "week", "month")
    pub period: String,
    /// Commands used
    pub commands_used: Vec<CommandUsage>,
    /// Guilds active in
    pub active_guilds: Vec<GuildActivity>,
    /// Total number of messages sent
    pub message_count: i32,
    /// First activity timestamp in the period
    pub first_activity: DateTime<Utc>,
    /// Last activity timestamp in the period
    pub last_activity: DateTime<Utc>,
}

/// Guild activity for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildActivity {
    /// Guild ID
    pub guild_id: i64,
    /// Guild name
    pub guild_name: String,
    /// Number of messages sent
    pub message_count: i32,
    /// Number of commands used
    pub commands_used: i32,
}

/// Request to log a new analytics event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEventRequest {
    /// Event type
    pub event_type: String,
    /// User ID (optional)
    pub user_id: Option<i64>,
    /// Guild ID (optional)
    pub guild_id: Option<i64>,
    /// Event data
    pub event_data: Option<HashMap<String, serde_json::Value>>,
}

/// Analytics query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsQueryParams {
    /// Start date
    pub start_date: Option<DateTime<Utc>>,
    /// End date
    pub end_date: Option<DateTime<Utc>>,
    /// Guild ID
    pub guild_id: Option<i64>,
    /// User ID
    pub user_id: Option<i64>,
    /// Event type
    pub event_type: Option<String>,
    /// Time period (e.g., "day", "week", "month")
    pub period: Option<String>,
    /// Maximum results to return
    pub limit: Option<i32>,
}

/// Response for analytics operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsResponse {
    /// Whether the operation was successful
    pub success: bool,
    /// Message describing the result
    pub message: String,
    /// Event ID for log operations
    pub event_id: Option<i32>,
}
