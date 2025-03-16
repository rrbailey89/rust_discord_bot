// src/web/routes/word_detection.rs
//! Word detection rules routes

use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::web::state::WebAppState;
use crate::web::middleware::auth::Claims;

/// Word detection rule
#[derive(Serialize, Deserialize)]
pub struct WordDetectionRule {
    id: Option<i32>,
    guild_id: String,
    pattern: String,
    action: String,
    action_params: Option<serde_json::Value>,
    created_at: Option<String>,
    updated_at: Option<String>,
}

/// Word detection rule test request
#[derive(Deserialize)]
pub struct TestRuleRequest {
    text: String,
    rule_id: Option<i32>,
    pattern: Option<String>,
}

/// Word detection rule test response
#[derive(Serialize)]
pub struct TestRuleResponse {
    matches: bool,
    match_details: Option<Vec<String>>,
}

/// Configure word detection routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/word-detection")
            .route("/{guild_id}", web::get().to(list_rules))
            .route("/{guild_id}", web::post().to(create_rule))
            .route("/{guild_id}/{rule_id}", web::get().to(get_rule))
            .route("/{guild_id}/{rule_id}", web::put().to(update_rule))
            .route("/{guild_id}/{rule_id}", web::delete().to(delete_rule))
            .route("/{guild_id}/test", web::post().to(test_rule))
    );
}

/// List all word detection rules for a guild
async fn list_rules(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<WebAppState>
) -> impl Responder {
    // TODO: Implement rule listing
    // This is a placeholder implementation
    
    let guild_id = path.into_inner();
    info!("Listing word detection rules for guild: {}", guild_id);
    
    HttpResponse::Ok().json(serde_json::json!([
        {
            "id": 1,
            "guild_id": guild_id,
            "pattern": "\\b(bad|offensive)\\s+word\\b",
            "action": "delete",
            "action_params": null,
            "created_at": "2025-03-15T14:30:00Z",
            "updated_at": "2025-03-15T14:30:00Z"
        },
        {
            "id": 2,
            "guild_id": guild_id,
            "pattern": "\\bspam\\s+link\\b",
            "action": "warn",
            "action_params": {
                "message": "Please don't post spam links"
            },
            "created_at": "2025-03-16T10:15:00Z",
            "updated_at": "2025-03-16T10:15:00Z"
        }
    ]))
}

/// Create a new word detection rule
async fn create_rule(
    req: HttpRequest,
    path: web::Path<String>,
    rule: web::Json<WordDetectionRule>,
    state: web::Data<WebAppState>
) -> impl Responder {
    // TODO: Implement rule creation
    // This is a placeholder implementation
    
    let guild_id = path.into_inner();
    info!("Creating word detection rule for guild: {}", guild_id);
    
    // Validate that the rule pattern is a valid regex
    match regex::Regex::new(&rule.pattern) {
        Ok(_) => {
            // Return the created rule with an assigned ID
            let mut created_rule = rule.into_inner();
            created_rule.id = Some(3); // In a real implementation, this would be assigned by the database
            created_rule.guild_id = guild_id;
            created_rule.created_at = Some("2025-03-16T14:52:00Z".to_string());
            created_rule.updated_at = Some("2025-03-16T14:52:00Z".to_string());
            
            HttpResponse::Created().json(created_rule)
        },
        Err(e) => {
            error!("Invalid regex pattern: {}", e);
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid regex pattern",
                "details": e.to_string()
            }))
        }
    }
}

/// Get a specific word detection rule
async fn get_rule(
    req: HttpRequest,
    path: web::Path<(String, i32)>,
    state: web::Data<WebAppState>
) -> impl Responder {
    // TODO: Implement rule retrieval
    // This is a placeholder implementation
    
    let (guild_id, rule_id) = path.into_inner();
    info!("Retrieving word detection rule {} for guild: {}", rule_id, guild_id);
    
    HttpResponse::Ok().json(serde_json::json!({
        "id": rule_id,
        "guild_id": guild_id,
        "pattern": "\\b(bad|offensive)\\s+word\\b",
        "action": "delete",
        "action_params": null,
        "created_at": "2025-03-15T14:30:00Z",
        "updated_at": "2025-03-15T14:30:00Z"
    }))
}

/// Update a word detection rule
async fn update_rule(
    req: HttpRequest,
    path: web::Path<(String, i32)>,
    rule: web::Json<WordDetectionRule>,
    state: web::Data<WebAppState>
) -> impl Responder {
    // TODO: Implement rule update
    // This is a placeholder implementation
    
    let (guild_id, rule_id) = path.into_inner();
    info!("Updating word detection rule {} for guild: {}", rule_id, guild_id);
    
    // Validate that the rule pattern is a valid regex
    match regex::Regex::new(&rule.pattern) {
        Ok(_) => {
            // Return the updated rule
            let mut updated_rule = rule.into_inner();
            updated_rule.id = Some(rule_id);
            updated_rule.guild_id = guild_id;
            updated_rule.updated_at = Some("2025-03-16T14:53:00Z".to_string());
            
            HttpResponse::Ok().json(updated_rule)
        },
        Err(e) => {
            error!("Invalid regex pattern: {}", e);
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid regex pattern",
                "details": e.to_string()
            }))
        }
    }
}

/// Delete a word detection rule
async fn delete_rule(
    req: HttpRequest,
    path: web::Path<(String, i32)>,
    state: web::Data<WebAppState>
) -> impl Responder {
    // TODO: Implement rule deletion
    // This is a placeholder implementation
    
    let (guild_id, rule_id) = path.into_inner();
    info!("Deleting word detection rule {} for guild: {}", rule_id, guild_id);
    
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Rule deleted successfully"
    }))
}

/// Test a word detection rule against text
async fn test_rule(
    req: HttpRequest,
    path: web::Path<String>,
    test_request: web::Json<TestRuleRequest>,
    state: web::Data<WebAppState>
) -> impl Responder {
    // TODO: Implement rule testing
    // This is a placeholder implementation
    
    let guild_id = path.into_inner();
    let request = test_request.into_inner();
    
    // If rule_id is provided, look up the rule pattern
    let pattern = match request.rule_id {
        Some(rule_id) => {
            info!("Testing rule {} for guild: {}", rule_id, guild_id);
            // In a real implementation, we would look up the pattern from the database
            "\\b(bad|offensive)\\s+word\\b".to_string()
        },
        None => {
            match request.pattern {
                Some(p) => {
                    info!("Testing custom pattern for guild: {}", guild_id);
                    p
                },
                None => {
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "error": "Either rule_id or pattern must be provided"
                    }));
                }
            }
        }
    };
    
    // Test the pattern against the text
    match regex::Regex::new(&pattern) {
        Ok(regex) => {
            let matches = regex.is_match(&request.text);
            let match_details = if matches {
                Some(
                    regex.find_iter(&request.text)
                        .map(|m| m.as_str().to_string())
                        .collect()
                )
            } else {
                None
            };
            
            HttpResponse::Ok().json(TestRuleResponse {
                matches,
                match_details,
            })
        },
        Err(e) => {
            error!("Invalid regex pattern: {}", e);
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid regex pattern",
                "details": e.to_string()
            }))
        }
    }
}
