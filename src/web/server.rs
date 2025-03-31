// src/web/server.rs
//! Actix Web server implementation

use crate::Data;
use crate::web::state::WebAppState;
use crate::web::routes;
use crate::web::middleware::{JwtAuth, RequestLogger};
use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_web::{
    web::{self, Data as WebData},
    App, HttpResponse, HttpServer, Responder, middleware::Logger,
};
use std::net::TcpListener;
use std::sync::Arc;
use tracing::{error, info};

/// Handle SPA routes by serving index.html for non-API routes
async fn spa_index() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("./static/index.html")?)
}

/// Start the web server
/// 
/// # Arguments
/// 
/// * `bot_data` - Shared bot data
/// * `port` - Port to listen on
/// 
/// # Returns
/// 
/// * `Result<(), crate::error::Error>` - Result
pub async fn start_server(bot_data: Arc<Data>, port: u16) -> Result<(), crate::error::Error> {
    info!("Starting web server on port {}", port);
    
    // Create the application state
    let app_state = WebAppState::new(bot_data);
    
    // Try to bind to the port
    let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)) {
        Ok(listener) => {
            info!("Successfully bound to port {}", port);
            listener
        }
        Err(e) => {
            error!("Failed to bind to port {}: {}", port, e);
            return Err(crate::error::Error::Unknown(format!(
                "Failed to bind to port {}: {}", 
                port, 
                e
            )));
        }
    };
    
    // Get JWT secret from configuration
    let jwt_secret = app_state.config().web.jwt_secret.clone();
    
    // Start HTTP server
    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        
        App::new()
            // Global middleware
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(RequestLogger)
            
            // Application state
            .app_data(WebData::new(app_state.clone()))
            
            // API routes - health and version endpoints
            .service(
                web::scope("/api")
                    .route("/health", web::get().to(health_check))
                    .route("/version", web::get().to(version))
                    
                    // Authentication routes (no JWT auth required)
                    .service(
                        web::scope("/auth")
                            .configure(routes::auth::configure)
                    )
                    
            // Protected routes (require JWT auth)
            .service(
                web::scope("")
                    .wrap(JwtAuth::new(jwt_secret.clone()))
                    .configure(routes::guilds::configure)
                    .configure(routes::commands::configure)
                    .configure(routes::word_detection::configure)
                    .configure(routes::settings::configure)
                    .configure(routes::analytics::configure)
                    .configure(routes::users::configure)
            )
            )
            
            // Client-side routing support - explicitly define SPA routes
            .route("/login", web::get().to(spa_index))
            .route("/auth-callback", web::get().to(spa_index))
            .route("/api/auth/callback", web::get().to(spa_index))
            .route("/api/auth/discord/callback", web::get().to(spa_index))
            
            // Static files (must come after explicit routes)
            .service(Files::new("/", "./static").index_file("index.html"))
            
            // Fallback route for SPA
            .default_service(web::get().to(spa_index))
    })
    .listen(listener)?
    .run()
    .await
    .map_err(|e| crate::error::Error::Unknown(format!("Web server error: {}", e)))
}

/// Health check endpoint
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Version endpoint
async fn version() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "name": env!("CARGO_PKG_NAME")
    }))
}
