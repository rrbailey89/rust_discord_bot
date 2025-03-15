use chrono::{DateTime, Utc};
use poise::serenity_prelude;
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;
use lazy_static::lazy_static;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Serenity error: {0}")]
    Serenity(#[from] serenity_prelude::Error),

    #[error("Database error: {0}")]
    Database(#[from] tokio_postgres::Error),
    
    #[error("Database pool error: {0}")]
    DbPool(#[from] deadpool_postgres::PoolError),
    
    #[error("Database configuration error: {0}")]
    DbConfig(#[from] deadpool_postgres::ConfigError),
    
    #[error("Cache error: {0}")]
    Cache(String),
    
    #[error("Cache key not found")]
    CacheKeyNotFound,
    
    #[error("Cache serialization error: {0}")]
    CacheSerialization(String),

    #[error("Configuration error: {0}")]
    Config(#[from] toml::de::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("OpenAI error: {0}")]
    OpenAI(#[from] async_openai::error::OpenAIError),

    #[error("Chrono parse error: {0}")]
    ChronoParse(#[from] chrono::ParseError),

    #[error("Timezone parse error: {0}")]
    TimezoneParse(#[from] chrono_tz::ParseError),

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    
    #[error("API rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Permission error: {0}")]
    Permission(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Timeout error: {0}")]
    Timeout(String),
    
    #[error("Operation cancelled: {0}")]
    Cancelled(String),
    
    #[error("Backpressure limit reached: {0}")]
    Backpressure(String),

    #[error("External service error: {service} - {message}")]
    ExternalService { service: String, message: String },

    #[error("Command error: {0}")]
    Command(String),

    #[error("Bot is not a member of the specified guild")]
    NotInGuild,

    #[error("Channel not found in the specified guild")]
    ChannelNotFound,

    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

// A struct to provide additional context for errors
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub command: Option<String>,
    pub guild_id: Option<i64>,
    pub user_id: Option<i64>,
    pub timestamp: DateTime<Utc>,
    pub additional_info: HashMap<String, String>,
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self {
            command: None,
            guild_id: None,
            user_id: None,
            timestamp: Utc::now(),
            additional_info: HashMap::new(),
        }
    }
}

impl ErrorContext {
    pub fn new() -> Self {
        Default::default()
    }
    
    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }
    
    pub fn with_guild(mut self, guild_id: i64) -> Self {
        self.guild_id = Some(guild_id);
        self
    }
    
    pub fn with_user(mut self, user_id: i64) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    pub fn add_info(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.additional_info.insert(key.into(), value.into());
        self
    }
}

// A rich error type that combines the Error enum with context
#[derive(Debug)]
pub struct RichError {
    pub error: Error,
    pub context: ErrorContext,
}

impl fmt::Display for RichError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)?;
        
        if let Some(cmd) = &self.context.command {
            write!(f, " [Command: {}]", cmd)?;
        }
        
        if let Some(guild) = self.context.guild_id {
            write!(f, " [Guild: {}]", guild)?;
        }
        
        if let Some(user) = self.context.user_id {
            write!(f, " [User: {}]", user)?;
        }
        
        Ok(())
    }
}

impl std::error::Error for RichError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

impl From<Error> for RichError {
    fn from(error: Error) -> Self {
        Self {
            error,
            context: ErrorContext::default(),
        }
    }
}

impl From<RichError> for Error {
    fn from(rich_error: RichError) -> Self {
        rich_error.error
    }
}

// Add user-friendly error messages
lazy_static! {
    static ref USER_FRIENDLY_MESSAGES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("Database error", "I'm having trouble accessing my database. Please try again later.");
        m.insert("Database pool error", "I'm having trouble connecting to my database. Please try again later.");
        m.insert("OpenAI error", "I'm having trouble generating a response. Please try again later.");
        m.insert("Request error", "I couldn't complete your request. Please try again later.");
        m.insert("Rate limit", "I've reached my request limit. Please try again in a few minutes.");
        m.insert("Network error", "I'm having trouble connecting to external services. Please try again later.");
        m.insert("Authentication error", "I'm having trouble authenticating with an external service. Please contact the bot administrator.");
        m.insert("Permission error", "I don't have permission to do that. Please make sure I have the correct permissions and try again.");
        m.insert("Not found", "I couldn't find what you're looking for. Please check your input and try again.");
        m.insert("Validation error", "Your input doesn't seem to be valid. Please check and try again.");
        m.insert("Timeout error", "The operation timed out. Please try again later.");
        m.insert("External service error", "An external service I rely on is having issues. Please try again later.");
        m.insert("Command error", "There was an error executing your command. Please check your input and try again.");
        m.insert("Unknown error", "Something went wrong. Please try again later.");
        m
    };
}

impl Error {
    // Get a user-friendly error message
    pub fn user_friendly_message(&self) -> String {
        let error_type = match self {
            Error::Database(_) => "Database error",
            Error::DbPool(_) => "Database pool error",
            Error::OpenAI(_) => "OpenAI error",
            Error::Request(_) => "Request error",
            Error::RateLimit(_) => "Rate limit",
            Error::Network(_) => "Network error",
            Error::Authentication(_) => "Authentication error",
            Error::Permission(_) => "Permission error",
            Error::NotFound(_) => "Not found",
            Error::Validation(_) => "Validation error",
            Error::Timeout(_) => "Timeout error",
            Error::ExternalService { .. } => "External service error",
            Error::Command(_) => "Command error",
            Error::Unknown(_) => "Unknown error",
            _ => "Unknown error",
        };
        
        USER_FRIENDLY_MESSAGES.get(error_type)
            .map(|msg| msg.to_string())
            .unwrap_or_else(|| "Something went wrong. Please try again later.".to_string())
    }
}

impl RichError {
    pub fn user_friendly_message(&self) -> String {
        self.error.user_friendly_message()
    }
    
    // Create a RichError from a poise framework error
    #[allow(unused)]
    pub fn from_poise_error(error: poise::FrameworkError<'_, crate::Data, Error>) -> Self {
        match error {
            poise::FrameworkError::Command { error, ctx, .. } => {
                let mut context = ErrorContext::default()
                    .with_command(ctx.command().qualified_name.clone());
                
                if let Some(guild_id) = ctx.guild_id() {
                    context = context.with_guild(guild_id.get() as i64);
                }
                
                RichError {
                    error,
                    context,
                }
            },
            poise::FrameworkError::ArgumentParse { error, input, ctx, .. } => {
                let mut context = ErrorContext::default()
                    .with_command(ctx.command().qualified_name.clone())
                    .add_info("input", input.to_string());
                
                if let Some(guild_id) = ctx.guild_id() {
                    context = context.with_guild(guild_id.get() as i64);
                }
                
                RichError {
                    error: Error::Command(format!("Failed to parse argument: {}", error)),
                    context,
                }
            },
            _ => {
                let error_message = format!("Unhandled framework error");
                RichError {
                    error: Error::Unknown(error_message),
                    context: ErrorContext::default(),
                }
            }
        }
    }
}

// Helper function for logging errors
pub fn log_error(error: &RichError) {
    let context = &error.context;
    
    tracing::error!(
        error = %error.error,
        command = ?context.command,
        guild_id = ?context.guild_id,
        user_id = ?context.user_id,
        timestamp = %context.timestamp,
        "Error occurred"
    );
    
    // Log additional context if available
    for (key, value) in &context.additional_info {
        tracing::debug!("{}: {}", key, value);
    }
}
