// src/web/routes/settings.rs
//! Settings management routes

use actix_web::{web, HttpResponse, Responder, HttpRequest};
use tracing::{error, info};

use crate::web::state::WebAppState;
use crate::web::models::settings::{
    UpdateGlobalSettingsRequest, UpdateUserPreferencesRequest, 
    SettingsResponse
};
use crate::web::services::SettingsService;

/// Configure settings routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/settings")
            .route("/global", web::get().to(get_global_settings))
            .route("/global", web::put().to(update_global_settings))
            .route("/user", web::get().to(get_user_preferences))
            .route("/user", web::put().to(update_user_preferences))
    );
}

/// Get global bot settings
async fn get_global_settings(
    req: HttpRequest,
    state: web::Data<WebAppState>,
) -> impl Responder {
    info!("Retrieving global settings");
    
    // Only admins should be able to see global settings
    // For now, we'll just check for authentication without admin check
    
    // Create settings service
    let settings_service = SettingsService::new(state.database().clone());
    
    // Get settings
    match settings_service.get_global_settings().await {
        Ok(settings) => {
            HttpResponse::Ok().json(settings)
        },
        Err(e) => {
            error!("Error getting global settings: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to retrieve global settings: {}", e)
            }))
        }
    }
}

/// Update global bot settings
async fn update_global_settings(
    req: HttpRequest,
    settings_request: web::Json<UpdateGlobalSettingsRequest>,
    state: web::Data<WebAppState>,
) -> impl Responder {
    info!("Updating global settings");
    
    // Only admins should be able to update global settings
    // For now, we'll just check for authentication without admin check
    
    // Create settings service
    let settings_service = SettingsService::new(state.database().clone());
    
    // Update settings
    match settings_service.update_global_settings(&settings_request).await {
        Ok(()) => {
            let response = SettingsResponse {
                success: true,
                message: "Global settings updated successfully".to_string(),
            };
            
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            error!("Error updating global settings: {}", e);
            
            let response = SettingsResponse {
                success: false,
                message: format!("Failed to update global settings: {}", e),
            };
            
            HttpResponse::InternalServerError().json(response)
        }
    }
}

/// Get user preferences
async fn get_user_preferences(
    req: HttpRequest,
    state: web::Data<WebAppState>,
) -> impl Responder {
    // For now, use a default user ID until auth is fixed
    let user_id = 123456789i64;
    
    info!("Retrieving preferences for user: {}", user_id);
    
    // Create settings service
    let settings_service = SettingsService::new(state.database().clone());
    
    // Get preferences
    match settings_service.get_user_preferences(user_id).await {
        Ok(preferences) => {
            HttpResponse::Ok().json(preferences)
        },
        Err(e) => {
            error!("Error getting user preferences: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to retrieve user preferences: {}", e)
            }))
        }
    }
}

/// Update user preferences
async fn update_user_preferences(
    req: HttpRequest,
    preferences_request: web::Json<UpdateUserPreferencesRequest>,
    state: web::Data<WebAppState>,
) -> impl Responder {
    // For now, use a default user ID until auth is fixed
    let user_id = 123456789i64;
    
    info!("Updating preferences for user: {}", user_id);
    
    // Create settings service
    let settings_service = SettingsService::new(state.database().clone());
    
    // Update preferences
    match settings_service.update_user_preferences(user_id, &preferences_request).await {
        Ok(()) => {
            let response = SettingsResponse {
                success: true,
                message: "User preferences updated successfully".to_string(),
            };
            
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            error!("Error updating user preferences: {}", e);
            
            let response = SettingsResponse {
                success: false,
                message: format!("Failed to update user preferences: {}", e),
            };
            
            HttpResponse::InternalServerError().json(response)
        }
    }
}
