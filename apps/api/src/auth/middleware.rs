//! Auth middleware
//!
//! Extracts and validates JWT from Authorization header.

use axum::{
    extract::{FromRequestParts, State},
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json, RequestPartsExt,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::error::ApiError;
use super::jwt::{decode_access_token, Claims};

/// Authenticated user extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub tier: String,
    pub token_version: i32,
}

impl From<Claims> for AuthUser {
    fn from(claims: Claims) -> Self {
        Self {
            id: claims.sub,
            email: claims.email,
            tier: claims.tier,
            token_version: claims.token_version,
        }
    }
}

/// Extractor that validates JWT and provides AuthUser
#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Get Authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(ApiError::Unauthorized)?;

        // Extract Bearer token
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(ApiError::Unauthorized)?;

        // Get app state for JWT secret
        let state = parts
            .extensions
            .get::<Arc<AppState>>()
            .ok_or(ApiError::Internal("App state not found".to_string()))?;

        // Decode and validate token
        let claims = decode_access_token(token, &state.config.jwt_secret)?;

        // Verify token version hasn't been invalidated
        // This is checked again against DB in critical operations

        Ok(AuthUser::from(claims))
    }
}

/// Middleware function for routes that require authentication
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<AuthUser, ApiError> {
    // Optionally verify against database that user still exists
    // and token_version matches (for critical operations)
    let user = sqlx::query!(
        r#"
        SELECT token_version FROM users WHERE id = $1
        "#,
        auth.id
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or(ApiError::UserNotFound)?;

    if user.token_version != auth.token_version {
        return Err(ApiError::InvalidToken);
    }

    Ok(auth)
}

/// Optional auth - doesn't fail if no token provided
pub struct OptionalAuthUser(pub Option<AuthUser>);

#[axum::async_trait]
impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match AuthUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(OptionalAuthUser(Some(user))),
            Err(_) => Ok(OptionalAuthUser(None)),
        }
    }
}
