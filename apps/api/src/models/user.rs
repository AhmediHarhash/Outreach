//! User model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// User database model
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
    #[serde(skip_serializing)]
    pub google_id: Option<String>,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub subscription_tier: String,
    pub subscription_expires_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing)]
    pub token_version: i32,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Public user info (safe to send to client)
#[derive(Debug, Clone, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub subscription_tier: String,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            avatar_url: user.avatar_url,
            subscription_tier: user.subscription_tier,
            email_verified: user.email_verified,
            created_at: user.created_at,
        }
    }
}

/// Registration request
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,

    #[validate(length(min = 1, max = 255, message = "Name is required"))]
    pub full_name: String,
}

/// Login request
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,

    pub password: String,

    /// Device ID for refresh token binding
    pub device_id: String,

    /// Device name (e.g., "Desktop App", "Chrome on Windows")
    pub device_name: Option<String>,
}

/// Auth response with tokens
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: UserInfo,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64, // seconds
}

/// Refresh token request
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
    pub device_id: String,
}

/// Google OAuth request
#[derive(Debug, Deserialize)]
pub struct GoogleAuthRequest {
    pub code: String,
    pub device_id: String,
    pub device_name: Option<String>,
}

/// User settings
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserSettings {
    pub id: Uuid,
    pub user_id: Uuid,
    pub default_mode: String,
    pub auto_record: bool,
    pub stealth_mode_default: bool,
    pub theme: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Update settings request
#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub default_mode: Option<String>,
    pub auto_record: Option<bool>,
    pub stealth_mode_default: Option<bool>,
    pub theme: Option<String>,
}
