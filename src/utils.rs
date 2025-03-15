use crate::error::{Error, ErrorContext, RichError};
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{oneshot, Mutex, RwLock};
use tokio::time::timeout;
use tracing::{debug, warn};

pub fn parse_datetime(month: &str, day: i64, year: i64, time: &str, timezone: &str) -> Result<DateTime<Utc>, Error> {
    let month_num = match month.to_lowercase().as_str() {
        "january" => 1, "february" => 2, "march" => 3, "april" => 4,
        "may" => 5, "june" => 6, "july" => 7, "august" => 8,
        "september" => 9, "october" => 10, "november" => 11, "december" => 12,
        _ => return Err(Error::Unknown("Invalid month".to_string())),
    };

    let (hour, minute) = parse_time(time)?;

    let naive_date = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(year as i32, month_num, day as u32).ok_or_else(|| Error::Unknown("Invalid date".to_string()))?,
        NaiveTime::from_hms_opt(hour, minute, 0).ok_or_else(|| Error::Unknown("Invalid time".to_string()))?,
    );

    let tz: Tz = timezone.parse()?;
    Ok(tz.from_local_datetime(&naive_date)
        .single() // This replaces `unwrap()` and handles ambiguous times
        .ok_or_else(|| Error::Unknown("Ambiguous or non-existent local time".to_string()))?
        .with_timezone(&Utc))
}

fn parse_time(time: &str) -> Result<(u32, u32), Error> {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 {
        return Err(Error::Unknown("Invalid time format".to_string()));
    }

    let hour: u32 = parts[0].parse().map_err(|_| Error::Unknown("Invalid hour".to_string()))?;
    let minute: u32 = parts[1].split_whitespace().next().unwrap().parse().map_err(|_| Error::Unknown("Invalid minute".to_string()))?;
    let period = time.to_lowercase();

    let hour = if period.contains("pm") && hour != 12 {
        hour + 12
    } else if period.contains("am") && hour == 12 {
        0
    } else {
        hour
    };

    Ok((hour, minute))
}

/// Async context for tracking and logging async operations
#[derive(Debug, Clone)]
pub struct AsyncOpContext {
    /// Unique name or identifier for the operation
    pub name: String,
    /// When the operation started
    pub started_at: Instant,
    /// Maximum allowed duration for the operation
    pub timeout: Option<Duration>,
    /// Number of retries attempted
    pub retries: usize,
}

impl AsyncOpContext {
    /// Create a new async operation context
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            started_at: Instant::now(),
            timeout: None,
            retries: 0,
        }
    }
    
    /// Set the timeout for this operation
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    /// Record a retry of this operation
    pub fn record_retry(&mut self) {
        self.retries += 1;
    }
    
    /// Get the elapsed time since this operation started
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }
    
    /// Create error context with operation details
    pub fn to_error_context(&self) -> ErrorContext {
        let mut ctx = ErrorContext::new()
            .add_info("async_op", &self.name)
            .add_info("elapsed_ms", &self.elapsed().as_millis().to_string());
        
        if let Some(timeout) = self.timeout {
            ctx = ctx.add_info("timeout_ms", &timeout.as_millis().to_string());
        }
        
        if self.retries > 0 {
            ctx = ctx.add_info("retries", &self.retries.to_string());
        }
        
        ctx
    }
}

/// A struct that encapsulates cancellation capability
/// Simplier version of tokio::sync::CancellationToken, more suitable
/// for our use case
#[derive(Debug, Clone)]
pub struct CancellationToken {
    /// Shared state for cancellation
    state: Arc<RwLock<CancellationState>>,
}

/// Internal state for the cancellation token
#[derive(Debug)]
struct CancellationState {
    /// Has this token been cancelled?
    cancelled: bool,
    /// Receiver that will be notified when cancel is called
    receiver: Option<oneshot::Receiver<()>>,
}

impl CancellationToken {
    /// Check if this token has been cancelled
    pub fn is_cancelled(&self) -> bool {
        // Use a non-async method to check cancellation
        // This only requires a read lock, so it's fast
        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let state = self.state.read().await;
                state.cancelled
            })
        })
    }
    
    /// Asynchronously wait for cancellation
    pub async fn cancelled(&self) -> () {
        // First do a quick check if we're already cancelled
        {
            let state = self.state.read().await;
            if state.cancelled {
                return;
            }
        }
        
        // Not cancelled yet, take the receiver if it exists
        let receiver_opt = {
            let mut state = self.state.write().await;
            state.receiver.take()
        };
        
        if let Some(receiver) = receiver_opt {
            // Wait for the cancel signal
            let _ = receiver.await;
        }
    }
    
    /// Create a new cancellation token
    fn new(receiver: oneshot::Receiver<()>) -> Self {
        Self {
            state: Arc::new(RwLock::new(CancellationState {
                cancelled: false,
                receiver: Some(receiver),
            })),
        }
    }
    
    /// Mark this token as cancelled
    fn set_cancelled(&self) {
        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut state = self.state.write().await;
                state.cancelled = true;
            })
        });
    }
}

/// Source for a cancellation token, used to trigger cancellation
#[derive(Debug)]
pub struct CancellationSource {
    token: CancellationToken,
    sender: Option<oneshot::Sender<()>>,
}

impl CancellationSource {
    /// Create a new cancellation source and token
    pub fn new() -> Self {
        let (sender, receiver) = oneshot::channel();
        Self {
            token: CancellationToken::new(receiver),
            sender: Some(sender),
        }
    }
    
