//! API Routes

mod auth;
mod users;
mod leads;
mod recordings;
mod health;

use axum::{routing::get, Router};

pub fn api_router() -> Router {
    Router::new()
        .route("/health", get(health::health_check))
        .nest("/auth", auth::router())
        .nest("/users", users::router())
        .nest("/leads", leads::router())
        .nest("/recordings", recordings::router())
}
