// src/web/routes/word_detection.rs
//! Word detection rule management routes

use actix_web::{web, HttpResponse, Responder, HttpRequest, http::StatusCode};
use serde::Serialize;
use tracing::{error, info, debug};

use crate::error::Error;
use crate::web::state::WebAppState;
use crate::web::models::rule::{
    RuleInfo, CreateRuleRequest, UpdateRuleRequest, RuleResponse,
    TestRuleRequest, TestRuleResponse
};
use crate::web::services::RuleService;
use crate::web::services::GuildService;

/// Configure word detection routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/word_detection")
            .route("/rules/{guild_id}", web::get().to(list_rules))
            .route("/rules/{guild_id}/{rule_id}", web::get().to(get_rule))
            .route("/rules/{guild_id}", web::post().to(create_rule))
            .route("/rules/{guild_id}/{rule_id}", web::put().to(update_rule))
            .route("/rules/{guild_id}/{rule_id}", web::delete().to(delete_rule))
            .route("/test", web::post().to(test_rule))
    );
}

/// List all word detection rules for a guild
async fn list_rules(
    req: HttpRequest,
    path: web::Path<i64>,
    state: web::Data<WebAppState>,
) -> impl Responder {
    let guild_id = path.into_inner();
    
    info!("Retrieving all word detection rules for guild: {}", guild_id);
    
    // Create services
    let rule_service = RuleService::new(state.database().clone());
    let guild_service = GuildService::new(state.database().clone());
    
    // Check if bot is in this guild
    let bot_joined = match guild_service.is_bot_in_guild(guild_id).await {
        Ok(result) => result,
        Err(e) => {
            error!("Error checking if bot is in guild {}: {}", guild_id, e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    
    if !bot_joined {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Bot is not in this guild"
        }));
    }
    
    // Get all rules for this guild
    match rule_service.get_rules(guild_id).await {
        Ok(rules) => {
            HttpResponse::Ok().json(rules)
        },
        Err(e) => {
            error!("Error getting word detection rules: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to retrieve word detection rules: {}", e)
            }))
        }
    }
}

/// Get a specific word detection rule
async fn get_rule(
    req: HttpRequest,
    path: web::Path<(i64, i32)>,
    state: web::Data<WebAppState>,
) -> impl Responder {
    let (guild_id, rule_id) = path.into_inner();
    
    info!("Retrieving word detection rule {} for guild: {}", rule_id, guild_id);
    
    // Create services
    let rule_service = RuleService::new(state.database().clone());
    let guild_service = GuildService::new(state.database().clone());
    
    // Check if bot is in this guild
    let bot_joined = match guild_service.is_bot_in_guild(guild_id).await {
        Ok(result) => result,
        Err(e) => {
            error!("Error checking if bot is in guild {}: {}", guild_id, e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    
    if !bot_joined {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Bot is not in this guild"
        }));
    }
    
    // Get the specific rule
    match rule_service.get_rule(guild_id, rule_id).await {
        Ok(rule) => {
            HttpResponse::Ok().json(rule)
        },
        Err(e) => {
            match e {
                _ if e.to_string().contains("not found") => {
                    HttpResponse::NotFound().json(serde_json::json!({
                        "error": format!("Rule not found: {}", rule_id)
                    }))
                },
                _ => {
                    error!("Error getting word detection rule: {}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Failed to retrieve word detection rule: {}", e)
                    }))
                }
            }
        }
    }
}

/// Create a new word detection rule
async fn create_rule(
    req: HttpRequest,
    path: web::Path<i64>,
    rule_request: web::Json<CreateRuleRequest>,
    state: web::Data<WebAppState>,
) -> impl Responder {
    let guild_id = path.into_inner();
    
    info!("Creating new word detection rule for guild: {}", guild_id);
    
    // Create services
    let rule_service = RuleService::new(state.database().clone());
    let guild_service = GuildService::new(state.database().clone());
    
    // Check if bot is in this guild
    let bot_joined = match guild_service.is_bot_in_guild(guild_id).await {
        Ok(result) => result,
        Err(e) => {
            error!("Error checking if bot is in guild {}: {}", guild_id, e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    
    if !bot_joined {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Bot is not in this guild"
        }));
    }
    
    // Create the new rule
    match rule_service.create_rule(guild_id, &rule_request).await {
        Ok(rule_id) => {
            let response = RuleResponse {
                success: true,
                message: "Word detection rule created successfully".to_string(),
                rule_id: Some(rule_id),
            };
            
            HttpResponse::Created().json(response)
        },
        Err(e) => {
            error!("Error creating word detection rule: {}", e);
            
            let response = RuleResponse {
                success: false,
                message: format!("Failed to create word detection rule: {}", e),
                rule_id: None,
            };
            
            HttpResponse::InternalServerError().json(response)
        }
    }
}

