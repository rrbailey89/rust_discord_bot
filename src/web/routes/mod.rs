// src/web/routes/mod.rs
//! API route definitions

mod auth;
mod guilds;
mod commands;
mod word_detection;
mod settings;
mod analytics;

use actix_web::web;
use actix_web::web::ServiceConfig;

/// Configure all API routes
pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/api")
            // Health check and version routes are configured directly in server.rs
            .configure(auth::configure)
            .configure(guilds::configure)
            .configure(commands::configure)
            .configure(word_detection::configure)
            .configure(settings::configure)
            .configure(analytics::configure)
    );
}
