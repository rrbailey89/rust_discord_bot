// src/web/models/rule.rs
//! Word detection rule data models for the web API

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Word detection rule information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleInfo {
    /// Rule ID
    pub id: i32,
    /// Guild ID
    pub guild_id: i64,
    /// Pattern to match
    pub pattern: String,
    /// Action to take when pattern matches
    pub action: String,
    /// Additional parameters for the action
    pub action_params: Option<HashMap<String, serde_json::Value>>,
    /// When the rule was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When the rule was last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Request to create a new word detection rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRuleRequest {
    /// Pattern to match
    pub pattern: String,
    /// Action to take when pattern matches
    pub action: String,
    /// Additional parameters for the action
    pub action_params: Option<HashMap<String, serde_json::Value>>,
}

/// Request to update an existing word detection rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRuleRequest {
    /// Pattern to match (optional)
    pub pattern: Option<String>,
    /// Action to take when pattern matches (optional)
    pub action: Option<String>,
    /// Additional parameters for the action (optional)
    pub action_params: Option<HashMap<String, serde_json::Value>>,
}

/// Response for rule operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleResponse {
    /// Whether the operation was successful
    pub success: bool,
    /// Message describing the result
    pub message: String,
    /// Rule ID for create operations
    pub rule_id: Option<i32>,
}

/// Request to test a pattern against sample text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRuleRequest {
    /// Pattern to test
    pub pattern: String,
    /// Sample text to test against
    pub sample_text: String,
}

/// Response for rule test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRuleResponse {
    /// Whether the pattern matched
    pub matches: bool,
    /// List of matched portions of the text
    pub matched_text: Vec<String>,
    /// Explanation of the match
    pub explanation: String,
}

/// Supported rule actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    /// Delete the message
    Delete,
    /// Warn the user
    Warn,
    /// Delete the message and warn the user
    DeleteAndWarn,
    /// Timeout the user
    Timeout,
    /// Log the message
    Log,
}

impl RuleAction {
    /// Convert string to RuleAction
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "delete" => Some(RuleAction::Delete),
            "warn" => Some(RuleAction::Warn),
            "deleteandwarn" => Some(RuleAction::DeleteAndWarn),
            "timeout" => Some(RuleAction::Timeout),
            "log" => Some(RuleAction::Log),
            _ => None,
        }
    }
    
    /// Convert RuleAction to string
    pub fn as_str(&self) -> &'static str {
        match self {
            RuleAction::Delete => "delete",
            RuleAction::Warn => "warn",
            RuleAction::DeleteAndWarn => "deleteandwarn",
            RuleAction::Timeout => "timeout",
            RuleAction::Log => "log",
        }
    }
}
