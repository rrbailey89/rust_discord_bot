// src/web/routes/mod.rs
//! API route definitions

pub mod auth;
pub mod guilds;
pub mod commands;
pub mod word_detection;
pub mod settings;
pub mod analytics;
pub mod users;
pub mod config; // Add config module

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
            .configure(users::configure)
    );
}
