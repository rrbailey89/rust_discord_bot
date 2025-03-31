// src/web/handlers/members.rs
//! Handlers for guild member-related API endpoints

use actix_web::{web, Responder, HttpRequest};
use serde::Deserialize;
use tracing::{error, info};

use crate::web::state::WebAppState;
use crate::web::services::discord::DiscordService;
use super::{success, error_response, get_claims_from_request};

/// Request parameters for fetching guild members
#[derive(Debug, Deserialize)]
pub struct GetMembersQuery {
    /// Maximum number of members to return (default: 100)
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    100
}

/// Get guild members handler
pub async fn get_guild_members(
    req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<GetMembersQuery>,
    state: web::Data<WebAppState>,
) -> impl Responder {
    // Get authenticated user from request
    let claims = match get_claims_from_request(&req) {
        Ok(claims) => claims,
        Err(response) => return response,
    };
    
    let guild_id = path.into_inner();
    let limit = query.limit;
    
    // Create Discord service with user's token
    // This will need to be fetched from the user session
    let discord_service = match state.database().get_client().await {
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
    
    // Get guild members
    match discord_service.get_guild_members(&guild_id, limit).await {
        Ok(members) => {
            // Store fetched members in the database
            let guild_id_i64 = match guild_id.parse::<i64>() {
                Ok(id) => id,
                Err(e) => {
                    error!("Failed to parse guild ID: {}", e);
                    return error_response(
                        actix_web::http::StatusCode::BAD_REQUEST,
                        "Invalid guild ID format",
                    );
                }
            };
            
            // Convert the members to JSON for storage
            let members_json = match serde_json::to_value(&members) {
                Ok(json) => {
                    if let Some(members_array) = json.as_array() {
                        // Extract just the array part for storage
                        serde_json::to_value(members_array).unwrap_or(serde_json::Value::Array(vec![]))
                    } else {
                        serde_json::Value::Array(vec![])
                    }
                },
                Err(e) => {
                    error!("Failed to serialize members to JSON: {}", e);
                    serde_json::Value::Array(vec![])
                }
            };
            
            // Store in database if we have valid JSON
            if let Some(members_array) = members_json.as_array() {
                match state.database().store_guild_members(guild_id_i64, members_array).await {
                    Ok(count) => {
                        info!("Stored {} members for guild {}", count, guild_id);
                    },
                    Err(e) => {
                        error!("Error storing guild members: {}", e);
                    }
                }
            }
            
            // Return the members to the caller regardless of storage success
            success(members)
        },
        Err(e) => {
            error!("Error fetching guild members: {}", e);
            error_response(
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Error fetching guild members: {}", e),
            )
        }
    }
}
