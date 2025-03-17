// src/web/services/rule.rs
//! Word detection rule service for database operations

use std::collections::HashMap;
use crate::error::Error;
use crate::services::database::DatabaseService;
use crate::web::models::rule::{
    RuleInfo, CreateRuleRequest, UpdateRuleRequest, RuleResponse,
    TestRuleRequest, TestRuleResponse, RuleAction
};
use chrono::{DateTime, Utc};
use regex::Regex;
use tracing::{debug, error, info};
use serde_json::Value;

/// Word detection rule service
pub struct RuleService {
    /// Database service
    db: DatabaseService,
}

impl RuleService {
    /// Create a new rule service
    pub fn new(db: DatabaseService) -> Self {
        Self { db }
    }

    /// Get all word detection rules for a guild
    pub async fn get_rules(&self, guild_id: i64) -> Result<Vec<RuleInfo>, Error> {
        let client = self.db.get_client().await?;

        // Query all rules for this guild
        let rows = client
            .query(
                "SELECT 
                    id, 
                    guild_id, 
                    pattern, 
                    action, 
                    action_params,
                    created_at,
                    updated_at
                FROM word_detection_rules
                WHERE guild_id = $1
                ORDER BY id ASC",
                &[&guild_id],
            )
            .await?;

        // Map rows to RuleInfo objects
        let rules = rows
            .iter()
            .map(|row| RuleInfo {
                id: row.get(0),
                guild_id: row.get(1),
                pattern: row.get(2),
                action: row.get(3),
                action_params: row.get::<_, Option<Value>>(4).map(|v| {
                    serde_json::from_value(v).unwrap_or_else(|_| HashMap::new())
                }),
                created_at: row.get(5),
                updated_at: row.get(6),
            })
            .collect();

        Ok(rules)
    }

    /// Get a specific word detection rule
    pub async fn get_rule(&self, guild_id: i64, rule_id: i32) -> Result<RuleInfo, Error> {
        let client = self.db.get_client().await?;

        // Query the specific rule
        let row = client
            .query_opt(
                "SELECT 
                    id, 
                    guild_id, 
                    pattern, 
                    action, 
                    action_params,
                    created_at,
                    updated_at
                FROM word_detection_rules
                WHERE guild_id = $1 AND id = $2",
                &[&guild_id, &rule_id],
            )
            .await?;

        // If rule not found, return error
        match row {
            Some(row) => {
                Ok(RuleInfo {
                    id: row.get(0),
                    guild_id: row.get(1),
                    pattern: row.get(2),
                    action: row.get(3),
                    action_params: row.get::<_, Option<Value>>(4).map(|v| {
                        serde_json::from_value(v).unwrap_or_else(|_| HashMap::new())
                    }),
                    created_at: row.get(5),
                    updated_at: row.get(6),
                })
            },
            None => Err(Error::Unknown(format!("Rule not found: {}", rule_id))),
        }
    }

    /// Create a new word detection rule
    pub async fn create_rule(
        &self,
        guild_id: i64,
        request: &CreateRuleRequest,
    ) -> Result<i32, Error> {
        // Validate the rule pattern
        self.validate_pattern(&request.pattern)?;
        
        // Validate the action
        self.validate_action(&request.action)?;
        
        let client = self.db.get_client().await?;

        // Convert action_params to JSON if present
        let action_params = match &request.action_params {
            Some(params) => {
                let json = serde_json::to_value(params)
                    .map_err(|e| Error::Unknown(format!("Failed to serialize action params: {}", e)))?;
                Some(json)
            },
            None => None,
        };

        // Insert the new rule
        let row = client
            .query_one(
                "INSERT INTO word_detection_rules
                (guild_id, pattern, action, action_params)
                VALUES ($1, $2, $3, $4)
                RETURNING id",
                &[&guild_id, &request.pattern, &request.action, &action_params],
            )
            .await?;

        let rule_id: i32 = row.get(0);
        info!("Created word detection rule: {} for guild {}", rule_id, guild_id);
        
        Ok(rule_id)
    }

