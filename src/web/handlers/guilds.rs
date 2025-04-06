// src/web/handlers/guilds.rs
//! Handlers for guild-related API endpoints

use actix_web::{web, Responder, HttpRequest};
use tracing::error;

use crate::web::state::WebAppState;
use crate::web::services::discord::DiscordService;
use super::{success, error_response, get_claims_from_request};

/// Get guild details handler
pub async fn get_guild_details(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<WebAppState>,
) -> impl Responder {
    // Get authenticated user from request
    let claims = match get_claims_from_request(&req) {
        Ok(claims) => claims,
        Err(response) => return response,
    };
    
    let guild_id = path.into_inner();
    
    // Create Discord service with user's token
    // This will need to be fetched from the user session
use serde_json::json;

let guild_service = match state.database().get_client().await {
        Ok(client) => {
            let row = match client.query_opt(
                "SELECT discord_token FROM user_sessions WHERE user_id = $1",
                &[&claims.sub.parse::<i64>().unwrap_or_default()],
            ).await {
                Ok(Some(row)) => row,
                Ok(None) => {
                    return error_response(
                        actix_web::http::StatusCode::UNAUTHORIZED,
                        "No active session found",
                    );
                },
                Err(e) => {
                    error!("Database error: {}", e);
                    return error_response(
                        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "Database error",
                    );
                }
            };
            
            let token: Option<String> = row.get(0);
            if let Some(token) = token {
                DiscordService::new(token)
            } else {
                return error_response(
                    actix_web::http::StatusCode::UNAUTHORIZED,
                    "No Discord token found",
                );
            }
        },
        Err(e) => {
            error!("Database error: {}", e);
            return error_response(
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Database error",
            );
        }
    };

    // Check if the bot is in this guild (without returning 404 if it's not)
    let guild_id_i64 = match guild_id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            error!("Invalid guild ID format: {}", guild_id);
            return error_response(
                actix_web::http::StatusCode::BAD_REQUEST,
                "Invalid guild ID format",
            );
        }
    };
    
    let bot_joined = match state.database().get_client().await {
        Ok(client) => {
            match client.query_opt(
                "SELECT 1 FROM guild_info WHERE guild_id = $1 LIMIT 1",
                &[&guild_id_i64],
            ).await {
                Ok(result) => result.is_some(),
                Err(e) => {
                    error!("Error checking if bot is in guild {}: {}", guild_id, e);
                    false // Default to false on error
                }
            }
        },
        Err(e) => {
            error!("Database error: {}", e);
            false // Default to false on error
        }
    };
    
    // Get guild details from Discord API
    match guild_service.get_guild(&guild_id, false).await {
        Ok(guild) => {
            // Attempt to fetch member list to get count
            let member_count = if let Ok(members) = guild_service.get_guild_members(&guild_id, 1000).await {
                members.len() as i64
            } else {
                // If an error occurs, default to 0 or handle differently as needed
                0
            };

            // Build response object with correct membership status
            let result = json!({
                "id": guild.data.id,
                "name": guild.data.name,
                "icon": guild.data.icon,
                "features": guild.data.features,
                "member_count": member_count,
                "botJoined": bot_joined
            });

            success(result)
        },
        Err(e) => {
            error!("Error fetching guild details: {}", e);
            error_response(
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Error fetching guild details: {}", e),
            )
        }
    }
}
