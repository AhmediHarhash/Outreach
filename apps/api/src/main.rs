//! Hekax API - The Brain
//!
//! Production-grade backend for the Hekax ecosystem.
//! Handles auth, leads, recordings, RAG, and sync.

mod config;
mod db;
mod error;
mod auth;
mod routes;
mod models;

use axum::{Router, Extension};
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::db::Database;

/// Application state shared across handlers
pub struct AppState {
    pub db: Database,
    pub config: Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "hekax_api=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Hekax API v{}", env!("CARGO_PKG_VERSION"));

    // Load config
    let config = Config::from_env()?;
    tracing::info!("Environment: {}", config.environment);

    // Connect to database
    let db = Database::connect(&config.database_url).await?;
    tracing::info!("Connected to PostgreSQL");

    // Run migrations
    db.migrate().await?;
    tracing::info!("Migrations complete");

    // Build app state
    let state = Arc::new(AppState { db, config: config.clone() });

    // Build router
    let app = Router::new()
        .nest("/api/v1", routes::api_router())
        .layer(Extension(state))
        .layer(TraceLayer::new_for_http())
        .layer(cors_layer(&config));

    // Start server
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

fn cors_layer(config: &Config) -> CorsLayer {
    use axum::http::{HeaderName, Method};
    use tower_http::cors::Any;

    if config.environment == "development" {
        CorsLayer::very_permissive()
    } else {
        CorsLayer::new()
            .allow_origin(config.allowed_origins.iter().map(|s| s.parse().unwrap()).collect::<Vec<_>>())
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
            .allow_headers([
                HeaderName::from_static("content-type"),
                HeaderName::from_static("authorization"),
            ])
            .allow_credentials(true)
    }
}
