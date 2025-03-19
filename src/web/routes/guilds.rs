// src/web/routes/guilds.rs
//! Guild management routes

use actix_web::{web, HttpResponse, Responder, HttpRequest, http::StatusCode, HttpMessage};
use serde::Serialize;
use tracing::{error, info, debug};

use crate::error::Error;
use crate::web::state::WebAppState;
use crate::web::middleware::auth::Claims;
use crate::web::models::guild::{
    GuildInfo, GuildDetails, ChannelInfo, GuildSettings, 
    UpdateGuildSettingsRequest, GuildResponse
};
use crate::web::services::AuthService;
use crate::web::services::DiscordService;
use crate::web::services::GuildService;

/// Configure guild routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/guilds")
            .route("", web::get().to(list_guilds))
            .route("/{guild_id}", web::get().to(get_guild))
            .route("/{guild_id}/settings", web::get().to(get_guild_settings))
            .route("/{guild_id}/settings", web::put().to(update_guild_settings))
    );
}

/// List guilds the user has access to
async fn list_guilds(
    req: HttpRequest,
    state: web::Data<WebAppState>
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
    
    // Log claims for debugging
    debug!("Processing request with claims: user_id={}, username={}", 
           claims.sub, claims.user.username);
    
    // Parse user ID from JWT subject
    let user_id = match claims.sub.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid user ID format"
        })),
    };
    
    // Get user session with Discord token
    let auth_service = AuthService::new(
        state.database().clone(),
        state.config().web.clone().into(),
    );
    
    let session = match auth_service.get_user_session(user_id).await {
        Ok(Some(session)) => session,
        Ok(None) => return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "No active session found"
        })),
        Err(e) => {
            error!("Database error: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    
    // Call Discord API to get guilds (with caching)
    let discord_service = state.discord_service(session.discord_token.unwrap_or_default());
    let guild_service = GuildService::new(state.database().clone());
    
    match discord_service.get_current_user_guilds().await {
        Ok(discord_guilds) => {
            // Convert Discord guilds to our API format
            let mut guild_infos = Vec::new();
            
            for discord_guild in discord_guilds {
                // Parse the guild ID to check if the bot is in this guild
                let guild_id = match discord_guild.id.parse::<i64>() {
                    Ok(id) => id,
                    Err(_) => {
                        error!("Invalid guild ID format: {}", discord_guild.id);
                        continue;
                    }
                };
                
                // Simple check if the bot is in this guild
                let bot_joined = guild_service.is_bot_in_guild(guild_id).await.unwrap_or_else(|e| {
                    error!("Error checking if bot is in guild {}: {}", guild_id, e);
                    false
                });
                
                // Parse permissions to u64
                let permissions = u64::from_str_radix(&discord_guild.permissions, 10)
                    .unwrap_or_default();
                
                // Build icon URL if available
                let icon_url = discord_guild.icon.as_ref().map(|icon| {
                    format!(
                        "https://cdn.discordapp.com/icons/{}/{}.png",
                        discord_guild.id, icon
                    )
                });
                
                // Get member count if the bot is in this guild
                let member_count = if bot_joined {
                    match guild_service.get_guild_member_count(guild_id).await {
                        Ok(count) => Some(count),
                        Err(e) => {
                            error!("Error fetching member count for guild {}: {}", guild_id, e);
                            None
                        }
                    }
                } else {
                    None
                };
                
                guild_infos.push(GuildInfo {
                    id: discord_guild.id.clone(),
                    name: discord_guild.name.clone(),
                    icon_url,
                    owner: discord_guild.owner.unwrap_or(false),
                    permissions,
                    bot_joined,
                    member_count,
                });
            }
            
            HttpResponse::Ok().json(guild_infos)
        },
        Err(e) => {
            error!("Discord API error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to retrieve guilds"
            }))
        }
    }
}

