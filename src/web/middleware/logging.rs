// src/web/middleware/logging.rs
//! Request logging middleware for Actix Web

use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::{ok, Ready};
use futures_util::Future;
use tracing::{debug, info};

/// Request logger middleware
#[derive(Default, Clone)]
pub struct RequestLogger;

impl<S, B> Transform<S, ServiceRequest> for RequestLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestLoggerMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RequestLoggerMiddleware { service })
    }
}

/// Middleware for logging requests
pub struct RequestLoggerMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestLoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<ServiceResponse<B>, Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start_time = Instant::now();
        let method = req.method().to_string();
        let path = req.path().to_string();
        
        // Get client IP address
        let client_ip = req
            .connection_info()
            .realip_remote_addr()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        
        // Log request
        debug!(
            method = %method,
            path = %path,
            client_ip = %client_ip,
            "Request received"
        );
        
        let fut = self.service.call(req);
        
        Box::pin(async move {
            let res = fut.await?;
            let elapsed = start_time.elapsed();
            
            // Log response
            info!(
                method = %method,
                path = %path,
                status = %res.status().as_u16(),
                duration_ms = %elapsed.as_millis(),
                client_ip = %client_ip,
                "Request completed"
            );
            
            Ok(res)
        })
    }
}
