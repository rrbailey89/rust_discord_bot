// src/web/handlers/users.rs
//! Handlers for user-related API endpoints

use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use tracing::{error, info, debug};

use crate::web::state::WebAppState;
use crate::web::middleware::auth::Claims;
use crate::web::services::discord::DiscordService;
use crate::error::Error;
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
    
    // Return user info from claims
    let profile = UserProfile {
        id: claims.user.id.clone(),
        username: claims.user.username.clone(),
        avatar_url: claims.user.avatar_url.clone(),
        email: claims.user.email.clone(),
        verified: claims.user.verified,
        locale: claims.user.locale.clone(),
    };
    
    success(profile)
}
