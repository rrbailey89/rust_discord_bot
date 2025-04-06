// src/web/routes/users.rs
//! User-related API routes

use actix_web::web;
use crate::web::handlers::users;

/// Configure user-related routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("/me", web::get().to(users::get_current_user))
    );
}
