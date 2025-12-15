//! JWT token handling
//!
//! Access tokens are short-lived (15 min) and contain user claims.
//! They should be stored in memory only, never persisted.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};

/// JWT claims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: Uuid,
    /// User email
    pub email: String,
    /// Subscription tier
    pub tier: String,
    /// Token version (must match user's token_version)
    pub token_version: i32,
    /// Issued at
    pub iat: i64,
    /// Expiration
    pub exp: i64,
}

impl Claims {
    pub fn new(user_id: Uuid, email: &str, tier: &str, token_version: i32, expiry_secs: i64) -> Self {
        let now = Utc::now();
        Self {
            sub: user_id,
            email: email.to_string(),
            tier: tier.to_string(),
            token_version,
            iat: now.timestamp(),
            exp: (now + Duration::seconds(expiry_secs)).timestamp(),
        }
    }
}

/// Create a new access token
pub fn create_access_token(
    user_id: Uuid,
    email: &str,
    tier: &str,
    token_version: i32,
    secret: &str,
    expiry_secs: i64,
) -> ApiResult<String> {
    let claims = Claims::new(user_id, email, tier, token_version, expiry_secs);

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| ApiError::Internal(format!("Failed to create token: {}", e)))
}

/// Decode and validate an access token
pub fn decode_access_token(token: &str, secret: &str) -> ApiResult<Claims> {
    let validation = Validation::default();

    let token_data: TokenData<Claims> = decode(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => ApiError::TokenExpired,
        _ => ApiError::InvalidToken,
    })?;

    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_decode_token() {
        let user_id = Uuid::new_v4();
        let secret = "test_secret_key_123";

        let token = create_access_token(
            user_id,
            "test@example.com",
            "pro",
            1,
            secret,
            900, // 15 min
        )
        .unwrap();

        let claims = decode_access_token(&token, secret).unwrap();

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.tier, "pro");
        assert_eq!(claims.token_version, 1);
    }

    #[test]
    fn test_invalid_secret() {
        let user_id = Uuid::new_v4();
        let token = create_access_token(user_id, "test@example.com", "free", 0, "secret1", 900).unwrap();

        let result = decode_access_token(&token, "wrong_secret");
        assert!(matches!(result, Err(ApiError::InvalidToken)));
    }
}
