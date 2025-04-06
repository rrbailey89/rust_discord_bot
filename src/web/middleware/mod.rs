// src/web/middleware/mod.rs
//! Middleware for web server

pub mod rate_limit;
pub mod logging;
pub mod auth;

pub use logging::RequestLogger;
pub use auth::JwtAuth;
