// src/web/middleware/auth.rs
//! JWT authentication middleware for Actix Web

use std::future::{ready, Ready};
use std::pin::Pin;
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures_util::Future;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::task::{Context, Poll};
use tracing::error;

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (typically user ID)
    pub sub: String,
    /// Expiration time (as Unix timestamp)
    pub exp: usize,
    /// Issued at (as Unix timestamp)
    pub iat: usize,
    /// Optional user roles
    pub roles: Option<Vec<String>>,
}

/// JWT Authentication middleware
pub struct JwtAuth {
    /// JWT secret key
    pub secret: String,
}

impl JwtAuth {
    /// Create a new JwtAuth middleware
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware {
            service,
            secret: self.secret.clone(),
        }))
    }
}

/// JWT Authentication middleware implementation
pub struct JwtAuthMiddleware<S> {
    service: S,
    secret: String,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
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
        // Check if token is in the Authorization header
        let auth_header = req.headers().get("Authorization");
        
        if auth_header.is_none() {
            return Box::pin(async {
                Ok(req.into_response(
                    HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "No authorization header provided"
                        }))
                        .into_body(),
                ))
            });
        }
        
        let auth_value = auth_header.unwrap().to_str();
        
        if auth_value.is_err() {
            return Box::pin(async {
                Ok(req.into_response(
                    HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "Invalid authorization header format"
                        }))
                        .into_body(),
                ))
            });
        }
        
        let auth_string = auth_value.unwrap();
        
        // Check if it's a bearer token
        if !auth_string.starts_with("Bearer ") {
            return Box::pin(async {
                Ok(req.into_response(
                    HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "Invalid authorization scheme, expected Bearer"
                        }))
                        .into_body(),
                ))
            });
        }
        
        // Extract the token
        let token = &auth_string[7..];
        
        // Decode and validate the token
        let secret = self.secret.clone();
        let validation = Validation::new(Algorithm::HS256);
        
        match decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation) {
            Ok(token_data) => {
                // Store claims in request extensions for handlers to access
                req.extensions_mut().insert(token_data.claims);
                
                let fut = self.service.call(req);
                Box::pin(async move {
                    fut.await
                })
            }
            Err(e) => {
                error!("JWT validation error: {}", e);
                Box::pin(async {
                    Ok(req.into_response(
                        HttpResponse::Unauthorized()
                            .json(serde_json::json!({
                                "error": "Invalid token",
                                "details": e.to_string()
                            }))
                            .into_body(),
                    ))
                })
            }
        }
    }
}

/// Extract JWT claims from request
pub fn extract_claims(req: &ServiceRequest) -> Option<Claims> {
    req.extensions().get::<Claims>().cloned()
}
