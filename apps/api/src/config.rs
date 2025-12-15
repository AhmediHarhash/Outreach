//! Application configuration

use anyhow::{Context, Result};

#[derive(Clone)]
pub struct Config {
    // Server
    pub host: String,
    pub port: u16,
    pub environment: String,

    // Database
    pub database_url: String,

    // JWT
    pub jwt_secret: String,
    pub jwt_access_expiry_secs: i64,
    pub jwt_refresh_expiry_days: i64,

    // OAuth
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_uri: String,

    // CORS
    pub allowed_origins: Vec<String>,

    // R2 Storage
    pub r2_account_id: String,
    pub r2_access_key: String,
    pub r2_secret_key: String,
    pub r2_bucket: String,

    // OpenAI (for embeddings)
    pub openai_api_key: String,

    // Frontend URLs
    pub web_app_url: String,
    pub desktop_scheme: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            // Server
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .context("Invalid PORT")?,
            environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),

            // Database
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL must be set")?,

            // JWT
            jwt_secret: std::env::var("JWT_SECRET")
                .context("JWT_SECRET must be set")?,
            jwt_access_expiry_secs: std::env::var("JWT_ACCESS_EXPIRY_SECS")
                .unwrap_or_else(|_| "900".to_string()) // 15 minutes
                .parse()
                .context("Invalid JWT_ACCESS_EXPIRY_SECS")?,
            jwt_refresh_expiry_days: std::env::var("JWT_REFRESH_EXPIRY_DAYS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .context("Invalid JWT_REFRESH_EXPIRY_DAYS")?,

            // OAuth
            google_client_id: std::env::var("GOOGLE_CLIENT_ID")
                .unwrap_or_default(),
            google_client_secret: std::env::var("GOOGLE_CLIENT_SECRET")
                .unwrap_or_default(),
            google_redirect_uri: std::env::var("GOOGLE_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:3000/auth/callback".to_string()),

            // CORS
            allowed_origins: std::env::var("ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),

            // R2 Storage
            r2_account_id: std::env::var("R2_ACCOUNT_ID").unwrap_or_default(),
            r2_access_key: std::env::var("R2_ACCESS_KEY").unwrap_or_default(),
            r2_secret_key: std::env::var("R2_SECRET_KEY").unwrap_or_default(),
            r2_bucket: std::env::var("R2_BUCKET").unwrap_or_else(|_| "hekax".to_string()),

            // OpenAI
            openai_api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),

            // Frontend URLs
            web_app_url: std::env::var("WEB_APP_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            desktop_scheme: std::env::var("DESKTOP_SCHEME")
                .unwrap_or_else(|_| "hekax".to_string()),
        })
    }

    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }
}