    /// Update an existing word detection rule
    pub async fn update_rule(
        &self,
        guild_id: i64,
        rule_id: i32,
        request: &UpdateRuleRequest,
    ) -> Result<(), Error> {
        // Get the existing rule to update
        let existing = self.get_rule(guild_id, rule_id).await?;
        
        // Validate pattern if provided
        if let Some(pattern) = &request.pattern {
            self.validate_pattern(pattern)?;
        }
        
        // Validate action if provided
        if let Some(action) = &request.action {
            self.validate_action(action)?;
        }
        
        let client = self.db.get_client().await?;

        // Determine what to update
        let pattern = request.pattern.as_ref().unwrap_or(&existing.pattern);
        let action = request.action.as_ref().unwrap_or(&existing.action);
        
        // Convert action_params to JSON if present or use existing
        let action_params = match &request.action_params {
            Some(params) => {
                let json = serde_json::to_value(params)
                    .map_err(|e| Error::Unknown(format!("Failed to serialize action params: {}", e)))?;
                Some(json)
            },
            None => {
                // Use existing params if available
                existing.action_params.map(|p| {
                    serde_json::to_value(p)
                        .unwrap_or(serde_json::Value::Null)
                })
            },
        };

        // Update the rule
        let result = client
            .execute(
                "UPDATE word_detection_rules
                SET pattern = $3, action = $4, action_params = $5, updated_at = NOW()
                WHERE guild_id = $1 AND id = $2",
                &[&guild_id, &rule_id, pattern, action, &action_params],
            )
            .await?;

        if result == 0 {
            return Err(Error::Unknown(format!("Rule not found: {}", rule_id)));
        }

        info!("Updated word detection rule: {} for guild {}", rule_id, guild_id);
        Ok(())
    }

    /// Delete a word detection rule
    pub async fn delete_rule(
        &self,
        guild_id: i64,
        rule_id: i32,
    ) -> Result<(), Error> {
        let client = self.db.get_client().await?;

        // Delete the rule
        let result = client
            .execute(
                "DELETE FROM word_detection_rules
                WHERE guild_id = $1 AND id = $2",
                &[&guild_id, &rule_id],
            )
            .await?;

        if result == 0 {
            return Err(Error::Unknown(format!("Rule not found: {}", rule_id)));
        }

        info!("Deleted word detection rule: {} for guild {}", rule_id, guild_id);
        Ok(())
    }

    /// Test a pattern against sample text
    pub async fn test_pattern(
        &self,
        request: &TestRuleRequest,
    ) -> Result<TestRuleResponse, Error> {
        // Validate the pattern
        self.validate_pattern(&request.pattern)?;
        
        // Compile the regex
        let regex = match Regex::new(&request.pattern) {
            Ok(re) => re,
            Err(e) => {
                return Err(Error::Unknown(format!("Invalid regex pattern: {}", e)));
            }
        };
        
        // Find all matches
        let matches: Vec<String> = regex
            .find_iter(&request.sample_text)
            .map(|m| m.as_str().to_string())
            .collect();
        
        // Determine if there were any matches
        let matches_found = !matches.is_empty();
        
        // Create explanation
        let explanation = if matches_found {
            format!("Found {} match(es) in the text", matches.len())
        } else {
            "No matches found in the text".to_string()
        };
        
        Ok(TestRuleResponse {
            matches: matches_found,
            matched_text: matches,
            explanation,
        })
    }
    
    /// Validate a regex pattern
    fn validate_pattern(&self, pattern: &str) -> Result<(), Error> {
        match Regex::new(pattern) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Unknown(format!("Invalid regex pattern: {}", e))),
        }
    }
    
    /// Validate an action
    fn validate_action(&self, action: &str) -> Result<(), Error> {
        match RuleAction::from_str(action) {
            Some(_) => Ok(()),
            None => Err(Error::Unknown(format!("Invalid action: {}", action))),
        }
    }
}
