// src/web/middleware/rate_limit.rs
//! Rate limiting middleware for Actix Web

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorTooManyRequests,
    http::header::{HeaderName, HeaderValue},
    Error,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use std::task::{Context, Poll};
use std::rc::Rc;

/// Rate limit configuration
#[derive(Clone)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed within the window
    pub requests: u32,
    /// Time window in seconds
    pub window: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests: 100,
            window: 60,
        }
    }
}

/// Rate limiter middleware
#[derive(Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
}

impl RateLimiter {
    /// Create a new RateLimiter with the given configuration
    pub fn new(config: RateLimitConfig) -> Self {
        Self { config }
    }
    
    /// Create a new RateLimiter with default configuration
    pub fn default() -> Self {
        Self {
            config: RateLimitConfig::default(),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static + Clone,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RateLimiterMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimiterMiddleware {
            service,
            config: self.config.clone(),
            clients: Rc::new(Mutex::new(HashMap::new())),
        })
    }
}

/// Middleware for rate limiting requests
pub struct RateLimiterMiddleware<S> {
    service: S,
    config: RateLimitConfig,
    clients: Rc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl<S, B> Service<ServiceRequest> for RateLimiterMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static + Clone,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Get client IP address
        let ip = match req.connection_info().realip_remote_addr() {
            Some(ip) => ip.to_string(),
            None => "unknown".to_string(),
        };
        
        // Track request time
        let now = Instant::now();
        let window = Duration::from_secs(self.config.window);
        
        // Update rate limit tracking
        let mut too_many_requests = false;
        let (requests_in_window, reset_time) = {
            let mut clients = self.clients.lock().unwrap();
            
            // Get or create client entry
            let client_requests = clients.entry(ip.clone()).or_insert_with(Vec::new);
            
            // Remove old requests outside the window
            client_requests.retain(|&t| now.duration_since(t) < window);
            
            // Check if client has exceeded rate limit
            if client_requests.len() >= self.config.requests as usize {
                too_many_requests = true;
            } else {
                // Add current request
                client_requests.push(now);
            }
            
            // Calculate remaining time in window
            let reset_time = if client_requests.is_empty() {
                self.config.window
            } else {
                let oldest = client_requests[0];
                let elapsed = now.duration_since(oldest).as_secs();
                if elapsed >= self.config.window {
                    0
                } else {
                    self.config.window - elapsed
                }
            };
            
            (client_requests.len(), reset_time)
        };
        
        // Set rate limit headers
        let mut rate_limit_headers = vec![
            (
                HeaderName::from_static("x-ratelimit-limit"),
                HeaderValue::from_str(&self.config.requests.to_string()).unwrap(),
            ),
            (
                HeaderName::from_static("x-ratelimit-remaining"),
                HeaderValue::from_str(&(self.config.requests as usize - requests_in_window).to_string()).unwrap(),
            ),
            (
                HeaderName::from_static("x-ratelimit-reset"),
                HeaderValue::from_str(&reset_time.to_string()).unwrap(),
            ),
        ];
        
        // Check if rate limit exceeded
        if too_many_requests {
            // Return an error directly instead of trying to convert to a ServiceResponse
            return Box::pin(futures_util::future::err(
                ErrorTooManyRequests("Rate limit exceeded")
            ));
        }
        
        // Process the request
        let service = self.service.clone();
        let fut = service.call(req);
        
        Box::pin(async move {
            let mut res = fut.await?;
            
            // Add rate limit headers to response
            for (name, value) in rate_limit_headers {
                res.headers_mut().insert(name, value);
            }
            
            Ok(res)
        })
    }
}
