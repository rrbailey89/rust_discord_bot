// error.rs - Central error handling module
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use poise::serenity_prelude;
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;
use lazy_static::lazy_static;

// Main Error enum
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
                    .add_info("input", input.unwrap_or_default().to_string());
                
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

// Error metrics collection
pub struct ErrorMetrics {
    error_counts: DashMap<String, AtomicUsize>,
    last_errors: DashMap<String, (DateTime<Utc>, String)>,
}

impl ErrorMetrics {
    pub fn new() -> Self {
        Self {
            error_counts: DashMap::new(),
            last_errors: DashMap::new(),
        }
    }
    
    pub fn record_error(&self, error_type: &str, error_message: &str) {
        self.error_counts
            .entry(error_type.to_string())
            .or_insert_with(|| AtomicUsize::new(0))
            .fetch_add(1, Ordering::SeqCst);
        
        self.last_errors.insert(
            error_type.to_string(),
            (Utc::now(), error_message.to_string())
        );
    }
    
    pub fn get_error_count(&self, error_type: &str) -> usize {
        self.error_counts
            .get(error_type)
            .map(|count| count.load(Ordering::SeqCst))
            .unwrap_or(0)
    }
    
    pub fn get_error_rates(&self) -> HashMap<String, (usize, Option<DateTime<Utc>>)> {
        let mut result = HashMap::new();
        
        for entry in self.error_counts.iter() {
            let error_type = entry.key();
            let count = entry.value().load(Ordering::SeqCst);
            
            let last_time = self.last_errors
                .get(error_type)
                .map(|entry| entry.0);
            
            result.insert(error_type.clone(), (count, last_time));
        }
        
        result
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

// Generic retry mechanism
pub async fn with_retry<F, Fut, T, E>(
    operation: F,
    max_retries: usize,
    initial_delay: Duration,
) -> Result<T, Error>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: Into<Error> + std::fmt::Display,
{
    let mut retries = 0;
    let mut delay = initial_delay;
    
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                if retries >= max_retries {
                    return Err(err.into());
                }
                
                tracing::warn!("Operation failed, retrying ({}/{}): {}", retries + 1, max_retries, err);
                sleep(delay).await;
                
                retries += 1;
                delay *= 2; // Exponential backoff
            }
        }
    }
}

// Circuit breaker pattern
pub mod circuit_breaker {
    use std::sync::atomic::{AtomicU8, AtomicUsize, AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH, Duration};
    use std::future::Future;
    use tokio::sync::RwLock;
    use std::sync::Arc;
    use once_cell::sync::Lazy;
    use std::collections::HashMap;
    use super::{Error, RichError, ErrorContext};

    // Circuit breaker states
    const CLOSED: u8 = 0;     // Normal operation
    const OPEN: u8 = 1;       // Preventing calls
    const HALF_OPEN: u8 = 2;  // Testing if service is back

    pub struct CircuitBreaker {
        /// Current state of the circuit breaker
        state: AtomicU8,
        
        /// Count of consecutive failures
        failure_count: AtomicUsize,
        
        /// Timestamp of last failure (seconds since UNIX epoch)
        last_failure: AtomicU64,
        
        /// Number of consecutive failures required to open the circuit
        failure_threshold: usize,
        
        /// Duration to wait before attempting to close the circuit
        reset_timeout: Duration,
        
        /// Name for this circuit breaker (for logging)
        name: &'static str,
    }

    impl CircuitBreaker {
        pub fn new(name: &'static str, failure_threshold: usize, reset_timeout: Duration) -> Self {
            Self {
                state: AtomicU8::new(CLOSED),
                failure_count: AtomicUsize::new(0),
                last_failure: AtomicU64::new(0),
                failure_threshold,
                reset_timeout,
                name,
            }
        }
        
        /// Get the current state of the circuit breaker
        pub fn state(&self) -> u8 {
            self.state.load(Ordering::SeqCst)
        }
        
        /// Check if the circuit is open (preventing calls)
        pub fn is_open(&self) -> bool {
            self.state() == OPEN
        }
        
        /// Reset the circuit breaker to closed state
        pub fn reset(&self) {
            self.state.store(CLOSED, Ordering::SeqCst);
            self.failure_count.store(0, Ordering::SeqCst);
            tracing::info!("Circuit breaker '{}' manually reset to CLOSED state", self.name);
        }
        
        /// Record a successful call
        pub fn record_success(&self) {
            let current_state = self.state();
            
            if current_state == HALF_OPEN {
                // If we were in half-open state and got a success, close the circuit
                self.state.store(CLOSED, Ordering::SeqCst);
                self.failure_count.store(0, Ordering::SeqCst);
                tracing::info!("Circuit breaker '{}' closed after successful test call", self.name);
            } else if current_state == CLOSED {
                // In closed state, reset failure count on success
                self.failure_count.store(0, Ordering::SeqCst);
            }
        }
        
