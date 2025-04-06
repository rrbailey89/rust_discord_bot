// src/web/handlers/users.rs
//! Handlers for user-related API endpoints

use actix_web::{web, Responder, HttpRequest};
use serde::Serialize;
use tracing::{error, debug};

use crate::web::state::WebAppState;
use super::{success, error_response, get_claims_from_request};

#[derive(Debug, Serialize)]
pub struct UserProfile {
    /// User ID
    pub id: String,
    /// Username
    pub username: String,
    /// User's avatar URL
    pub avatar_url: Option<String>,
    /// User's email
    pub email: Option<String>,
    /// Whether the user's email is verified
    pub verified: Option<bool>,
    /// User's locale
    pub locale: Option<String>,
    /// User's Discord guilds (servers)
    pub guilds: Vec<crate::web::models::auth::Guild>,
}

/// Get current user profile information
pub async fn get_current_user(
    req: HttpRequest,
    state: web::Data<WebAppState>,
) -> impl Responder {
    // Get authenticated user from request
    let claims = match get_claims_from_request(&req) {
        Ok(claims) => claims,
        Err(response) => return response,
    };
    
    // Get user session to get their Discord token
    let user_id = match claims.user.id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            error!("Failed to parse user ID to i64: {}", claims.user.id);
            return error_response(
                actix_web::http::StatusCode::BAD_REQUEST,
                "Invalid user ID format"
            );
        }
    };
    
    // Get user's full guild information from Discord API
    let mut guilds = Vec::new();
    
    let auth_service = state.auth_service();
    
    if let Ok(Some(session)) = auth_service.get_user_session(user_id).await {
        if let Some(discord_token) = &session.discord_token {
            // Create Discord service with caching
            let discord_service = state.discord_service(discord_token);
            
            // Fetch guilds from Discord API (with caching)
            match discord_service.get_current_user_guilds(false).await {
                Ok(discord_guilds) => {
                    // Convert to our Guild model
                    guilds = discord_guilds.data
                        .into_iter()
                        .map(|g| {
                            // Convert permissions from string to u64 (accounting for Option)
                            let permissions = g.permissions
                                .as_ref()
                                .and_then(|p| u64::from_str_radix(p, 10).ok())
                                .unwrap_or(0);
                            
                            // Create Guild object with full details
                            crate::web::models::auth::Guild {
                                id: g.id,
                                name: g.name,
                                icon: g.icon,
                                owner: g.owner.unwrap_or(false),
                                permissions,
                                bot_joined: false, // Will be set later if needed
                            }
                        })
                        .collect();
                        
                    debug!("Fetched {} guilds for user {}", guilds.len(), claims.user.id);
                }
                Err(e) => {
                    error!("Failed to fetch Discord guilds: {}", e);
                }
            }
        }
    }
    
    // Return user info from claims with the fetched guilds
    let profile = UserProfile {
        id: claims.user.id.clone(),
        username: claims.user.username.clone(),
        avatar_url: claims.user.avatar_url.clone(),
        email: claims.user.email.clone(),
        verified: claims.user.verified,
        locale: claims.user.locale.clone(),
        guilds,
    };
    
    success(profile)
}