    /// Get the token that can be passed to async operations
    pub fn token(&self) -> CancellationToken {
        self.token.clone()
    }
    
    /// Cancel all operations using this token
    pub fn cancel(mut self) {
        // Mark the token as cancelled
        self.token.set_cancelled();
        
        // Send the signal if the sender is still available
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(());
        }
    }
}

impl Default for CancellationSource {
    fn default() -> Self {
        Self::new()
    }
}

/// Execute an operation with a timeout
pub async fn with_timeout<F, T, E>(
    duration: Duration,
    operation_name: &str,
    future: F,
) -> Result<T, Error>
where
    F: Future<Output = Result<T, E>>,
    E: Into<Error>,
{
    match timeout(duration, future).await {
        Ok(result) => result.map_err(Into::into),
        Err(_) => {
            warn!("Operation {} timed out after {:?}", operation_name, duration);
            Err(Error::Timeout(format!("Operation {} timed out after {:?}", operation_name, duration)))
        }
    }
}

/// Execute an operation with a timeout and context
pub async fn with_timeout_context<F, T, E>(
    duration: Duration,
    context: &AsyncOpContext,
    future: F,
) -> Result<T, RichError>
where
    F: Future<Output = Result<T, E>>,
    E: Into<Error>,
{
    match timeout(duration, future).await {
        Ok(result) => result.map_err(|e| {
            let error = e.into();
            RichError {
                error,
                context: context.to_error_context(),
            }
        }),
        Err(_) => {
            warn!(
                "Operation {} timed out after {:?} (elapsed: {:?})",
                context.name, duration, context.elapsed()
            );
            Err(RichError {
                error: Error::Timeout(format!(
                    "Operation {} timed out after {:?}",
                    context.name, duration
                )),
                context: context.to_error_context(),
            })
        }
    }
}

/// Execute an operation that can be cancelled
pub async fn cancellable<F, T, E>(
    future: F,
    token: CancellationToken,
    operation_name: &str,
) -> Result<T, Error>
where
    F: Future<Output = Result<T, E>>,
    E: Into<Error>,
{
    tokio::select! {
        result = future => result.map_err(Into::into),
        _ = token.cancelled() => {
            warn!("Operation {} was cancelled by cancellation token", operation_name);
            Err(Error::Cancelled(format!("Operation {} was cancelled", operation_name)))
        }
    }
}

/// Execute an operation that can be cancelled with context
pub async fn cancellable_with_context<F, T, E>(
    future: F,
    token: CancellationToken,
    context: &AsyncOpContext,
) -> Result<T, RichError>
where
    F: Future<Output = Result<T, E>>,
    E: Into<Error>,
{
    tokio::select! {
        result = future => result.map_err(|e| {
            let error = e.into();
            RichError {
                error,
                context: context.to_error_context(),
            }
        }),
        _ = token.cancelled() => {
            warn!("Operation {} was cancelled by cancellation token", context.name);
            Err(RichError {
                error: Error::Cancelled(format!("Operation {} was cancelled", context.name)),
                context: context.to_error_context(),
            })
        }
    }
}

/// Retry a fallible operation with configurable backoff
pub async fn with_retry<F, Fut, T, E>(
    operation: F,
    max_attempts: usize,
    base_delay: Duration,
    max_delay: Duration,
    jitter: bool,
    context: &mut AsyncOpContext,
) -> Result<T, Error>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: Into<Error> + std::fmt::Debug,
{
    debug!("Starting operation {} with retry (max attempts: {})", context.name, max_attempts);
    
    let mut attempt = 0;
    let mut delay = base_delay;
    
    loop {
        attempt += 1;
        debug!("Attempt {}/{} for operation {}", attempt, max_attempts, context.name);
        
        match operation().await {
            Ok(value) => {
                if attempt > 1 {
                    debug!("Operation {} succeeded after {} retries", context.name, attempt - 1);
                }
                return Ok(value);
            }
            Err(err) => {
                let error = err.into();
                
                // Don't retry certain errors or if we've reached max attempts
                if !is_retryable_error(&error) || attempt >= max_attempts {
                    if attempt > 1 {
                        warn!("Operation {} failed after {} retries: {:?}", context.name, attempt - 1, error);
                    }
                    return Err(error);
                }
                
                warn!(
                    "Attempt {}/{} for operation {} failed, retrying in {:?}: {:?}",
                    attempt, max_attempts, context.name, delay, error
                );
                
                context.record_retry();
                
                tokio::time::sleep(delay).await;
                
                // Calculate next delay with exponential backoff
                delay = std::cmp::min(
                    delay.mul_f64(1.5),
                    max_delay
                );
                
                // Add jitter if enabled
                if jitter {
                    use rand::Rng;
                    let jitter_factor = rand::thread_rng().gen_range(0.8..1.2);
                    delay = delay.mul_f64(jitter_factor);
                }
            }
        }
    }
}

/// Determine if an error is retryable
fn is_retryable_error(error: &Error) -> bool {
    match error {
        // Typically retryable errors
        Error::Database(_) => true,
        Error::DbPool(_) => true,
        Error::Network(_) => true,
        Error::Timeout(_) => true,
        Error::Request(e) if e.is_timeout() || e.is_connect() => true,
        Error::ExternalService { .. } => true,
        
        // Typically non-retryable errors
        Error::NotFound(_) => false,
        Error::Validation(_) => false,
        Error::Permission(_) => false,
        Error::Authentication(_) => false,
        Error::Command(_) => false,
        
        // Default to retrying for other error types
        _ => true,
    }
}
