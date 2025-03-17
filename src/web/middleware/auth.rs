// src/web/middleware/auth.rs
//! JWT authentication middleware for Actix Web

use std::future::{ready, Ready};
use std::fmt;
use std::pin::Pin;
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header, Error, HttpMessage, HttpResponse,
};
use actix_web::error::{ErrorUnauthorized, ResponseError};
use futures_util::Future;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::task::{Context, Poll};
use std::rc::Rc;
use tracing::error;

use crate::web::models::auth::TokenClaims;

/// Claims type from models (re-export for compatibility)
pub type Claims = TokenClaims;

/// Custom error type for auth middleware
#[derive(Debug)]
struct AuthError(String);

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Authentication error: {}", self.0)
    }
}

impl ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::Unauthorized().json(serde_json::json!({
            "error": self.0,
        }))
    }
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

/// JWT Authentication middleware implementation
pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
    secret: String,
}

impl<S> JwtAuthMiddleware<S> {
    /// Extract token from various sources (header, cookie, etc.)
    fn extract_token_from_request(req: &ServiceRequest) -> Option<String> {
        // First try Authorization header
        if let Some(auth_header) = req.headers().get(header::AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    tracing::debug!("Found token in Authorization header");
                    return Some(auth_str[7..].to_string());
                }
            }
        }
        
        // Then try cookies
        if let Some(cookie) = req.cookie("token") {
            tracing::debug!("Found token in 'token' cookie");
            return Some(cookie.value().to_string());
        }
        
        if let Some(cookie) = req.cookie("auth_token") {
            tracing::debug!("Found token in 'auth_token' cookie");
            return Some(cookie.value().to_string());
        }
        
        // Log that we didn't find a token
        tracing::debug!("No authentication token found in request: {}", req.path());
        
        // No valid token found
        None
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
            service: Rc::new(service),
            secret: self.secret.clone(),
        }))
    }
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
        // Get a potential token from multiple sources
        let token_opt = Self::extract_token_from_request(&req);
        
        // If no token found, return unauthorized
        let token = match token_opt {
            Some(token) => token,
            None => {
                return Box::pin(futures_util::future::err(
                    ErrorUnauthorized("No valid authentication token found")
                ));
            }
        };
        
        // Decode and validate the token
        let secret = self.secret.clone();
        let validation = Validation::new(Algorithm::HS256);
        
        let token_str = token.as_str();
        match decode::<Claims>(token_str, &DecodingKey::from_secret(secret.as_bytes()), &validation) {
            Ok(token_data) => {
                // Store claims in request extensions for handlers to access
                req.extensions_mut().insert(token_data.claims);
                
                let service = Rc::clone(&self.service);
                Box::pin(async move {
                    service.call(req).await
                })
            }
            Err(e) => {
                error!("JWT validation error: {}", e);
                Box::pin(futures_util::future::err(
                    ErrorUnauthorized(format!("Invalid token: {}", e))
                ))
            }
        }
    }
}

/// Extract JWT claims from request
pub fn extract_claims(req: &ServiceRequest) -> Option<Claims> {
    req.extensions().get::<Claims>().cloned()
}
