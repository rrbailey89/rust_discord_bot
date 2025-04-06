// src/web/routes/config.rs
//! Configuration-related API endpoints

use actix_web::{web, HttpResponse, Responder};
use serde::Serialize;
use std::env;
use tracing::error;

// Response structure for the client ID
#[derive(Serialize)]
struct ClientIdResponse {
    #[serde(rename = "clientId")]
    client_id: String,
}

/// Handler to get the Discord Client ID
async fn get_client_id() -> impl Responder {
    match env::var("DISCORD_CLIENT_ID") {
        Ok(client_id) => {
            if client_id.is_empty() {
                error!("DISCORD_CLIENT_ID environment variable is set but empty.");
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Server configuration error: Client ID is empty."
                }))
            } else {
                HttpResponse::Ok().json(ClientIdResponse { client_id })
            }
        }
        Err(_) => {
            error!("DISCORD_CLIENT_ID environment variable not found.");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Server configuration error: Client ID not configured."
            }))
        }
    }
}

/// Configure config routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/config")
            .route("/client-id", web::get().to(get_client_id))
    );
}
