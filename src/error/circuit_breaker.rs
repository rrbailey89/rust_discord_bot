use std::sync::atomic::{AtomicU8, AtomicUsize, AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use crate::error::{Error, RichError};
use std::future::Future;
use tokio::sync::RwLock;
use std::sync::Arc;
use once_cell::sync::Lazy;
use std::collections::HashMap;

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
    pub async fn call_with_context<F, Fut, T, E>(&self, operation: F, context_fn: impl Fn() -> crate::error::ErrorContext) -> Result<T, RichError>
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
    context_fn: impl Fn() -> crate::error::ErrorContext,
) -> Result<T, RichError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: Into<Error>,
{
    let breaker = REGISTRY.register(name, failure_threshold, reset_timeout).await;
    breaker.call_with_context(operation, context_fn).await
}
