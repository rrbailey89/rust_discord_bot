// src/web/routes/auth.rs
//! Authentication routes

use actix_web::{web, HttpResponse, Responder, HttpRequest, cookie::Cookie};
use serde::{Deserialize, Serialize};
use tracing::{error, info, debug};

use crate::web::state::WebAppState;
use crate::web::middleware::auth::Claims;
use crate::web::models::auth::AuthResponse;
use crate::web::services::auth::AuthService;
use crate::error::Error;

/// OAuth2 callback parameters
#[derive(Deserialize)]
pub struct OAuthCallback {
    code: String,
    state: Option<String>,
}

/// Token refresh request
#[derive(Deserialize)]
pub struct RefreshTokenRequest {
    refresh_token: String,
}

/// Configure authentication routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .route("/login", web::get().to(login))
            .route("/callback", web::get().to(oauth_callback))
            .route("/discord/callback", web::get().to(oauth_callback))  // Add compatibility with Discord's expected callback
            .route("/refresh", web::post().to(refresh_token))
    );
}

/// Redirect to Discord OAuth login
async fn login(state: web::Data<WebAppState>) -> impl Responder {
    // Create auth service
    let auth_service = AuthService::new(
        state.database().clone(),
        state.config().web.clone().into(),
    );
    
    // Get authorization URL
    let (auth_url, csrf_state) = auth_service.get_authorization_url();
    
    debug!("Generated Discord OAuth URL: {}", auth_url);
    
    // Redirect to Discord's OAuth page
    HttpResponse::Found()
        .append_header(("Location", auth_url))
        .finish()
}

/// Handle OAuth callback
async fn oauth_callback(
    req: HttpRequest,
    query: web::Query<OAuthCallback>,
    state: web::Data<WebAppState>,
) -> impl Responder {
    let code = query.code.clone();
    
    info!("Received OAuth callback with code: {}", code);
    
    // Create auth service
    let auth_service = AuthService::new(
        state.database().clone(),
        state.config().web.clone().into(),
    );
    
    // Exchange the code for tokens and create authentication response
    match auth_service.create_auth_response(code).await {
        Ok(auth_response) => {
            debug!("Successfully authenticated user: {}", auth_response.user.username);
            
            // Set JWT token as a cookie
            let mut cookie = Cookie::new("token", auth_response.token.clone());
            cookie.set_path("/");
            cookie.set_http_only(true);
            
            // Encode token for URL parameter
            let token_param = auth_response.token.clone();
            
            // Redirect to frontend callback handler with token
            HttpResponse::Found()
                .cookie(cookie)
                .append_header(("Location", format!("/auth-callback?token={}", token_param)))
                .finish()
        }
        Err(e) => {
            error!("Authentication error: {}", e);
            // Redirect to error page with error message
            let error_msg = format!("Authentication failed: {}", e);
            HttpResponse::Found()
                .append_header(("Location", format!("/auth-callback?error={}", 
                    urlencoding::encode(&error_msg))))
                .finish()
        }
    }
}

/// Refresh an existing token
async fn refresh_token(
    req: HttpRequest,
    refresh_req: web::Json<RefreshTokenRequest>,
    state: web::Data<WebAppState>,
) -> impl Responder {
    let refresh_token = refresh_req.refresh_token.clone();
    
    // Create auth service
    let auth_service = AuthService::new(
        state.database().clone(),
        state.config().web.clone().into(),
    );
    
    // Extract user ID from JWT if available
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                if let Ok(claims) = auth_service.verify_jwt(token) {
                    // User is authenticated, attempt to refresh token
                    match auth_service.refresh_discord_token(&refresh_token).await {
                        Ok(token) => {
                            // Store the refreshed token
                            let user_id = claims.sub.parse::<i64>().unwrap_or_default();
                            if let Err(e) = auth_service.store_user_session(user_id, &token).await {
                                error!("Failed to store refreshed token: {}", e);
                            }
                            
                            // Clone user info for generating JWT
                            let id = claims.user.id.clone();
                            let username = claims.user.username.clone();
                            let avatar_url = claims.user.avatar_url.clone();
                            let guilds = claims.user.guilds.clone();
                            
                            // Generate new JWT
                            match auth_service.generate_jwt_from_info(&id, &username, avatar_url.clone(), guilds.clone()) {
                                Ok((jwt, expires_in)) => {
                                    let auth_response = AuthResponse {
                                        token: jwt,
                                        expires_in,
                                        user: crate::web::models::auth::UserInfo {
                                            id,
                                            username,
                                            avatar_url,
                                            guilds,
                                        },
                                    };
                                    
                                    return HttpResponse::Ok().json(auth_response);
                                }
                                Err(e) => {
                                    error!("Failed to generate JWT: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to refresh Discord token: {}", e);
                        }
                    }
                }
            }
        }
    }
    
    // If we got here, something went wrong
    HttpResponse::Unauthorized().json(serde_json::json!({
        "error": "Invalid or expired authentication"
    }))
}