/// Get details for a specific guild
async fn get_guild(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<WebAppState>
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
    
    // Parse user ID from JWT subject
    let user_id = match claims.sub.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid user ID format"
        })),
    };
    
    let guild_id = path.into_inner();
    info!("Retrieving details for guild: {}", guild_id);
    
    // Check if the guild ID is valid
    let guild_id_i64 = match guild_id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            error!("Invalid guild ID format: {}", guild_id);
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid guild ID format"
            }));
        }
    };
    
    // Get user session with Discord token
    let auth_service = AuthService::new(
        state.database().clone(),
        state.config().web.clone().into(),
    );
    
    let session = match auth_service.get_user_session(user_id).await {
        Ok(Some(session)) => session,
        Ok(None) => return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "No active session found"
        })),
        Err(e) => {
            error!("Database error: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    
    // Extract token and convert to string for multiple uses
    let discord_token = match &session.discord_token {
        Some(token) => token.clone(),
        None => String::new(),
    };
    
    // Call Discord API to get guild and channels with caching
    let discord_service = state.discord_service(discord_token.clone());
    let guild_service = GuildService::new(state.database().clone());
    
    // Check if the bot is in this guild
    let bot_joined = match guild_service.is_bot_in_guild(guild_id_i64).await {
        Ok(result) => result,
        Err(e) => {
            error!("Error checking if bot is in guild {}: {}", guild_id, e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    
    // First check if we have this guild in the user's guild list
    let guild_info = match discord_service.get_current_user_guilds().await {
        Ok(guilds) => {
            // Find this specific guild in the list
            let this_guild = guilds.iter().find(|g| g.id == guild_id);
            if let Some(guild) = this_guild {
                debug!("Found guild {} in user's cached guild list", guild_id);
                // Return this guild info from the list
                Some(guild.clone())
            } else {
                debug!("Guild {} not found in user's guild list", guild_id);
                None
            }
        },
        Err(e) => {
            error!("Error fetching guilds from Discord: {}", e);
            None
        }
    };
    
    // Get guild details from Discord API - but don't fail if this fails
    let guild_result = discord_service.get_guild(&guild_id).await;
    
    // Get channels only if the bot is in the guild
    let channels_result = if bot_joined {
        discord_service.get_guild_channels(&guild_id).await
    } else {
        // Return empty channels if bot is not in the guild
        Ok(Vec::new())
    };
    
    // Handle the response based on available data
    match (guild_result, channels_result) {
        // Case 1: We successfully got both guild details and channels
        (Ok(guild), Ok(channels)) => {
            // Parse permissions to u64
            let permissions = u64::from_str_radix(&guild.permissions, 10)
                .unwrap_or_default();
            
            // Build icon URL if available
            let icon_url = guild.icon.as_ref().map(|icon| {
                format!(
                    "https://cdn.discordapp.com/icons/{}/{}.png",
                    guild.id, icon
                )
            });
            
            // Convert channels to our API format
            let channel_infos: Vec<ChannelInfo> = channels.into_iter()
                .filter_map(|channel| {
                    // Only include channels that have a name
                    channel.name.as_ref().map(|name| {
                        ChannelInfo {
                            id: channel.id,
                            name: name.clone(),
                            channel_type: channel.channel_type,
                            position: channel.position.unwrap_or(0),
                            topic: channel.topic,
                        }
                    })
                })
                .collect();
            
            // Get member count from the database
            let member_count = match guild_service.get_guild_member_count(guild_id_i64).await {
                Ok(count) => Some(count),
                Err(e) => {
                    error!("Error fetching member count for guild {}: {}", guild_id, e);
                    None
                }
            };
            
            // Build the guild details response
            let guild_details = GuildDetails {
                id: guild.id,
                name: guild.name,
                icon_url,
                owner: guild.owner.unwrap_or(false),
                permissions,
                member_count,
                channels: channel_infos,
            };
            
            HttpResponse::Ok().json(guild_details)
        },
        // Case 2: We failed to get guild details but have a backup from guild list
        (Err(e), channels_result) if guild_info.is_some() => {
            error!("Failed to get detailed guild info, using fallback: {}", e);
            let guild = guild_info.unwrap();
            
            // Parse permissions to u64
            let permissions = u64::from_str_radix(&guild.permissions, 10)
                .unwrap_or_default();
            
            // Build icon URL if available
            let icon_url = guild.icon.as_ref().map(|icon| {
                format!(
                    "https://cdn.discordapp.com/icons/{}/{}.png",
                    guild.id, icon
                )
            });
            
            // Get channels from result or empty vec
            let channels = match channels_result {
                Ok(chans) => chans,
                Err(_) => Vec::new(),
            };
            
            // Convert channels to our API format
            let channel_infos: Vec<ChannelInfo> = channels.into_iter()
                .filter_map(|channel| {
                    // Only include channels that have a name
                    channel.name.as_ref().map(|name| {
                        ChannelInfo {
                            id: channel.id,
                            name: name.clone(),
                            channel_type: channel.channel_type,
                            position: channel.position.unwrap_or(0),
                            topic: channel.topic,
                        }
                    })
                })
                .collect();
            
            // Get member count from the database
            let member_count = match guild_service.get_guild_member_count(guild_id_i64).await {
                Ok(count) => Some(count),
                Err(e) => {
                    error!("Error fetching member count for guild {}: {}", guild_id, e);
                    None
                }
            };
            
            // Build the guild details response
            let guild_details = GuildDetails {
                id: guild.id,
                name: guild.name,
                icon_url,
                owner: guild.owner.unwrap_or(false),
                permissions,
                member_count,
                channels: channel_infos,
            };
            
            HttpResponse::Ok().json(guild_details)
        },
        // Case 3: Both approaches failed
        (Err(e), _) | (_, Err(e)) => {
            error!("Discord API error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to retrieve guild details"
            }))
        }
    }
}

/// Get settings for a specific guild
async fn get_guild_settings(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<WebAppState>
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
    info!("Retrieving settings for guild: {}", guild_id);
    
    // Check if the guild ID is valid
    let guild_id_i64 = match guild_id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            error!("Invalid guild ID format: {}", guild_id);
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid guild ID format"
            }));
        }
    };
    
    // Create guild service
    let guild_service = GuildService::new(state.database().clone());
    
    // Check if the bot is in this guild
    let bot_joined = match guild_service.is_bot_in_guild(guild_id_i64).await {
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
    
    // Get settings from database
    match guild_service.get_guild_settings(guild_id_i64).await {
        Ok(settings) => {
            HttpResponse::Ok().json(settings)
        },
        Err(e) => {
            error!("Database error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to retrieve guild settings"
            }))
        }
    }
}

/// Update settings for a specific guild
async fn update_guild_settings(
    req: HttpRequest,
    path: web::Path<String>,
    settings: web::Json<UpdateGuildSettingsRequest>,
    state: web::Data<WebAppState>
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
    info!("Updating settings for guild: {}", guild_id);
    
    // Check if the guild ID is valid
    let guild_id_i64 = match guild_id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            error!("Invalid guild ID format: {}", guild_id);
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid guild ID format"
            }));
        }
    };
    
    // Create guild service
    let guild_service = GuildService::new(state.database().clone());
    
    // Check if the bot is in this guild
    let bot_joined = match guild_service.is_bot_in_guild(guild_id_i64).await {
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
    
    // Update settings in database
    match guild_service.update_guild_settings(guild_id_i64, &settings).await {
        Ok(_) => {
            let response = GuildResponse {
                success: true,
                message: "Guild settings updated successfully".to_string(),
            };
            
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            error!("Database error: {}", e);
            
            let response = GuildResponse {
                success: false,
                message: format!("Failed to update guild settings: {}", e),
            };
            
            HttpResponse::InternalServerError().json(response)
        }
    }
}