/// Update an existing word detection rule
async fn update_rule(
    req: HttpRequest,
    path: web::Path<(i64, i32)>,
    rule_request: web::Json<UpdateRuleRequest>,
    state: web::Data<WebAppState>,
) -> impl Responder {
    let (guild_id, rule_id) = path.into_inner();
    
    info!("Updating word detection rule {} for guild: {}", rule_id, guild_id);
    
    // Create services
    let rule_service = RuleService::new(state.database().clone());
    let guild_service = GuildService::new(state.database().clone());
    
    // Check if bot is in this guild
    let bot_joined = match guild_service.is_bot_in_guild(guild_id).await {
        Ok(result) => result,
        Err(e) => {
            error!("Error checking if bot is in guild {}: {}", guild_id, e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    
    if !bot_joined {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Bot is not in this guild"
        }));
    }
    
    // Update the rule
    match rule_service.update_rule(guild_id, rule_id, &rule_request).await {
        Ok(()) => {
            let response = RuleResponse {
                success: true,
                message: "Word detection rule updated successfully".to_string(),
                rule_id: Some(rule_id),
            };
            
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            let response = RuleResponse {
                success: false,
                message: format!("Failed to update word detection rule: {}", e),
                rule_id: Some(rule_id),
            };
            
            match e {
                _ if e.to_string().contains("not found") => {
                    HttpResponse::NotFound().json(response)
                },
                _ => {
                    error!("Error updating word detection rule: {}", e);
                    HttpResponse::InternalServerError().json(response)
                }
            }
        }
    }
}

/// Delete a word detection rule
async fn delete_rule(
    req: HttpRequest,
    path: web::Path<(i64, i32)>,
    state: web::Data<WebAppState>,
) -> impl Responder {
    let (guild_id, rule_id) = path.into_inner();
    
    info!("Deleting word detection rule {} for guild: {}", rule_id, guild_id);
    
    // Create services
    let rule_service = RuleService::new(state.database().clone());
    let guild_service = GuildService::new(state.database().clone());
    
    // Check if bot is in this guild
    let bot_joined = match guild_service.is_bot_in_guild(guild_id).await {
        Ok(result) => result,
        Err(e) => {
            error!("Error checking if bot is in guild {}: {}", guild_id, e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    
    if !bot_joined {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Bot is not in this guild"
        }));
    }
    
    // Delete the rule
    match rule_service.delete_rule(guild_id, rule_id).await {
        Ok(()) => {
            let response = RuleResponse {
                success: true,
                message: "Word detection rule deleted successfully".to_string(),
                rule_id: Some(rule_id),
            };
            
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            let response = RuleResponse {
                success: false,
                message: format!("Failed to delete word detection rule: {}", e),
                rule_id: Some(rule_id),
            };
            
            match e {
                _ if e.to_string().contains("not found") => {
                    HttpResponse::NotFound().json(response)
                },
                _ => {
                    error!("Error deleting word detection rule: {}", e);
                    HttpResponse::InternalServerError().json(response)
                }
            }
        }
    }
}

/// Test a pattern against sample text
async fn test_rule(
    req: HttpRequest,
    test_request: web::Json<TestRuleRequest>,
    state: web::Data<WebAppState>,
) -> impl Responder {
    info!("Testing word detection pattern");
    
    // Create rule service
    let rule_service = RuleService::new(state.database().clone());
    
    // Test the pattern
    match rule_service.test_pattern(&test_request).await {
        Ok(result) => {
            HttpResponse::Ok().json(result)
        },
        Err(e) => {
            error!("Error testing word detection pattern: {}", e);
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Failed to test pattern: {}", e)
            }))
        }
    }
}
