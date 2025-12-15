//! Refresh token management
//!
//! Implements secure refresh token handling:
//! - Tokens are never stored in plaintext (SHA256 hash)
//! - Token rotation on every refresh (old token invalidated)
//! - Device binding (one token per device)
//! - Token versioning (user.token_version must match)

use rand::Rng;
use sha2::{Sha256, Digest};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Duration, Utc};

use crate::error::{ApiError, ApiResult};

/// Generate a cryptographically secure random token
pub fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}

/// Hash a token using SHA256
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Create a new refresh token for a user/device
///
/// If a token already exists for this device, it will be replaced.
pub async fn create_refresh_token(
    pool: &PgPool,
    user_id: Uuid,
    device_id: &str,
    device_name: Option<&str>,
    token_version: i32,
    expiry_days: i64,
) -> ApiResult<String> {
    let token = generate_token();
    let token_hash = hash_token(&token);
    let expires_at = Utc::now() + Duration::days(expiry_days);

    // Upsert: insert or replace existing token for this device
    sqlx::query!(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, device_id, device_name, token_version, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (user_id, device_id)
        DO UPDATE SET
            token_hash = $2,
            token_version = $5,
            expires_at = $6,
            last_used_at = NULL,
            created_at = NOW()
        "#,
        user_id,
        token_hash,
        device_id,
        device_name,
        token_version,
        expires_at
    )
    .execute(pool)
    .await?;

    Ok(token)
}

/// Verify a refresh token and return the user ID if valid
pub async fn verify_refresh_token(
    pool: &PgPool,
    token: &str,
    device_id: &str,
) -> ApiResult<(Uuid, i32)> {
    let token_hash = hash_token(token);

    // Find the token
    let record = sqlx::query!(
        r#"
        SELECT rt.user_id, rt.token_version, rt.expires_at, u.token_version as user_token_version
        FROM refresh_tokens rt
        JOIN users u ON u.id = rt.user_id
        WHERE rt.token_hash = $1 AND rt.device_id = $2
        "#,
        token_hash,
        device_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::InvalidToken)?;

    // Check expiration
    if record.expires_at < Utc::now() {
        return Err(ApiError::TokenExpired);
    }

    // Check token version matches user's current version
    // If user changed password or logged out all devices, this will fail
    if record.token_version != record.user_token_version {
        return Err(ApiError::InvalidToken);
    }

    // Update last_used_at
    sqlx::query!(
        r#"
        UPDATE refresh_tokens
        SET last_used_at = NOW()
        WHERE token_hash = $1 AND device_id = $2
        "#,
        token_hash,
        device_id
    )
    .execute(pool)
    .await?;

    Ok((record.user_id, record.token_version))
}

/// Rotate a refresh token (invalidate old, create new)
///
/// This is called on every token refresh to implement rotation.
pub async fn rotate_refresh_token(
    pool: &PgPool,
    old_token: &str,
    device_id: &str,
    expiry_days: i64,
) -> ApiResult<(Uuid, String, i32)> {
    // Verify old token first
    let (user_id, token_version) = verify_refresh_token(pool, old_token, device_id).await?;

    // Create new token (this replaces the old one due to UNIQUE constraint)
    let new_token = create_refresh_token(
        pool,
        user_id,
        device_id,
        None, // Keep existing device name
        token_version,
        expiry_days,
    )
    .await?;

    Ok((user_id, new_token, token_version))
}

/// Invalidate a specific refresh token
pub async fn revoke_refresh_token(
    pool: &PgPool,
    user_id: Uuid,
    device_id: &str,
) -> ApiResult<()> {
    sqlx::query!(
        r#"
        DELETE FROM refresh_tokens
        WHERE user_id = $1 AND device_id = $2
        "#,
        user_id,
        device_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Invalidate all refresh tokens for a user by incrementing token_version
///
/// Called on password change or "logout all devices"
pub async fn revoke_all_refresh_tokens(pool: &PgPool, user_id: Uuid) -> ApiResult<i32> {
    let result = sqlx::query!(
        r#"
        UPDATE users
        SET token_version = token_version + 1, updated_at = NOW()
        WHERE id = $1
        RETURNING token_version
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?;

    // Optionally clean up old tokens (they're invalid anyway)
    sqlx::query!(
        r#"
        DELETE FROM refresh_tokens
        WHERE user_id = $1
        "#,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(result.token_version)
}

/// Clean up expired tokens (should be run periodically)
pub async fn cleanup_expired_tokens(pool: &PgPool) -> ApiResult<u64> {
    let result = sqlx::query!(
        r#"
        DELETE FROM refresh_tokens
        WHERE expires_at < NOW()
        "#
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
