// src/web/handlers/guilds.rs
//! Handlers for guild-related API endpoints

use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use tracing::{error, info, debug};

use crate::web::state::WebAppState;
use crate::web::middleware::auth::Claims;
use crate::web::services::discord::DiscordService;
use crate::error::Error;
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
    
    // Get guild details
    match guild_service.get_guild(&guild_id).await {
        Ok(guild) => success(guild),
        Err(e) => {
            error!("Error fetching guild details: {}", e);
            error_response(
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Error fetching guild details: {}", e),
            )
        }
    }
}