        /// Record a failed call
        pub fn record_failure(&self) {
            let current_state = self.state();
            
            if current_state == HALF_OPEN {
                // If we failed while testing, go back to open state
                self.state.store(OPEN, Ordering::SeqCst);
                self.failure_count.store(self.failure_threshold, Ordering::SeqCst);
                
                // Update last failure time
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                self.last_failure.store(now, Ordering::SeqCst);
                
                tracing::warn!("Circuit breaker '{}' reopened after failed test call", self.name);
            } else if current_state == CLOSED {
                // Increment failure count
                let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                
                // Update last failure time
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                self.last_failure.store(now, Ordering::SeqCst);
                
                // Check if we need to open the circuit
                if failures >= self.failure_threshold {
                    self.state.store(OPEN, Ordering::SeqCst);
                    tracing::warn!("Circuit breaker '{}' opened after {} consecutive failures", 
                                  self.name, failures);
                }
            }
        }
        
        /// Execute an operation with circuit breaker protection
        pub async fn call<F, Fut, T, E>(&self, operation: F) -> Result<T, Error>
        where
            F: Fn() -> Fut,
            Fut: Future<Output = Result<T, E>>,
            E: Into<Error>,
        {
            // Check if circuit is open
            let current_state = self.state();
            
            if current_state == OPEN {
                // See if enough time has passed since last failure
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let last_failure = self.last_failure.load(Ordering::SeqCst);
                
                if (now - last_failure) as u64 >= self.reset_timeout.as_secs() {
                    // Transition to half-open state to test the service
                    self.state.store(HALF_OPEN, Ordering::SeqCst);
                    tracing::info!("Circuit breaker '{}' transitioning to HALF-OPEN state for testing", self.name);
                } else {
                    // Circuit is still open
                    return Err(Error::ExternalService { 
                        service: self.name.to_string(), 
                        message: format!("Circuit breaker '{}' is open", self.name)
                    });
                }
            }
            
            // Execute the operation
            match operation().await {
                Ok(result) => {
                    // Record the success
                    self.record_success();
                    Ok(result)
                },
                Err(err) => {
                    // Record the failure
                    self.record_failure();
                    Err(err.into())
                }
            }
        }
        
        /// Execute an operation with circuit breaker protection and return RichError
        pub async fn call_with_context<F, Fut, T, E>(&self, operation: F, context_fn: impl Fn() -> ErrorContext) -> Result<T, RichError>
        where
            F: Fn() -> Fut,
            Fut: Future<Output = Result<T, E>>,
            E: Into<Error>,
        {
            match self.call(operation).await {
                Ok(result) => Ok(result),
                Err(error) => {
                    Err(RichError {
                        error,
                        context: context_fn(),
                    })
                }
            }
        }
    }

    // Global registry for circuit breakers
    pub struct CircuitBreakerRegistry {
        breakers: RwLock<HashMap<&'static str, Arc<CircuitBreaker>>>,
    }

    impl CircuitBreakerRegistry {
        pub fn new() -> Self {
            Self {
                breakers: RwLock::new(HashMap::new()),
            }
        }
        
        /// Register a new circuit breaker or return existing one
        pub async fn register(
            &self, 
            name: &'static str, 
            failure_threshold: usize, 
            reset_timeout: Duration
        ) -> Arc<CircuitBreaker> {
            // First try to read without locking for writes
            {
                let breakers = self.breakers.read().await;
                if let Some(breaker) = breakers.get(name) {
                    return breaker.clone();
                }
            }
            
            // If not found, lock for writing and try again (double-checked locking)
            let mut breakers = self.breakers.write().await;
            if let Some(breaker) = breakers.get(name) {
                return breaker.clone();
            }
            
            // Create new circuit breaker
            let breaker = Arc::new(CircuitBreaker::new(name, failure_threshold, reset_timeout));
            breakers.insert(name, breaker.clone());
            
            tracing::info!("Registered new circuit breaker: {}", name);
            breaker
        }
        
        /// Get a circuit breaker by name
        pub async fn get(&self, name: &'static str) -> Option<Arc<CircuitBreaker>> {
            self.breakers.read().await.get(name).cloned()
        }
        
        /// Get all registered circuit breakers
        pub async fn get_all(&self) -> Vec<(&'static str, Arc<CircuitBreaker>)> {
            self.breakers.read().await
                .iter()
                .map(|(&name, breaker)| (name, breaker.clone()))
                .collect()
        }
        
        /// Reset all circuit breakers
        pub async fn reset_all(&self) {
            for (name, breaker) in self.breakers.read().await.iter() {
                breaker.reset();
                tracing::info!("Reset circuit breaker: {}", name);
            }
        }
    }

    // Global registry instance
    pub static REGISTRY: Lazy<CircuitBreakerRegistry> = Lazy::new(CircuitBreakerRegistry::new);

    // Helper function to use a circuit breaker or create it if it doesn't exist
    pub async fn with_circuit_breaker<F, Fut, T, E>(
        name: &'static str, 
        failure_threshold: usize, 
        reset_timeout: Duration,
        operation: F,
    ) -> Result<T, Error>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: Into<Error>,
    {
        let breaker = REGISTRY.register(name, failure_threshold, reset_timeout).await;
        breaker.call(operation).await
    }

    // Helper function with rich error context
    pub async fn with_circuit_breaker_context<F, Fut, T, E>(
        name: &'static str, 
        failure_threshold: usize, 
        reset_timeout: Duration,
        operation: F,
        context_fn: impl Fn() -> ErrorContext,
    ) -> Result<T, RichError>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: Into<Error>,
    {
        let breaker = REGISTRY.register(name, failure_threshold, reset_timeout).await;
        breaker.call_with_context(operation, context_fn).await
    }
}
