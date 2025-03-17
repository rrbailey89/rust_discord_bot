// src/web/routes/commands.rs
//! Command management routes

use actix_web::{web, HttpResponse, Responder, HttpRequest, http::StatusCode};
use serde::Serialize;
use tracing::{error, info, debug};

use crate::error::Error;
use crate::web::state::WebAppState;
use crate::web::models::command::{
    CommandInfo, CommandDetails, CommandSettings, UpdateCommandSettingsRequest, CommandResponse
};
use crate::web::services::CommandService;
use crate::web::services::GuildService;

/// Configure command routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/commands")
            .route("", web::get().to(list_commands))
            .route("/{command_id}", web::get().to(get_command_details))
            .route("/{command_id}/settings", web::get().to(get_command_settings))
            .route("/{command_id}/settings", web::put().to(update_command_settings))
            .route("/guild/{guild_id}", web::get().to(list_guild_commands))
    );
}

/// List all available commands
async fn list_commands(
    req: HttpRequest,
    state: web::Data<WebAppState>,
) -> impl Responder {
    // Create command service
    let command_service = CommandService::new(state.database().clone());
    
    // Get all available commands
    match command_service.get_available_commands().await {
        Ok(commands) => {
            HttpResponse::Ok().json(commands)
        },
        Err(e) => {
            error!("Error getting available commands: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to retrieve commands"
            }))
        }
    }
}

/// Get detailed info about a specific command
async fn get_command_details(
    req: HttpRequest,
    path: web::Path<(String,)>,
    state: web::Data<WebAppState>,
) -> impl Responder {
    // For now, use a default guild ID until auth is fixed
    let guild_id = 123456789i64;
    let command_id = path.0.clone();
    
    info!("Retrieving details for command: {}", command_id);
    
    // Create command service
    let command_service = CommandService::new(state.database().clone());
    
    // Get command details
    match command_service.get_command_details(guild_id, &command_id).await {
        Ok(details) => {
            HttpResponse::Ok().json(details)
        },
        Err(e) => {
            error!("Error getting command details: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to retrieve command details: {}", e)
            }))
        }
    }
}

/// Get command settings for a guild
async fn get_command_settings(
    req: HttpRequest,
    path: web::Path<(String,)>,
    query: web::Query<GuildQuery>,
    state: web::Data<WebAppState>,
) -> impl Responder {
    let command_id = path.0.clone();
    
    // Use query param for guild_id or default
    let guild_id = query.guild_id.unwrap_or(123456789);
    
    info!("Retrieving settings for command: {} in guild: {}", command_id, guild_id);
    
    // Create command service
    let command_service = CommandService::new(state.database().clone());
    
    // Get command settings
    match command_service.get_command_settings(guild_id, &command_id).await {
        Ok(settings) => {
            HttpResponse::Ok().json(settings)
        },
        Err(e) => {
            error!("Error getting command settings: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to retrieve command settings: {}", e)
            }))
        }
    }
}

/// Update command settings for a guild
async fn update_command_settings(
    req: HttpRequest,
    path: web::Path<(String,)>,
    query: web::Query<GuildQuery>,
    settings: web::Json<UpdateCommandSettingsRequest>,
    state: web::Data<WebAppState>,
) -> impl Responder {
    let command_id = path.0.clone();
    
    // Use query param for guild_id or default
    let guild_id = query.guild_id.unwrap_or(123456789);
    
    info!("Updating settings for command: {} in guild: {}", command_id, guild_id);
    
    // Create command service
    let command_service = CommandService::new(state.database().clone());
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
    
    // Update command settings
    match command_service.update_command_settings(guild_id, &command_id, &settings).await {
        Ok(()) => {
            let response = CommandResponse {
                success: true,
                message: "Command settings updated successfully".to_string(),
            };
            
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            error!("Error updating command settings: {}", e);
            
            let response = CommandResponse {
                success: false,
                message: format!("Failed to update command settings: {}", e),
            };
            
            HttpResponse::InternalServerError().json(response)
        }
    }
}

/// List all commands with their settings for a guild
async fn list_guild_commands(
    req: HttpRequest,
    path: web::Path<i64>,
    state: web::Data<WebAppState>,
) -> impl Responder {
    let guild_id = path.into_inner();
    
    info!("Retrieving all commands for guild: {}", guild_id);
    
    // Create services
    let command_service = CommandService::new(state.database().clone());
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
    
    // Get all command settings for this guild
    match command_service.get_all_command_settings(guild_id).await {
        Ok(settings) => {
            HttpResponse::Ok().json(settings)
        },
        Err(e) => {
            error!("Error getting all command settings: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to retrieve command settings: {}", e)
            }))
        }
    }
}

/// Query parameters for guild requests
#[derive(serde::Deserialize)]
struct GuildQuery {
    guild_id: Option<i64>,
}
