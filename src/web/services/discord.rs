// src/web/services/discord.rs
//! Discord API service for interacting with Discord's API

use crate::error::Error;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn as tracing_warn};
use chrono::Utc;

/// Structure to store cache metadata with the data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResponse<T> {
    /// The actual data
    pub data: T,
    /// When this data was cached (Unix timestamp)
    pub cached_at: i64,
    /// Source of the data ("api" or "cache")
    pub source: String,
    /// Optional ETag from the response
    pub etag: Option<String>,
}

/// Helper function to create a CachedResponse from API data
fn create_cached_response<T>(data: T, etag: Option<String>) -> CachedResponse<T> {
    CachedResponse {
        data,
        cached_at: Utc::now().timestamp(),
        source: "api".to_string(),
        etag,
    }
}

/// Discord Guild representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordGuild {
    /// Guild ID
    pub id: String,
    /// Guild name
    pub name: String,
    /// Guild icon hash
    pub icon: Option<String>,
    /// Icon hash returned in template object
    #[serde(rename = "icon_hash", default)]
    pub icon_hash: Option<String>,
    /// Splash hash
    #[serde(default)]
    pub splash: Option<String>,
    /// Discovery splash hash
    #[serde(rename = "discovery_splash", default)]
    pub discovery_splash: Option<String>,
    /// Whether the user is the owner
    pub owner: Option<bool>,
    /// ID of the guild owner
    #[serde(rename = "owner_id")]
    pub owner_id: Option<String>,
    /// Permissions for the user in the guild (may be missing when using bot token)
    #[serde(default)]
    pub permissions: Option<String>,
    /// Voice region ID (deprecated)
    #[serde(default)]
    pub region: Option<String>,
    /// ID of AFK channel
    #[serde(rename = "afk_channel_id", default)]
    pub afk_channel_id: Option<String>,
    /// AFK timeout in seconds
    #[serde(rename = "afk_timeout", default)]
    pub afk_timeout: Option<i32>,
    /// Widget enabled
    #[serde(rename = "widget_enabled", default)]
    pub widget_enabled: Option<bool>,
    /// Verification level
    #[serde(rename = "verification_level", default)]
    pub verification_level: Option<i32>,
    /// Features enabled for the guild
    #[serde(default)]
    pub features: Vec<String>,
    /// Approximate member count
    #[serde(rename = "approximate_member_count", default)]
    pub approximate_member_count: Option<i32>,
    /// Approximate presence count
    #[serde(rename = "approximate_presence_count", default)]
    pub approximate_presence_count: Option<i32>,
    
    // Allow any other fields to be deserialized without errors
    #[serde(flatten)]
    pub _extra: std::collections::HashMap<String, serde_json::Value>,
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

use crate::services::cache::{CacheService, CacheResult};
use std::sync::Arc;
use std::time::Duration;

/// Token type for Discord API requests
#[derive(Debug, Clone, Copy)]
pub enum TokenType {
    /// User token (OAuth2) - uses "Bearer" prefix
    User,
    /// Bot token - uses "Bot" prefix
    Bot
}

/// Discord application command structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordApplicationCommand {
    /// Command ID (if existing)
    pub id: Option<String>,
    /// Command name
    pub name: String,
    /// Command description
    pub description: String,
    /// Command options
    #[serde(default)]
    pub options: Vec<DiscordApplicationCommandOption>,
}

/// Discord application command option structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordApplicationCommandOption {
    /// Option type (1=SubCommand, 2=SubCommandGroup, 3=String, 4=Integer, etc.)
    #[serde(rename = "type")]
    pub option_type: i32,
    /// Option name
    pub name: String,
    /// Option description
    pub description: String,
    /// Whether option is required
    #[serde(default)]
    pub required: bool,
    /// Choices for the option
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub choices: Vec<DiscordApplicationCommandOptionChoice>,
    /// Sub-options for this option
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<DiscordApplicationCommandOption>,
}

/// Discord application command option choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordApplicationCommandOptionChoice {
    /// Choice name
    pub name: String,
    /// Choice value
    pub value: serde_json::Value,
}

