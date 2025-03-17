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

/// JWT Authentication middleware implementation
pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
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
        let auth_header = req.headers().get(header::AUTHORIZATION);
        
        if auth_header.is_none() {
            return Box::pin(futures_util::future::err(
                ErrorUnauthorized("No authorization header provided")
            ));
        }
        
        let auth_value = auth_header.unwrap().to_str();
        
        if auth_value.is_err() {
            return Box::pin(futures_util::future::err(
                ErrorUnauthorized("Invalid authorization header format")
            ));
        }
        
        let auth_string = auth_value.unwrap();
        
        // Check if it's a bearer token
        if !auth_string.starts_with("Bearer ") {
            return Box::pin(futures_util::future::err(
                ErrorUnauthorized("Invalid authorization scheme, expected Bearer")
            ));
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
