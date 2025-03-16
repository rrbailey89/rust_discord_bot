// src/web/middleware/mod.rs
//! Middleware for web server

mod rate_limit;
mod logging;
pub mod auth;

pub use rate_limit::RateLimiter;
pub use logging::RequestLogger;
pub use auth::JwtAuth;
