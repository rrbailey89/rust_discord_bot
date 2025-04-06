// config/web.rs
//! Web server configuration

use std::env;
use rand::Rng;
use crate::error::Error;

/// Web server configuration
#[derive(Clone, Debug)]
pub struct WebConfig {
    /// Web server port
    pub port: u16,
    /// Discord OAuth client ID
    pub discord_client_id: String,
    /// Discord OAuth client secret
    pub discord_client_secret: String,
    /// OAuth redirect URI
    pub oauth_redirect_uri: String,
    /// JWT secret key for token generation
    pub jwt_secret: String,
    /// JWT token expiration in seconds
    pub jwt_expiration: u64,
}

impl WebConfig {
    /// Load web server configuration from environment variables
    pub fn load() -> Result<Self, Error> {
        // Generate a random JWT secret if not provided
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
            // Generate a simple random string without using deprecated functions
            let mut rng = rand::thread_rng();
            let mut rand_string = String::with_capacity(32);
            for _ in 0..32 {
                // Use ASCII alphanumeric characters (33-126)
                let c = (rng.gen_range(0..26) + 97) as u8 as char; // lowercase a-z
                rand_string.push(c);
            }
            tracing::warn!("JWT_SECRET not set. Using a randomly generated secret for this session only.");
            rand_string
        });
        
        // Convert port string to u16
        let port = env::var("WEB_SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .unwrap_or(8080);
            
        // Get Discord OAuth configuration
        let discord_client_id = env::var("DISCORD_CLIENT_ID")
            .unwrap_or_else(|_| {
                tracing::warn!("DISCORD_CLIENT_ID not set. OAuth will not work correctly.");
                "".to_string()
            });
            
        let discord_client_secret = env::var("DISCORD_CLIENT_SECRET")
            .unwrap_or_else(|_| {
                tracing::warn!("DISCORD_CLIENT_SECRET not set. OAuth will not work correctly.");
                "".to_string()
            });
            
        // Default to a localhost redirect for development
        let oauth_redirect_uri = env::var("OAUTH_REDIRECT_URI")
            .unwrap_or_else(|_| {
                let default_uri = format!("http://localhost:{}/api/auth/callback", port);
                tracing::warn!("OAUTH_REDIRECT_URI not set. Using default: {}", default_uri);
                default_uri
            });
            
        // JWT token expiration in seconds (default to 24 hours)
        let jwt_expiration = env::var("JWT_EXPIRATION")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(86400);
            
        Ok(Self {
            port,
            discord_client_id,
            discord_client_secret,
            oauth_redirect_uri,
            jwt_secret,
            jwt_expiration,
        })
    }
}
