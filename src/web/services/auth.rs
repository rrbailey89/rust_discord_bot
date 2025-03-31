// src/web/services/auth.rs
//! Authentication services for web API

use crate::config::WebConfig;
use crate::error::Error;
use crate::services::database::DatabaseService;
use crate::web::models::auth::{UserSession, DiscordUser, DiscordTokenResponse, TokenClaims, UserInfo, AuthResponse};
use crate::web::services::discord::DiscordGuild;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, Algorithm};
use oauth2::{
    AuthUrl, ClientId, ClientSecret, TokenUrl, RedirectUrl, AuthorizationCode, CsrfToken,
    basic::BasicClient, TokenResponse, reqwest::async_http_client
};
use std::time::{SystemTime, UNIX_EPOCH};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use tracing::{info, error, debug};

/// Discord OAuth configuration
#[derive(Clone, Debug)]
pub struct DiscordOAuthConfig {
    /// Discord client ID
    pub client_id: String,
    /// Discord client secret
    pub client_secret: String,
    /// Redirect URI
    pub redirect_uri: String,
    /// JWT secret key for token generation
    pub jwt_secret: String,
    /// JWT token expiration in seconds
    pub jwt_expiration: u64,
}

/// Conversion from WebConfig to DiscordOAuthConfig
impl From<WebConfig> for DiscordOAuthConfig {
    fn from(config: WebConfig) -> Self {
        Self {
            client_id: config.discord_client_id,
            client_secret: config.discord_client_secret,
            redirect_uri: config.oauth_redirect_uri,
            jwt_secret: config.jwt_secret,
            jwt_expiration: config.jwt_expiration,
        }
    }
}

