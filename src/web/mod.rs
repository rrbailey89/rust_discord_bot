// src/web/mod.rs
//! Web server module for Discord bot management

mod server;
mod state;
mod middleware;
mod routes;
mod handlers;
pub mod models; // Make models public
pub mod services;

pub use server::start_server;

/// Initialize the web server module.
/// This function is called at application startup.
pub async fn init() {
    tracing::info!("Initializing web server module");
}