/// Discord API service
pub struct DiscordService {
    /// HTTP client
    client: reqwest::Client,
    /// Discord token
    token: String,
    /// Token type (User or Bot)
    token_type: TokenType,
    /// API base URL
    api_base: String,
    /// Cache service (optional)
    cache: Option<Arc<CacheService>>,
    /// Application ID (for bot commands)
    application_id: Option<String>,
}

impl DiscordService {
    /// Create a new Discord API service with user token (OAuth2)
    pub fn new(token: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            token: token.into(),
            token_type: TokenType::User,
            api_base: "https://discord.com/api/v10".to_string(),
            cache: None,
            application_id: None,
        }
    }
    
    /// Create a new Discord API service with bot token
    pub fn new_bot(token: impl Into<String>, application_id: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            token: token.into(),
            token_type: TokenType::Bot,
            api_base: "https://discord.com/api/v10".to_string(),
            cache: None,
            application_id,
        }
    }
    
    /// Create a new Discord API service with user token and caching
    pub fn new_with_cache(token: impl Into<String>, cache: Arc<CacheService>) -> Self {
        let client = reqwest::Client::builder()
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            token: token.into(),
            token_type: TokenType::User,
            api_base: "https://discord.com/api/v10".to_string(),
            cache: Some(cache),
            application_id: None,
        }
    }
    
    /// Create a new Discord API service with bot token and caching
    pub fn new_bot_with_cache(token: impl Into<String>, cache: Arc<CacheService>, application_id: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            token: token.into(),
            token_type: TokenType::Bot,
            api_base: "https://discord.com/api/v10".to_string(),
            cache: Some(cache),
            application_id,
        }
    }

    /// Create authorization headers with the token using the appropriate prefix
    fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        
        // Use different prefix based on token type
        let auth_value = match self.token_type {
            TokenType::User => format!("Bearer {}", self.token),
            TokenType::Bot => format!("Bot {}", self.token),
        };
        
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value)
                .expect("Invalid header value"),
        );
        headers
    }

    /// Generate a hash of the token for use in cache keys
    fn token_hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.token.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get current user's guilds with rate limit handling and caching
    pub async fn get_current_user_guilds(&self, force_refresh: bool) -> Result<CachedResponse<Vec<DiscordGuild>>, Error> {
        let url = format!("{}/users/@me/guilds", self.api_base);
        let cache_key = format!("discord:guilds:{}", self.token_hash());
        let etag_key = format!("discord:guilds:etag:{}", self.token_hash());
        
        // Check cache first if not forcing refresh
        if !force_refresh {
            if let Some(cache) = &self.cache {
                debug!("Checking cache for key: {}", cache_key);
                
                match cache.get::<CachedResponse<Vec<DiscordGuild>>, _>(&cache_key).await {
                    Ok(cached) => {
                        debug!("Cache hit for Discord guilds");
                        // Clone the response and update source
                        let mut response = cached.value;
                        response.source = "cache".to_string();
                        return Ok(response);
                    },
                    Err(_) => {
                        debug!("Cache miss for Discord guilds");
                    }
                }
            }
        }
        
        debug!("Fetching guilds from Discord API: {}", url);
        
        // Handle rate limiting here with retries if needed
        let mut retries = 0;
        let max_retries = 3;
        
        loop {
            // Initialize the request
            let mut request = self.client.get(&url);
            request = request.headers(self.auth_headers());
            
            // Add ETag header if available in cache
            if let Some(cache) = &self.cache {
                if let Ok(cached_etag) = cache.get::<String, _>(&etag_key).await {
                    let etag = cached_etag.value;
                    debug!("Using cached ETag: {}", &etag);
                    request = request.header("If-None-Match", etag);
                }
            }
            
            let response = request.send().await.map_err(|e| {
                error!("Discord API request error: {}", e);
                Error::Unknown(format!("Discord API request error: {}", e))
            })?;
            
            // Check for 304 Not Modified response
            if response.status() == reqwest::StatusCode::NOT_MODIFIED {
                debug!("Discord API returned 304 Not Modified for guilds");
                
                // Return cached data since nothing has changed
                if let Some(cache) = &self.cache {
                    match cache.get::<CachedResponse<Vec<DiscordGuild>>, _>(&cache_key).await {
                        Ok(cached) => {
                            debug!("Using cached data - 304 Not Modified");
                            let mut response = cached.value;
                            response.source = "cache".to_string();
                            return Ok(response);
                        },
                        Err(_) => {
                            debug!("Cache inconsistency - 304 but no cached data");
                            // Fall through to regular response handling
                        }
                    }
                }
            }
            
            // Handle rate limiting
            if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                let retry_after = response.headers()
                    .get("retry-after")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(1.0);
                
                retries += 1;
                if retries > max_retries {
                    let text = response.text().await
                        .unwrap_or_else(|_| "Could not read response body".to_string());
                    
                    error!("Discord API rate limit exceeded after {} retries. Body: {}", max_retries, text);
                    return Err(Error::Unknown(format!("Discord API rate limit exceeded after {} retries", max_retries)));
                }
                
                info!("Rate limited by Discord API, retrying after {} seconds (attempt {}/{})", 
                      retry_after, retries, max_retries);
                
                // Sleep for the specified time before retrying
                tokio::time::sleep(std::time::Duration::from_secs_f64(retry_after)).await;
                continue;
            }
            
            // Handle other non-success responses
            if !response.status().is_success() {
                let status = response.status();
                let text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Could not read response body".to_string());

                error!("Discord API error: Status {}, Body: {}", status, text);
                return Err(Error::Unknown(format!("Discord API error: {}", status)));
            }
            
            // Get new ETag if present
            let new_etag = response.headers()
                .get("ETag")
                .and_then(|v| v.to_str().ok())
                .map(String::from);
                
            // Parse the response
            let guilds: Vec<DiscordGuild> = response.json().await.map_err(|e| {
                error!("Failed to parse Discord guilds response: {}", e);
                Error::Unknown(format!("Failed to parse Discord guilds response: {}", e))
            })?;

            // Create cached response
            let cached_response = create_cached_response(guilds, new_etag.clone());

            // Store in cache if available
            if let Some(cache) = &self.cache {
                let ttl = Some(Duration::from_secs(300)); // 5 minute cache TTL
                
                if let Err(e) = cache.set_serialized(&cache_key, &cached_response, ttl, false).await {
                    tracing_warn!("Failed to cache Discord guilds: {}", e);
                } else {
                    debug!("Cached Discord guilds for 5 minutes");
                }
                
                // Also store ETag separately if we have one
                if let Some(etag_value) = new_etag {
                    if let Err(e) = cache.set(&etag_key, etag_value, ttl, false).await {
                        tracing_warn!("Failed to cache Discord guilds ETag: {}", e);
                    }
                }
            }

            return Ok(cached_response);
        }
    }
    
    /// Method to invalidate guilds cache
    pub async fn invalidate_guilds_cache(&self) -> Result<(), Error> {
        if let Some(cache) = &self.cache {
            let cache_key = format!("discord:guilds:{}", self.token_hash());
            let etag_key = format!("discord:guilds:etag:{}", self.token_hash());
            
            debug!("Invalidating guild caches");
            
            // Just create cache key with very short TTL to effectively invalidate
            let short_ttl = Some(Duration::from_secs(1));
            
            // Ignore errors during cache invalidation
            let _ = cache.set_serialized(&cache_key, &Vec::<DiscordGuild>::new(), short_ttl, true).await;
            let _ = cache.set(&etag_key, String::new(), short_ttl, true).await;
        }
        Ok(())
    }

    /// Get detailed information about a specific guild with caching
    pub async fn get_guild(&self, guild_id: &str, force_refresh: bool) -> Result<CachedResponse<DiscordGuild>, Error> {
        let url = format!("{}/guilds/{}?with_counts=true", self.api_base, guild_id);
        let cache_key = format!("discord:guild:{}:{}", self.token_hash(), guild_id);
        let etag_key = format!("discord:guild:etag:{}:{}", self.token_hash(), guild_id);
        
        // Check cache first if not forcing refresh
        if !force_refresh {
            if let Some(cache) = &self.cache {
                debug!("Checking cache for key: {}", cache_key);
                
                match cache.get::<CachedResponse<DiscordGuild>, _>(&cache_key).await {
                    Ok(cached) => {
                        debug!("Cache hit for Discord guild details");
                        // Clone the response and update source
                        let mut response = cached.value;
                        response.source = "cache".to_string();
                        return Ok(response);
                    },
                    Err(_) => {
                        debug!("Cache miss for Discord guild details");
                    }
                }
            }
        }
        
        debug!("Fetching guild details from Discord API: {}", url);

        // Initialize the request
        let mut request = self.client.get(&url);
        request = request.headers(self.auth_headers());
        
        // Add ETag header if available in cache
        if let Some(cache) = &self.cache {
            if let Ok(cached_etag) = cache.get::<String, _>(&etag_key).await {
                let etag = cached_etag.value;
                debug!("Using cached ETag for guild: {}", &etag);
                request = request.header("If-None-Match", etag);
            }
        }
        
        let response = request.send().await.map_err(|e| {
            error!("Discord API request error: {}", e);
            Error::Unknown(format!("Discord API request error: {}", e))
        })?;
        
        // Check for 304 Not Modified
        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            debug!("Discord API returned 304 Not Modified for guild");
            
            // Return cached data since nothing has changed
            if let Some(cache) = &self.cache {
                match cache.get::<CachedResponse<DiscordGuild>, _>(&cache_key).await {
                    Ok(cached) => {
                        debug!("Using cached data - 304 Not Modified");
                        let mut response = cached.value;
                        response.source = "cache".to_string();
                        return Ok(response);
                    },
                    Err(_) => {
                        debug!("Cache inconsistency - 304 but no cached data");
                        // Continue to regular request
                    }
                }
            }
        }

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Could not read response body".to_string());

            error!("Discord API error: Status {}, Body: {}", status, text);
            return Err(Error::Unknown(format!("Discord API error: {}", status)));
        }

        // Get new ETag if present
        let new_etag = response.headers()
            .get("ETag")
            .and_then(|v| v.to_str().ok())
            .map(String::from);
            
        // Get the response body as text first for better error handling
        let body = response.text().await.map_err(|e| {
            error!("Failed to read Discord guild response body: {}", e);
            Error::Unknown(format!("Failed to read Discord guild response body: {}", e))
        })?;
        
        // Try to parse the JSON body with detailed error reporting
        let guild: DiscordGuild = match serde_json::from_str(&body) {
            Ok(parsed) => parsed,
            Err(e) => {
                error!("Failed to parse Discord guild response: {}, Body: {}", e, body);
                return Err(Error::Unknown(format!("Failed to parse Discord guild response: {}", e)));
            }
        };

        // Create cached response
        let cached_response = create_cached_response(guild, new_etag.clone());

        // Store in cache if available
        if let Some(cache) = &self.cache {
            let ttl = Some(Duration::from_secs(300)); // 5 minute cache TTL
            
            if let Err(e) = cache.set_serialized(&cache_key, &cached_response, ttl, false).await {
                tracing_warn!("Failed to cache Discord guild: {}", e);
            } else {
                debug!("Cached Discord guild for 5 minutes");
            }
            
            // Also store ETag separately
            if let Some(etag_value) = new_etag {
                if let Err(e) = cache.set(&etag_key, etag_value, ttl, false).await {
                    tracing_warn!("Failed to cache Discord guild ETag: {}", e);
                }
            }
        }

        Ok(cached_response)
    }

    /// Get channels for a specific guild with caching
    pub async fn get_guild_channels(&self, guild_id: &str, force_refresh: bool) -> Result<CachedResponse<Vec<DiscordChannel>>, Error> {
        let url = format!("{}/guilds/{}/channels", self.api_base, guild_id);
        let cache_key = format!("discord:channels:{}:{}", self.token_hash(), guild_id);
        let etag_key = format!("discord:channels:etag:{}:{}", self.token_hash(), guild_id);
        
        // Check cache first if not forcing refresh
        if !force_refresh {
            if let Some(cache) = &self.cache {
                debug!("Checking cache for key: {}", cache_key);
                
                match cache.get::<CachedResponse<Vec<DiscordChannel>>, _>(&cache_key).await {
                    Ok(cached) => {
                        debug!("Cache hit for Discord channels");
                        // Clone the response and update source
                        let mut response = cached.value;
                        response.source = "cache".to_string();
                        return Ok(response);
                    },
                    Err(_) => {
                        debug!("Cache miss for Discord channels");
                    }
                }
            }
        }
        
        debug!("Fetching guild channels from Discord API: {}", url);

        // Initialize the request
        let mut request = self.client.get(&url);
        request = request.headers(self.auth_headers());
        
        // Add ETag header if available in cache
        if let Some(cache) = &self.cache {
            if let Ok(cached_etag) = cache.get::<String, _>(&etag_key).await {
                let etag = cached_etag.value;
                debug!("Using cached ETag for channels: {}", &etag);
                request = request.header("If-None-Match", etag);
            }
        }
        
        let response = request.send().await.map_err(|e| {
            error!("Discord API request error: {}", e);
            Error::Unknown(format!("Discord API request error: {}", e))
        })?;
        
        // Check for 304 Not Modified
        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            debug!("Discord API returned 304 Not Modified for channels");
            
            // Return cached data since nothing has changed
            if let Some(cache) = &self.cache {
                match cache.get::<CachedResponse<Vec<DiscordChannel>>, _>(&cache_key).await {
                    Ok(cached) => {
                        debug!("Using cached data - 304 Not Modified");
                        let mut response = cached.value;
                        response.source = "cache".to_string();
                        return Ok(response);
                    },
                    Err(_) => {
                        debug!("Cache inconsistency - 304 but no cached data");
                        // Continue to regular request
                    }
                }
            }
        }

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Could not read response body".to_string());

            error!("Discord API error: Status {}, Body: {}", status, text);
            return Err(Error::Unknown(format!("Discord API error: {}", status)));
        }

        // Get new ETag if present
        let new_etag = response.headers()
            .get("ETag")
            .and_then(|v| v.to_str().ok())
            .map(String::from);
            
        // Parse the response
        let channels: Vec<DiscordChannel> = response.json().await.map_err(|e| {
            error!("Failed to parse Discord channels response: {}", e);
            Error::Unknown(format!("Failed to parse Discord channels response: {}", e))
        })?;

        // Create cached response
        let cached_response = create_cached_response(channels, new_etag.clone());

        // Store in cache if available
        if let Some(cache) = &self.cache {
            let ttl = Some(Duration::from_secs(300)); // 5 minute cache TTL
            
            if let Err(e) = cache.set_serialized(&cache_key, &cached_response, ttl, false).await {
                tracing_warn!("Failed to cache Discord channels: {}", e);
            } else {
                debug!("Cached Discord channels for 5 minutes");
            }
            
            // Also store ETag separately
            if let Some(etag_value) = new_etag {
                if let Err(e) = cache.set(&etag_key, etag_value, ttl, false).await {
                    tracing_warn!("Failed to cache Discord channels ETag: {}", e);
                }
            }
        }

        Ok(cached_response)
    }
    
    /// Method to invalidate channels cache
    pub async fn invalidate_channels_cache(&self, guild_id: &str) -> Result<(), Error> {
        if let Some(cache) = &self.cache {
            let cache_key = format!("discord:channels:{}:{}", self.token_hash(), guild_id);
            let etag_key = format!("discord:channels:etag:{}:{}", self.token_hash(), guild_id);
            
            debug!("Invalidating channels cache for guild {}", guild_id);
            
            // Just create cache key with very short TTL to effectively invalidate
            let short_ttl = Some(Duration::from_secs(1));
            
            // Ignore errors during cache invalidation
            let _ = cache.set_serialized(&cache_key, &Vec::<DiscordChannel>::new(), short_ttl, true).await;
            let _ = cache.set(&etag_key, String::new(), short_ttl, true).await;
        }
        Ok(())
    }

    /// Get members of a specific guild
    pub async fn get_guild_members(&self, guild_id: &str, limit: usize) -> Result<Vec<DiscordGuildMember>, Error> {
        let url = format!("{}/guilds/{}/members?limit={}", self.api_base, guild_id, limit);
        debug!("Fetching guild members from Discord API: {}", url);

        let response = self.client
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

    /// Get current user information with caching and ETag support
    pub async fn get_current_user(&self, force_refresh: bool) -> Result<CachedResponse<DiscordUser>, Error> {
        let url = format!("{}/users/@me", self.api_base);
        let cache_key = format!("discord:user:{}", self.token_hash());
        let etag_key = format!("discord:user:etag:{}", self.token_hash());
        
        // Check cache first if not forcing refresh
        if !force_refresh {
            if let Some(cache) = &self.cache {
                debug!("Checking cache for key: {}", cache_key);
                
                match cache.get::<CachedResponse<DiscordUser>, _>(&cache_key).await {
                    Ok(cached) => {
                        debug!("Cache hit for Discord user");
                        // Clone the response and update source
                        let mut response = cached.value;
                        response.source = "cache".to_string();
                        return Ok(response);
                    },
                    Err(_) => {
                        debug!("Cache miss for Discord user");
                    }
                }
            }
        }
        
        debug!("Fetching current user from Discord API: {}", url);
        
        // Initialize the request
        let mut request = self.client.get(&url);
        request = request.headers(self.auth_headers());
        
        // Add ETag header if available in cache
        if let Some(cache) = &self.cache {
            if let Ok(cached_etag) = cache.get::<String, _>(&etag_key).await {
                let etag = cached_etag.value;
                debug!("Using cached ETag for user: {}", &etag);
                request = request.header("If-None-Match", etag);
            }
        }
        
        let response = request.send().await.map_err(|e| {
            error!("Discord API request error: {}", e);
            Error::Unknown(format!("Discord API request error: {}", e))
        })?;
        
        // Check for 304 Not Modified
        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            debug!("Discord API returned 304 Not Modified for user");
            
            // Return cached data since nothing has changed
            if let Some(cache) = &self.cache {
                match cache.get::<CachedResponse<DiscordUser>, _>(&cache_key).await {
                    Ok(cached) => {
                        debug!("Using cached data - 304 Not Modified");
                        let mut response = cached.value;
                        response.source = "cache".to_string();
                        return Ok(response);
                    },
                    Err(_) => {
                        debug!("Cache inconsistency - 304 but no cached data");
                        // Continue to regular request
                    }
                }
            }
        }

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Could not read response body".to_string());

            error!("Discord API error: Status {}, Body: {}", status, text);
            return Err(Error::Unknown(format!("Discord API error: {}", status)));
        }

        // Get new ETag if present
        let new_etag = response.headers()
            .get("ETag")
            .and_then(|v| v.to_str().ok())
            .map(String::from);
            
        // Parse the response
        let user: DiscordUser = response.json().await.map_err(|e| {
            error!("Failed to parse Discord user response: {}", e);
            Error::Unknown(format!("Failed to parse Discord user response: {}", e))
        })?;

        // Create cached response
        let cached_response = create_cached_response(user, new_etag.clone());

        // Store in cache if available
        if let Some(cache) = &self.cache {
            let ttl = Some(Duration::from_secs(300)); // 5 minute cache TTL
            
            if let Err(e) = cache.set_serialized(&cache_key, &cached_response, ttl, false).await {
                tracing_warn!("Failed to cache Discord user: {}", e);
            } else {
                debug!("Cached Discord user for 5 minutes");
            }
            
            // Also store ETag separately
            if let Some(etag_value) = new_etag {
                if let Err(e) = cache.set(&etag_key, etag_value, ttl, false).await {
                    tracing_warn!("Failed to cache Discord user ETag: {}", e);
                }
            }
        }

        Ok(cached_response)
    }
    
    /// Method to invalidate user cache
    pub async fn invalidate_user_cache(&self) -> Result<(), Error> {
        if let Some(cache) = &self.cache {
            let cache_key = format!("discord:user:{}", self.token_hash());
            let etag_key = format!("discord:user:etag:{}", self.token_hash());
            
            debug!("Invalidating user caches");
            
            // Just create cache key with very short TTL to effectively invalidate
            let short_ttl = Some(Duration::from_secs(1));
            
            // Ignore errors during cache invalidation
            let _ = cache.set_serialized(&cache_key, &Vec::<DiscordUser>::new(), short_ttl, true).await;
            let _ = cache.set(&etag_key, String::new(), short_ttl, true).await;
        }
        Ok(())
    }
    
    /// Set application ID for the bot
    pub fn set_application_id(&mut self, application_id: String) {
        self.application_id = Some(application_id);
    }
    
    /// Get application ID
    pub fn application_id(&self) -> Option<&str> {
        self.application_id.as_deref()
    }
    
    /// Get the list of global application commands
    pub async fn get_global_commands(&self) -> Result<Vec<DiscordApplicationCommand>, Error> {
        // Ensure we have an application ID
        let app_id = match &self.application_id {
            Some(id) => id,
            None => {
                error!("Cannot get global commands: application ID not set");
                return Err(Error::Unknown("Application ID not set".to_string()));
            }
        };
        
        // Endpoint for getting global commands
        let url = format!(
            "{}/applications/{}/commands", 
            self.api_base, 
            app_id
        );
        
        debug!("Getting global commands");
        
        // Send GET request 
        let response = self.client
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

            error!("Discord API error getting global commands: Status {}, Body: {}", status, text);
            return Err(Error::Unknown(format!("Discord API error getting global commands: {}", status)));
        }
        
        // Parse the response
        let commands: Vec<DiscordApplicationCommand> = response.json().await.map_err(|e| {
            error!("Failed to parse Discord global commands response: {}", e);
            Error::Unknown(format!("Failed to parse Discord global commands response: {}", e))
        })?;
        
        debug!("Retrieved {} global commands", commands.len());
        Ok(commands)
    }
    
    /// Sync commands for a guild based on enabled settings
    pub async fn sync_guild_commands(
        &self, 
        guild_id: &str, 
        enabled_commands: Vec<DiscordApplicationCommand>
    ) -> Result<(), Error> {
        // Ensure we have an application ID
        let app_id = match &self.application_id {
            Some(id) => id,
            None => {
                error!("Cannot sync guild commands: application ID not set");
                return Err(Error::Unknown("Application ID not set".to_string()));
            }
        };
        
        // Endpoint for bulk overwriting guild commands
        let url = format!(
            "{}/applications/{}/guilds/{}/commands", 
            self.api_base, 
            app_id, 
            guild_id
        );
        
        debug!("Syncing {} commands to guild {}", enabled_commands.len(), guild_id);
        
        // Send PUT request to overwrite all commands
        let response = self.client
            .put(&url)
            .headers(self.auth_headers())
            .json(&enabled_commands)
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

            error!("Discord API error syncing commands: Status {}, Body: {}", status, text);
            return Err(Error::Unknown(format!("Discord API error syncing commands: {}", status)));
        }
        
        info!("Successfully synced {} commands to guild {}", enabled_commands.len(), guild_id);
        Ok(())
    }
    
    /// Get the list of registered commands for a guild
    pub async fn get_guild_commands(&self, guild_id: &str) -> Result<Vec<DiscordApplicationCommand>, Error> {
        // Ensure we have an application ID
        let app_id = match &self.application_id {
            Some(id) => id,
            None => {
                error!("Cannot get guild commands: application ID not set");
                return Err(Error::Unknown("Application ID not set".to_string()));
            }
        };
        
        // Endpoint for getting guild commands
        let url = format!(
            "{}/applications/{}/guilds/{}/commands", 
            self.api_base, 
            app_id, 
            guild_id
        );
        
        debug!("Getting commands for guild {}", guild_id);
        
        // Send GET request 
        let response = self.client
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

            error!("Discord API error getting commands: Status {}, Body: {}", status, text);
            return Err(Error::Unknown(format!("Discord API error getting commands: {}", status)));
        }
        
        // Parse the response
        let commands: Vec<DiscordApplicationCommand> = response.json().await.map_err(|e| {
            error!("Failed to parse Discord commands response: {}", e);
            Error::Unknown(format!("Failed to parse Discord commands response: {}", e))
        })?;
        
        debug!("Retrieved {} commands from guild {}", commands.len(), guild_id);
        Ok(commands)
    }
    
    /// Clear all global commands
    pub async fn clear_global_commands(&self) -> Result<(), Error> {
        // Ensure we have an application ID
        let app_id = match &self.application_id {
            Some(id) => id,
            None => {
                error!("Cannot clear global commands: application ID not set");
                return Err(Error::Unknown("Application ID not set".to_string()));
            }
        };
        
        // Endpoint for bulk overwriting global commands
        let url = format!(
            "{}/applications/{}/commands", 
            self.api_base, 
            app_id
        );
        
        info!("Clearing all global commands");
        
        // Send PUT request with empty array to remove all commands
        let response = self.client
            .put(&url)
            .headers(self.auth_headers())
            .json(&Vec::<DiscordApplicationCommand>::new()) // Empty array
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

            error!("Discord API error clearing global commands: Status {}, Body: {}", status, text);
            return Err(Error::Unknown(format!("Discord API error clearing global commands: {}", status)));
        }
        
        info!("Successfully cleared all global commands");
        Ok(())
    }
}