/// Authentication service
#[derive(Clone)]
pub struct AuthService {
    /// Database service
    db: DatabaseService,
    /// OAuth configuration
    config: DiscordOAuthConfig,
    /// HTTP client
    http_client: reqwest::Client,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new(db: DatabaseService, config: DiscordOAuthConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            db,
            config,
            http_client,
        }
    }
    
    /// Create an OAuth client for Discord authentication
    pub fn create_oauth_client(&self) -> BasicClient {
        BasicClient::new(
            ClientId::new(self.config.client_id.clone()),
            Some(ClientSecret::new(self.config.client_secret.clone())),
            AuthUrl::new("https://discord.com/api/oauth2/authorize".to_string())
                .expect("Invalid authorization endpoint URL"),
            Some(TokenUrl::new("https://discord.com/api/oauth2/token".to_string())
                .expect("Invalid token endpoint URL"))
        )
        .set_redirect_uri(RedirectUrl::new(self.config.redirect_uri.clone())
            .expect("Invalid redirect URL"))
    }
    
    /// Generate an authorization URL for Discord OAuth
    pub fn get_authorization_url(&self) -> (String, String) {
        let client = self.create_oauth_client();
        
        // Create a CSRF state token
        let state = oauth2::CsrfToken::new_random().secret().to_string();
        
        // Generate the authorization URL
        let csrf_token = CsrfToken::new_random();
        let (auth_url, _) = client
            .authorize_url(|| csrf_token.clone())
            .add_scope(oauth2::Scope::new("identify".to_string()))
            .add_scope(oauth2::Scope::new("guilds".to_string()))
            .add_scope(oauth2::Scope::new("email".to_string()))          // Request email access
            .add_scope(oauth2::Scope::new("guilds.members.read".to_string()))  // Request guild members access
            .url();
        
        info!("Generated Discord OAuth URL with additional scopes (email, guilds.members.read)");
        (auth_url.to_string(), state)
    }
    
    /// Exchange an authorization code for tokens
    pub async fn exchange_code(&self, code: String) -> Result<DiscordTokenResponse, Error> {
        let client = self.create_oauth_client();
        
        // Exchange the code for a token
        let token_result = client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await
            .map_err(|e| Error::Unknown(format!("OAuth token exchange error: {}", e)))?;
        
        // Convert to our token response format
        let token = DiscordTokenResponse {
            access_token: token_result.access_token().secret().clone(),
            token_type: format!("{:?}", token_result.token_type()),
            expires_in: token_result.expires_in().unwrap_or_default().as_secs() as i32,
            refresh_token: token_result.refresh_token()
                .map(|t| t.secret().clone())
                .unwrap_or_default(),
            scope: token_result.scopes()
                .map(|s| s.iter().map(|scope| scope.to_string()).collect::<Vec<_>>().join(" "))
                .unwrap_or_default(),
        };
        
        Ok(token)
    }
    
    /// Fetch user information from Discord API
    pub async fn fetch_discord_user(&self, access_token: &str) -> Result<DiscordUser, Error> {
        // Create headers with authorization
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", access_token))
                .map_err(|e| Error::Unknown(format!("Invalid header value: {}", e)))?,
        );
        
        // Fetch user data from Discord API
        let response = self.http_client
            .get("https://discord.com/api/v10/users/@me")
            .headers(headers)
            .send()
            .await
            .map_err(|e| Error::Unknown(format!("Discord API request error: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await
                .unwrap_or_else(|_| "Could not read response body".to_string());
                
            error!("Discord API error: Status {}, Body: {}", status, text);
            return Err(Error::Unknown(format!("Discord API error: {}", status)));
        }
        
        // Parse the response
        let user: DiscordUser = response.json().await
            .map_err(|e| Error::Unknown(format!("Failed to parse Discord user response: {}", e)))?;
            
        Ok(user)
    }
    
    /// Fetch user's guilds from Discord API with rate limit handling
    pub async fn fetch_discord_guilds(&self, access_token: &str) -> Result<Vec<DiscordGuild>, Error> {
        // Create headers with authorization
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", access_token))
                .map_err(|e| Error::Unknown(format!("Invalid header value: {}", e)))?,
        );
        
        let url = "https://discord.com/api/v10/users/@me/guilds";
        let mut retries = 0;
        let max_retries = 3;
        
        loop {
            // Fetch guilds data from Discord API
            let response = self.http_client
                .get(url)
                .headers(headers.clone())
                .send()
                .await
                .map_err(|e| Error::Unknown(format!("Discord API request error: {}", e)))?;
                
            if response.status().is_success() {
                // Parse the response
                let guilds: Vec<DiscordGuild> = response.json().await
                    .map_err(|e| Error::Unknown(format!("Failed to parse Discord guilds response: {}", e)))?;
                    
                return Ok(guilds);
            } else if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                // Handle rate limiting
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
                
                // Log that we're making a retry attempt
                debug!("Attempting retry #{} for Discord API guild request", retries);
                continue;
            } else {
                // Other error
                let status = response.status();
                let text = response.text().await
                    .unwrap_or_else(|_| "Could not read response body".to_string());
                    
                error!("Discord API error when fetching guilds: Status {}, Body: {}", status, text);
                return Err(Error::Unknown(format!("Discord API error: {}", status)));
            }
        }
    }
    
    /// Store user session in the database
    pub async fn store_user_session(
        &self,
        user_id: i64,
        token: &DiscordTokenResponse
    ) -> Result<(), Error> {
        let client = self.db.get_client().await?;
        
        // Calculate token expiration time
        let expires_at = Utc::now() + Duration::seconds(token.expires_in as i64);
        
        // Store in database
        client.execute(
            "INSERT INTO user_sessions (user_id, discord_token, token_expires_at, refresh_token, last_login)
             VALUES ($1, $2, $3, $4, NOW())
             ON CONFLICT (user_id) DO UPDATE SET
                discord_token = EXCLUDED.discord_token,
                token_expires_at = EXCLUDED.token_expires_at,
                refresh_token = EXCLUDED.refresh_token,
                last_login = EXCLUDED.last_login",
            &[
                &user_id,
                &token.access_token,
                &expires_at,
                &token.refresh_token,
            ],
        ).await?;
        
        Ok(())
    }
    
    /// Get a user session from the database
    pub async fn get_user_session(&self, user_id: i64) -> Result<Option<UserSession>, Error> {
        let client = self.db.get_client().await?;
        
        let row = client.query_opt(
            "SELECT user_id, discord_token, token_expires_at, refresh_token, last_login
             FROM user_sessions
             WHERE user_id = $1",
            &[&user_id],
        ).await?;
        
        Ok(row.map(|r| UserSession {
            user_id: r.get(0),
            discord_token: r.get(1),
            token_expires_at: r.get(2),
            refresh_token: r.get(3),
            last_login: r.get(4),
        }))
    }
    
    /// Refresh a Discord token
    pub async fn refresh_discord_token(&self, refresh_token: &str) -> Result<DiscordTokenResponse, Error> {
        let client = self.create_oauth_client();
        
        // Refresh the token
        let token_result = client
            .exchange_refresh_token(&oauth2::RefreshToken::new(refresh_token.to_string()))
            .request_async(async_http_client)
            .await
            .map_err(|e| Error::Unknown(format!("OAuth token refresh error: {}", e)))?;
            
        // Convert to our token response format
        let token = DiscordTokenResponse {
            access_token: token_result.access_token().secret().clone(),
            token_type: format!("{:?}", token_result.token_type()),
            expires_in: token_result.expires_in().unwrap_or_default().as_secs() as i32,
            refresh_token: token_result.refresh_token()
                .map(|t| t.secret().clone())
                .unwrap_or_else(|| refresh_token.to_string()),
            scope: token_result.scopes()
                .map(|s| s.iter().map(|scope| scope.to_string()).collect::<Vec<_>>().join(" "))
                .unwrap_or_default(),
        };
        
        Ok(token)
    }
    
    /// Generate a JWT token
    pub fn generate_jwt(&self, user: &DiscordUser, guild_ids: Vec<String>) -> Result<(String, usize), Error> {
        // Calculate expiration time
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as usize;
            
        let exp = now + self.config.jwt_expiration as usize;
        
        // Build the avatar URL if available
        let avatar_url = user.avatar.as_ref().map(|avatar| {
            format!(
                "https://cdn.discordapp.com/avatars/{}/{}.png",
                user.id, avatar
            )
        });
        
        // Create claims
        let claims = TokenClaims {
            sub: user.id.clone(),
            iat: now,
            exp,
            user: UserInfo {
                id: user.id.clone(),
                username: user.username.clone(),
                avatar_url,
                guilds: guild_ids,  // Use the guild IDs
                email: user.email.clone(),
                verified: user.verified,
                locale: user.locale.clone(),
            },
        };
        
        // Generate the token
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|e| Error::Unknown(format!("JWT encoding error: {}", e)))?;
        
        Ok((token, self.config.jwt_expiration as usize))
    }
    
    /// Overloaded version of generate_jwt that takes individual user fields
    pub fn generate_jwt_from_info(
        &self, 
        user_id: &str, 
        username: &str, 
        avatar_url: Option<String>, 
        guilds: Vec<String>
    ) -> Result<(String, usize), Error> {
        // Calculate expiration time
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as usize;
            
        let exp = now + self.config.jwt_expiration as usize;
        
        // Create claims
        let claims = TokenClaims {
            sub: user_id.to_string(),
            iat: now,
            exp,
            user: UserInfo {
                id: user_id.to_string(),
                username: username.to_string(),
                avatar_url,
                guilds,
                email: None,
                verified: None,
                locale: None,
            },
        };
        
        // Generate the token
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|e| Error::Unknown(format!("JWT encoding error: {}", e)))?;
        
        Ok((token, self.config.jwt_expiration as usize))
    }
    
    /// Verify a JWT token
    pub fn verify_jwt(&self, token: &str) -> Result<TokenClaims, Error> {
        // Decode and validate the token
        let token_data = decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|e| Error::Unknown(format!("JWT validation error: {}", e)))?;
        
        Ok(token_data.claims)
    }
    
    /// Create full authentication response
    pub async fn create_auth_response(
        &self,
        code: String,
    ) -> Result<AuthResponse, Error> {
        // Exchange authorization code for token
        let token = self.exchange_code(code).await?;
        
        // Fetch user info
        let user = self.fetch_discord_user(&token.access_token).await?;
        
        // Store user session
        self.store_user_session(user.id.parse::<i64>().unwrap_or_default(), &token).await?;
        
        // Fetch user's guilds from Discord API
        let discord_guilds = self.fetch_discord_guilds(&token.access_token).await
            .unwrap_or_else(|e| {
                // Log the error but continue with empty guilds
                error!("Failed to fetch guilds for user {}: {}", user.id, e);
                vec![]
            });
            
        // Create a vector of guild objects to use in the frontend
        let guilds = discord_guilds.into_iter()
            .map(|g| {
                // Convert permissions from string to u64 (accounting for Option)
                let permissions = g.permissions
                    .as_ref()
                    .and_then(|p| u64::from_str_radix(p, 10).ok())
                    .unwrap_or(0);
                
                // Create Guild object with full details
                crate::web::models::auth::Guild {
                    id: g.id.clone(),
                    name: g.name,
                    icon: g.icon,
                    owner: g.owner.unwrap_or(false),
                    permissions,
                    bot_joined: false, // Will be set later if needed
                }
            })
            .collect::<Vec<_>>();
        
        // Log how many guilds we found
        info!("Found {} guilds for user {}", guilds.len(), user.id);
        
        // Extract guild IDs for JWT - we store only IDs in the JWT to keep it small
        let guild_ids = guilds.iter().map(|g| g.id.clone()).collect::<Vec<String>>();
        
        // Generate JWT
        let (jwt, expires_in) = self.generate_jwt(&user, guild_ids.clone())?;
        
        // Build the response
        let auth_response = AuthResponse {
            token: jwt,
            expires_in,
            user: UserInfo {
                id: user.id.clone(),
                username: user.username.clone(),
                avatar_url: user.avatar.as_ref().map(|avatar| {
                    format!(
                        "https://cdn.discordapp.com/avatars/{}/{}.png",
                        user.id, avatar
                    )
                }),
                guilds: guild_ids, // Use just the guild IDs in the UserInfo
                email: user.email.clone(),
                verified: user.verified,
                locale: user.locale.clone(),
            },
        };
        
        Ok(auth_response)
    }
}
