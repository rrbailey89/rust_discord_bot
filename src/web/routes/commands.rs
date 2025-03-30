// src/web/routes/commands.rs
//! Command management routes

use actix_web::{web, HttpResponse, Responder, HttpRequest, http::StatusCode, HttpMessage};
use serde::Serialize;
use tracing::{error, info, debug, warn};

use crate::error::Error;
use crate::web::state::WebAppState;
use crate::web::middleware::auth::Claims;
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
    // Get authenticated user from request extensions
    let extensions = req.extensions();
    let claims = match extensions.get::<Claims>() {
        Some(claims) => claims,
        None => {
            error!("No authentication claims found in request");
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Not authenticated"
            }));
        }
    };
    
    debug!("Processing request with claims: user_id={}, username={}", 
           claims.sub, claims.user.username);
    
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
    query: web::Query<GuildQuery>,
    state: web::Data<WebAppState>,
) -> impl Responder {
    // Get authenticated user from request extensions
    let extensions = req.extensions();
    let claims = match extensions.get::<Claims>() {
        Some(claims) => claims,
        None => {
            error!("No authentication claims found in request");
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Not authenticated"
            }));
        }
    };
    
    let command_id = path.0.clone();
    
    // Use guild_id from query params, or default to first available guild
    let guild_id = match query.guild_id {
        Some(id) => id,
        None => {
            // Check if user has any guilds in claims
            if let Some(first_guild) = claims.user.guilds.first() {
                match first_guild.parse::<i64>() {
                    Ok(id) => id,
                    Err(_) => {
                        error!("Invalid guild ID format in user claims: {}", first_guild);
                        return HttpResponse::BadRequest().json(serde_json::json!({
                            "error": "Invalid guild ID format"
                        }));
                    }
                }
            } else {
                error!("No guild ID provided and user has no guilds");
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "No guild ID provided and user has no guilds"
                }));
            }
        }
    };
    
    info!("Retrieving details for command: {} in guild: {}", command_id, guild_id);
    
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
    // Get authenticated user from request extensions
    let extensions = req.extensions();
    let claims = match extensions.get::<Claims>() {
        Some(claims) => claims,
        None => {
            error!("No authentication claims found in request");
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Not authenticated"
            }));
        }
    };
    
    let command_id = path.0.clone();
    
    // Use guild_id from query params, or check available guilds from claims
    let guild_id = match query.guild_id {
        Some(id) => id,
        None => {
            // Check if user has any guilds in claims
            if let Some(first_guild) = claims.user.guilds.first() {
                match first_guild.parse::<i64>() {
                    Ok(id) => id,
                    Err(_) => {
                        error!("Invalid guild ID format in user claims: {}", first_guild);
                        return HttpResponse::BadRequest().json(serde_json::json!({
                            "error": "Invalid guild ID format"
                        }));
                    }
                }
            } else {
                error!("No guild ID provided and user has no guilds");
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "No guild ID provided and user has no guilds"
                }));
            }
        }
    };
    
    info!("Retrieving settings for command: {} in guild: {}", command_id, guild_id);
    
    // Verify user has access to this guild
    if !claims.user.guilds.iter().any(|g| g.parse::<i64>().map_or(false, |id| id == guild_id)) {
        error!("User does not have access to guild {}", guild_id);
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "You don't have access to this guild"
        }));
    }
    
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
    // Get authenticated user from request extensions
    let extensions = req.extensions();
    let claims = match extensions.get::<Claims>() {
        Some(claims) => claims,
        None => {
            error!("No authentication claims found in request");
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Not authenticated"
            }));
        }
    };
    
    let command_id = path.0.clone();
    
    // Use guild_id from query params or default to first available guild
    let guild_id = match query.guild_id {
        Some(id) => id,
        None => {
            // Check if user has any guilds in claims
            if let Some(first_guild) = claims.user.guilds.first() {
                match first_guild.parse::<i64>() {
                    Ok(id) => id,
                    Err(_) => {
                        error!("Invalid guild ID format in user claims: {}", first_guild);
                        return HttpResponse::BadRequest().json(serde_json::json!({
                            "error": "Invalid guild ID format"
                        }));
                    }
                }
            } else {
                error!("No guild ID provided and user has no guilds");
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "No guild ID provided and user has no guilds"
                }));
            }
        }
    };
    
    info!("Updating settings for command: {} in guild: {}", command_id, guild_id);
    
    // Verify user has access to this guild
    if !claims.user.guilds.iter().any(|g| g.parse::<i64>().map_or(false, |id| id == guild_id)) {
        error!("User does not have access to guild {}", guild_id);
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "You don't have access to this guild"
        }));
    }
    
    // Create services
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
    
    // Create command service with Discord integration
    let mut command_service = CommandService::new(state.database().clone());
    
    // If we have a bot token and application ID, add Discord service for command sync
    // If we have a bot token and application ID, add Discord service for command sync
    if let (Some(token), Some(app_id)) = (
        Some(state.config().bot.bot_token.clone()), 
        state.config().bot.application_id.clone()
    ) {
        let discord_service = crate::web::services::discord::DiscordService::new_bot(
            token, 
            Some(app_id)
        );
        command_service = command_service.with_discord_service(discord_service);
        debug!("Discord API service configured for command sync");
    } else {
        warn!("Bot token or application ID missing, command sync with Discord will be disabled");
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
    // Get authenticated user from request extensions
    let extensions = req.extensions();
    let claims = match extensions.get::<Claims>() {
        Some(claims) => claims,
        None => {
            error!("No authentication claims found in request");
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Not authenticated"
            }));
        }
    };
    
    let guild_id = path.into_inner();
    
    info!("Retrieving all commands for guild: {}", guild_id);
    
    // Verify user has access to this guild
    if !claims.user.guilds.iter().any(|g| g.parse::<i64>().map_or(false, |id| id == guild_id)) {
        error!("User does not have access to guild {}", guild_id);
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "You don't have access to this guild"
        }));
    }
    
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
    
    // Get all command settings for this guild with error handling for missing table
    match command_service.get_all_command_settings(guild_id).await {
        Ok(settings) => {
            HttpResponse::Ok().json(settings)
        },
        Err(e) => {
            let error_string = e.to_string();
            
            // Special handling for the case where the table doesn't exist yet
            if error_string.contains("relation \"commands\" does not exist") {
                error!("Database error: Table commands does not exist yet");
                
                // Return empty array since commands aren't defined yet
                HttpResponse::Ok().json(serde_json::json!([]))
            } else {
                error!("Error getting all command settings: {}", e);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to retrieve command settings: {}", e)
                }))
            }
        }
    }
}

/// Query parameters for guild requests
#[derive(serde::Deserialize)]
struct GuildQuery {
    guild_id: Option<i64>,
}
