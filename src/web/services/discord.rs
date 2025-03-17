// src/web/services/discord.rs
//! Discord API service for interacting with Discord's API

use crate::error::Error;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

/// Discord Guild representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordGuild {
    /// Guild ID
    pub id: String,
    /// Guild name
    pub name: String,
    /// Guild icon hash
    pub icon: Option<String>,
    /// Whether the user is the owner
    pub owner: Option<bool>,
    /// Permissions for the user in the guild
    pub permissions: String,
    /// Features enabled for the guild
    pub features: Vec<String>,
}

/// Discord User representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordUser {
    /// User ID
    pub id: String,
    /// Username
    pub username: String,
    /// User's discriminator
    pub discriminator: String,
    /// User's avatar hash
    pub avatar: Option<String>,
    /// User's email
    pub email: Option<String>,
    /// Whether the user's email is verified
    pub verified: Option<bool>,
    /// User locale
    pub locale: Option<String>,
    /// Whether the user has MFA enabled
    pub mfa_enabled: Option<bool>,
}

/// Discord Guild Member representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordGuildMember {
    /// User object for this member
    pub user: Option<DiscordUser>,
    /// Member's nickname in the guild
    pub nick: Option<String>,
    /// Member's roles
    pub roles: Vec<String>,
    /// When the user joined the guild
    pub joined_at: String,
    /// Whether the member is muted
    pub mute: bool,
    /// Whether the member is deafened
    pub deaf: bool,
    /// Whether the member has passed guild membership screening
    pub pending: Option<bool>,
}

/// Discord Channel representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordChannel {
    /// Channel ID
    pub id: String,
    /// Channel type
    #[serde(rename = "type")]
    pub channel_type: i32,
    /// Guild ID of the channel
    pub guild_id: Option<String>,
    /// Channel position
    pub position: Option<i32>,
    /// Channel name
    pub name: Option<String>,
    /// Channel topic
    pub topic: Option<String>,
}

/// Discord API service
pub struct DiscordService {
    /// HTTP client
    client: reqwest::Client,
    /// Discord token
    token: String,
    /// API base URL
    api_base: String,
}

impl DiscordService {
    /// Create a new Discord API service
    pub fn new(token: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            token: token.into(),
            api_base: "https://discord.com/api/v10".to_string(),
        }
    }

    /// Create authorization headers with the token
    fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.token))
                .expect("Invalid header value"),
        );
        headers
    }

    /// Get current user's guilds
    pub async fn get_current_user_guilds(&self) -> Result<Vec<DiscordGuild>, Error> {
        let url = format!("{}/users/@me/guilds", self.api_base);
        debug!("Fetching guilds from Discord API: {}", url);

        let response = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .map_err(|e| {
                error!("Discord API request error: {}", e);
                Error::Unknown(format!("Discord API request error: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Could not read response body".to_string());

            error!("Discord API error: Status {}, Body: {}", status, text);
            return Err(Error::Unknown(format!("Discord API error: {}", status)));
        }

        // Parse the response
        let guilds: Vec<DiscordGuild> = response.json().await.map_err(|e| {
            error!("Failed to parse Discord guilds response: {}", e);
            Error::Unknown(format!("Failed to parse Discord guilds response: {}", e))
        })?;

        Ok(guilds)
    }

    /// Get detailed information about a specific guild
pub async fn get_guild(&self, guild_id: &str) -> Result<DiscordGuild, Error> {
        let url = format!("{}/guilds/{}?with_counts=true", self.api_base, guild_id);
        debug!("Fetching guild details from Discord API: {}", url);

let response = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .map_err(|e| {
                error!("Discord API request error: {}", e);
                Error::Unknown(format!("Discord API request error: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Could not read response body".to_string());

            error!("Discord API error: Status {}, Body: {}", status, text);
            return Err(Error::Unknown(format!("Discord API error: {}", status)));
        }

        // Parse the response
        let guild: DiscordGuild = response.json().await.map_err(|e| {
            error!("Failed to parse Discord guild response: {}", e);
            Error::Unknown(format!("Failed to parse Discord guild response: {}", e))
        })?;

        Ok(guild)
    }

    /// Get channels for a specific guild
    pub async fn get_guild_channels(&self, guild_id: &str) -> Result<Vec<DiscordChannel>, Error> {
        let url = format!("{}/guilds/{}/channels", self.api_base, guild_id);
        debug!("Fetching guild channels from Discord API: {}", url);

        let response = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .map_err(|e| {
                error!("Discord API request error: {}", e);
                Error::Unknown(format!("Discord API request error: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Could not read response body".to_string());

            error!("Discord API error: Status {}, Body: {}", status, text);
            return Err(Error::Unknown(format!("Discord API error: {}", status)));
        }

        // Parse the response
        let channels: Vec<DiscordChannel> = response.json().await.map_err(|e| {
            error!("Failed to parse Discord channels response: {}", e);
            Error::Unknown(format!("Failed to parse Discord channels response: {}", e))
        })?;

        Ok(channels)
    }

    /// Get members of a specific guild
pub async fn get_guild_members(&self, guild_id: &str, limit: usize) -> Result<Vec<DiscordGuildMember>, Error> {
        let url = format!("{}/guilds/{}/members?limit={}", self.api_base, guild_id, limit);
        debug!("Fetching guild members from Discord API: {}", url);

        let response = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .map_err(|e| {
                error!("Discord API request error: {}", e);
                Error::Unknown(format!("Discord API request error: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Could not read response body".to_string());

            error!("Discord API error: Status {}, Body: {}", status, text);
            return Err(Error::Unknown(format!("Discord API error: {}", status)));
        }

        // Parse the response
        let members: Vec<DiscordGuildMember> = response.json().await.map_err(|e| {
            error!("Failed to parse Discord guild members response: {}", e);
            Error::Unknown(format!("Failed to parse Discord guild members response: {}", e))
        })?;

        info!("Fetched {} members for guild {}", members.len(), guild_id);
        Ok(members)
    }

    /// Get current user information
    pub async fn get_current_user(&self) -> Result<DiscordUser, Error> {
        let url = format!("{}/users/@me", self.api_base);
        debug!("Fetching current user from Discord API: {}", url);

        let response = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .map_err(|e| {
                error!("Discord API request error: {}", e);
                Error::Unknown(format!("Discord API request error: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Could not read response body".to_string());

            error!("Discord API error: Status {}, Body: {}", status, text);
            return Err(Error::Unknown(format!("Discord API error: {}", status)));
        }

        // Parse the response
        let user: DiscordUser = response.json().await.map_err(|e| {
            error!("Failed to parse Discord user response: {}", e);
            Error::Unknown(format!("Failed to parse Discord user response: {}", e))
        })?;

        Ok(user)
    }
}
