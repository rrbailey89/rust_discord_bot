// src/web/handlers/mod.rs
//! Request handlers for the web server

use actix_web::{web, HttpResponse, Responder, HttpRequest, http::StatusCode, HttpMessage};
use serde::{Deserialize, Serialize};
use tracing::{error, info, debug};

use crate::error::Error;
use crate::web::state::WebAppState;
use crate::web::middleware::auth::Claims;
use crate::web::services::discord::DiscordService;

// Modules for different handler categories
pub mod users;
pub mod guilds;
pub mod members;

// Re-exports for easier imports
pub use users::*;
pub use guilds::*;
pub use members::*;

/// Common response wrapper for API endpoints
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    /// Whether the request was successful
    pub success: bool,
    /// Optional message
    pub message: Option<String>,
    /// Optional data payload
    pub data: Option<T>,
}

/// Create a success response with data
pub fn success<T>(data: T) -> HttpResponse 
where 
    T: Serialize
{
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: None,
        data: Some(data),
    })
}

/// Create a success response with a message
pub fn success_message(message: &str) -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse::<()> {
        success: true,
        message: Some(message.to_string()),
        data: None,
    })
}

/// Create an error response
pub fn error_response(status: StatusCode, message: &str) -> HttpResponse {
    HttpResponse::build(status).json(ApiResponse::<()> {
        success: false,
        message: Some(message.to_string()),
        data: None,
    })
}

/// Extract claims from request
pub fn get_claims_from_request(req: &HttpRequest) -> Result<Claims, HttpResponse> {
    let claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => {
            error!("No authentication claims found in request");
            return Err(error_response(
                StatusCode::UNAUTHORIZED,
                "Not authenticated",
            ));
        }
    };
    
    Ok(claims)
}

/// Initialize handler module
pub fn init() {
    // We'll use this later if we need to do any one-time initialization
    info!("Initializing API handlers");
}
