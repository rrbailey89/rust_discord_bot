// src/web/routes/analytics.rs
//! Analytics management routes

use actix_web::{web, HttpResponse, Responder, HttpRequest};
use tracing::{error, info};

use crate::web::state::WebAppState;
use crate::web::models::analytics::{
    LogEventRequest, AnalyticsQueryParams, AnalyticsResponse
};
use crate::web::services::AnalyticsService;
use crate::web::services::GuildService;

/// Configure analytics routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/analytics")
            .route("/events", web::post().to(log_event))
            .route("/events", web::get().to(get_events))
            .route("/guild/{guild_id}", web::get().to(get_guild_summary))
            .route("/user/{user_id}", web::get().to(get_user_summary))
    );
}

/// Log a new analytics event
async fn log_event(
    req: HttpRequest,
    event_request: web::Json<LogEventRequest>,
    state: web::Data<WebAppState>,
) -> impl Responder {
    info!("Logging analytics event: {}", event_request.event_type);
    
    // Create analytics service
    let analytics_service = AnalyticsService::new(state.database().clone());
    
    // Log the event
    match analytics_service.log_event(&event_request).await {
        Ok(event_id) => {
            let response = AnalyticsResponse {
                success: true,
                message: "Analytics event logged successfully".to_string(),
                event_id: Some(event_id),
            };
            
            HttpResponse::Created().json(response)
        },
        Err(e) => {
            error!("Error logging analytics event: {}", e);
            
            let response = AnalyticsResponse {
                success: false,
                message: format!("Failed to log analytics event: {}", e),
                event_id: None,
            };
            
            HttpResponse::InternalServerError().json(response)
        }
    }
}

/// Get analytics events matching query parameters
async fn get_events(
    req: HttpRequest,
    query: web::Query<AnalyticsQueryParams>,
    state: web::Data<WebAppState>,
) -> impl Responder {
    info!("Retrieving analytics events");
    
    // Only admins should be able to query analytics events
    // For now, we'll skip admin check during development
    
    // Create analytics service
    let analytics_service = AnalyticsService::new(state.database().clone());
    
    // Get events
    match analytics_service.get_events(&query).await {
        Ok(events) => {
            HttpResponse::Ok().json(events)
        },
        Err(e) => {
            error!("Error getting analytics events: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to retrieve analytics events: {}", e)
            }))
        }
    }
}

/// Get guild analytics summary (handles "all" for all guilds)
async fn get_guild_summary(
    req: HttpRequest,
    path: web::Path<String>, // Changed to String to handle "all"
    query: web::Query<PeriodQuery>,
    state: web::Data<WebAppState>,
) -> impl Responder {
    let guild_id_str = path.into_inner();
    let period = query.get_period(); // Use helper method

    // Determine if fetching for a specific guild or all guilds
    let guild_id_opt: Option<i64> = if guild_id_str.eq_ignore_ascii_case("all") {
        info!("Retrieving analytics summary for ALL guilds (period: {})", period);
        None
    } else {
        match guild_id_str.parse::<i64>() {
            Ok(id) => {
                info!("Retrieving analytics summary for guild: {} (period: {})", id, period);
                Some(id)
            },
            Err(_) => {
                error!("Invalid guild ID format in path: {}", guild_id_str);
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid guild ID format. Use numeric ID or 'all'."
                }));
            }
        }
    };

    // Create services
    let analytics_service = AnalyticsService::new(state.database().clone());
    let guild_service = GuildService::new(state.database().clone()); // Keep for potential future checks

    // Optional: Check if bot is in the specific guild if one is provided
    if let Some(guild_id) = guild_id_opt {
        match guild_service.is_bot_in_guild(guild_id).await {
            Ok(true) => { /* Bot is in guild, proceed */ },
            Ok(false) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Bot is not in this guild"
                }));
            },
            Err(e) => {
                error!("Error checking if bot is in guild {}: {}", guild_id, e);
                return HttpResponse::InternalServerError().finish();
            }
        }
    }
    // No check needed if guild_id_opt is None (fetching for "all")

    // Get guild summary (passing Option<i64>)
    match analytics_service.get_guild_summary(guild_id_opt, &period).await {
        Ok(summary) => {
            // TODO: Consider if the summary structure needs adjustment when guild_id is 0 (for "all")
            HttpResponse::Ok().json(summary)
        },
        Err(e) => {
            error!("Error getting guild analytics summary: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to retrieve guild analytics summary: {}", e)
            }))
        }
    }
}

/// Get user activity summary
async fn get_user_summary(
    req: HttpRequest,
    path: web::Path<i64>,
    query: web::Query<PeriodQuery>,
    state: web::Data<WebAppState>,
) -> impl Responder {
    let user_id = path.into_inner();
    let period = query.period.as_deref().unwrap_or("week");
    
    info!("Retrieving activity summary for user: {} (period: {})", user_id, period);
    
    // Create analytics service
    let analytics_service = AnalyticsService::new(state.database().clone());
    
    // Get user summary
    match analytics_service.get_user_summary(user_id, period).await {
        Ok(summary) => {
            HttpResponse::Ok().json(summary)
        },
        Err(e) => {
            error!("Error getting user activity summary: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to retrieve user activity summary: {}", e)
            }))
        }
    }
}

/// Query parameters for period
#[derive(serde::Deserialize, Debug)] // Added Debug
struct PeriodQuery {
    period: Option<String>,
}

impl PeriodQuery {
    // Helper to get period string with default
    fn get_period(&self) -> String {
        self.period.as_deref().unwrap_or("week").to_lowercase()
    }
}
