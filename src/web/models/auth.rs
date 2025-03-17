// src/web/models/auth.rs
//! Authentication models for web API

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// User session data stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    /// Discord user ID
    pub user_id: i64,
    /// Discord access token
    pub discord_token: Option<String>,
    /// When the Discord token expires
    pub token_expires_at: Option<DateTime<Utc>>,
    /// Discord refresh token
    pub refresh_token: Option<String>,
    /// Last login timestamp
    pub last_login: DateTime<Utc>,
}

/// Discord user information returned from the API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordUser {
    /// Discord user ID
    pub id: String,
    /// Discord username
    pub username: String,
    /// Discord discriminator
    #[serde(default)]
    pub discriminator: String,
    /// Discord avatar hash
    pub avatar: Option<String>,
    /// Whether the user is a bot
    #[serde(default)]
    pub bot: bool,
    /// Whether the user is an official Discord system user
    #[serde(default)]
    pub system: bool,
    /// Whether the user has MFA enabled
    #[serde(default)]
    pub mfa_enabled: bool,
    /// User locale
    pub locale: Option<String>,
    /// Whether the user's email is verified
    pub verified: Option<bool>,
    /// User email
    pub email: Option<String>,
    /// User flags
    #[serde(default)]
    pub flags: i32,
    /// User premium type
    #[serde(default)]
    pub premium_type: i32,
    /// User public flags
    #[serde(default)]
    pub public_flags: i32,
}

/// OAuth token response from Discord
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordTokenResponse {
    /// Access token
    pub access_token: String,
    /// Token type (usually "Bearer")
    pub token_type: String,
    /// When the token expires (in seconds)
    pub expires_in: i32,
    /// Refresh token
    pub refresh_token: String,
    /// Token scope
    pub scope: String,
}

/// JWT token claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    /// Subject (user ID)
    pub sub: String,
    /// Issued at (timestamp)
    pub iat: usize,
    /// Expires at (timestamp)
    pub exp: usize,
    /// Discord user information
    pub user: UserInfo,
}

/// User information stored in JWT token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    /// Discord user ID
    pub id: String,
    /// Discord username
    pub username: String,
    /// Discord avatar URL
    pub avatar_url: Option<String>,
    /// User's authorized guilds (server IDs)
    #[serde(default)]
    pub guilds: Vec<String>,
    /// User email
    pub email: Option<String>,
    /// Whether the user's email is verified
    pub verified: Option<bool>,
    /// User locale
    pub locale: Option<String>,
}

/// Authentication response sent to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    /// JWT token
    pub token: String,
    /// Token expiration time in seconds
    pub expires_in: usize,
    /// User information
    pub user: UserInfo,
}
