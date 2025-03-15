// services/rate_limiter.rs
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Semaphore, RwLock, Mutex};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, warn};
use crate::error::{Error, RichError, ErrorContext};

/// Rate limit definition
#[derive(Debug, Clone)]
pub struct RateLimit {
    /// Maximum number of concurrent operations
    pub max_concurrent: usize,
    /// Optional delay between operations
    pub delay: Option<Duration>,
    /// Optional timeout for operations
    pub timeout: Option<Duration>,
}

/// Metric for rate limiter tracking
#[derive(Debug, Clone)]
pub enum RateLimiterMetric {
    /// Number of current active operations
    ActiveOperations,
    /// Number of queued operations
    QueuedOperations,
    /// Number of rejected operations
    RejectedOperations,
    /// Number of timed out operations
    TimedOutOperations,
}

/// Rate limiter service for controlling concurrency and backpressure
#[derive(Debug)]
pub struct RateLimiter {
    /// Semaphores for each named operation
    semaphores: Arc<RwLock<HashMap<String, Arc<Semaphore>>>>,
    /// Active counts for metrics
    active_counts: Arc<RwLock<HashMap<String, usize>>>,
    /// Metrics tracking
    metrics: Arc<Mutex<HashMap<String, u64>>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new() -> Self {
        Self {
            semaphores: Arc::new(RwLock::new(HashMap::new())),
            active_counts: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Execute an operation with rate limiting
    pub async fn with_rate_limit<F, T, E>(&self, operation_name: &str, max_concurrent: usize, operation: F) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: From<Error>,
    {
        // Get or create the semaphore for this operation
        let semaphore = {
            let mut semaphores = self.semaphores.write().await;
            let sem = semaphores
                .entry(operation_name.to_string())
                .or_insert_with(|| Arc::new(Semaphore::new(max_concurrent)))
                .clone();
            sem
        };

        // Try to acquire a permit
        let permit = match semaphore.try_acquire() {
            Ok(permit) => permit,
            Err(_) => {
                self.increment_metric(operation_name, RateLimiterMetric::RejectedOperations).await;
                warn!("Rate limit exceeded for operation: {}", operation_name);
                return Err(Error::RateLimit(format!("Rate limit exceeded for operation: {}", operation_name)).into());
            }
        };

        // Increment active count
        {
            let mut active_counts = self.active_counts.write().await;
            *active_counts.entry(operation_name.to_string()).or_insert(0) += 1;
            self.increment_metric(operation_name, RateLimiterMetric::ActiveOperations).await;
        }

        // Execute the operation
        let result = operation.await;

        // Decrement active count
        {
            let mut active_counts = self.active_counts.write().await;
            if let Some(count) = active_counts.get_mut(operation_name) {
                if *count > 0 {
                    *count -= 1;
                }
            }
        }

        // Drop the permit
        drop(permit);

        result
    }

    /// Execute an operation with rate limiting and timeout
    pub async fn with_rate_limit_timeout<F, T, E>(&self, operation_name: &str, max_concurrent: usize, timeout_duration: Duration, operation: F) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: From<Error>,
    {
        // Get or create the semaphore for this operation
        let semaphore = {
            let mut semaphores = self.semaphores.write().await;
            let sem = semaphores
                .entry(operation_name.to_string())
                .or_insert_with(|| Arc::new(Semaphore::new(max_concurrent)))
                .clone();
            sem
        };

        // Try to acquire a permit
        let permit = match semaphore.try_acquire() {
            Ok(permit) => permit,
            Err(_) => {
                self.increment_metric(operation_name, RateLimiterMetric::RejectedOperations).await;
                warn!("Rate limit exceeded for operation: {}", operation_name);
                return Err(Error::RateLimit(format!("Rate limit exceeded for operation: {}", operation_name)).into());
            }
        };

        // Increment active count
        {
            let mut active_counts = self.active_counts.write().await;
            *active_counts.entry(operation_name.to_string()).or_insert(0) += 1;
            self.increment_metric(operation_name, RateLimiterMetric::ActiveOperations).await;
        }

        // Execute the operation with timeout
        let result = match timeout(timeout_duration, operation).await {
            Ok(res) => res,
            Err(_) => {
                self.increment_metric(operation_name, RateLimiterMetric::TimedOutOperations).await;
                warn!("Operation timed out: {}", operation_name);
                Err(Error::Timeout(format!("Operation timed out: {}", operation_name)).into())
            }
        };

        // Decrement active count
        {
            let mut active_counts = self.active_counts.write().await;
            if let Some(count) = active_counts.get_mut(operation_name) {
                if *count > 0 {
                    *count -= 1;
                }
            }
        }

        // Drop the permit
        drop(permit);

        result
    }

    /// Get the current number of active operations
    pub async fn get_active_count(&self, operation_name: &str) -> usize {
        let active_counts = self.active_counts.read().await;
        *active_counts.get(operation_name).unwrap_or(&0)
    }

    /// Get available permits for an operation
    pub async fn get_available_permits(&self, operation_name: &str) -> Option<usize> {
        let semaphores = self.semaphores.read().await;
        semaphores.get(operation_name).map(|s| s.available_permits())
    }

    /// Increment a metric
    async fn increment_metric(&self, operation_name: &str, metric_type: RateLimiterMetric) {
        let metric_name = match metric_type {
            RateLimiterMetric::ActiveOperations => format!("{}_active", operation_name),
            RateLimiterMetric::QueuedOperations => format!("{}_queued", operation_name),
            RateLimiterMetric::RejectedOperations => format!("{}_rejected", operation_name),
            RateLimiterMetric::TimedOutOperations => format!("{}_timeout", operation_name),
        };

        let mut metrics = self.metrics.lock().await;
        *metrics.entry(metric_name).or_insert(0) += 1;
    }

    /// Get metrics
    pub async fn get_metrics(&self) -> HashMap<String, u64> {
        let metrics = self.metrics.lock().await;
        metrics.clone()
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}
