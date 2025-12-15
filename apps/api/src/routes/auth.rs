//! Authentication routes

use axum::{
    extract::{Extension, State},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use validator::Validate;

use crate::{
    auth::{
        create_access_token, create_refresh_token, hash_password, rotate_refresh_token,
        revoke_all_refresh_tokens, revoke_refresh_token, verify_password, AuthUser,
    },
    error::{ApiError, ApiResult},
    models::{
        AuthResponse, LoginRequest, RefreshRequest, RegisterRequest, User, UserInfo,
    },
    AppState,
};

pub fn router() -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/logout-all", post(logout_all))
        .route("/me", get(me))
}

/// Register a new user
async fn register(
    Extension(state): Extension<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Validate request
    req.validate().map_err(|e| ApiError::Validation(e.to_string()))?;

    // Check if user exists
    let existing = sqlx::query!(
        r#"SELECT id FROM users WHERE email = $1"#,
        req.email.to_lowercase()
    )
    .fetch_optional(state.db.pool())
    .await?;

    if existing.is_some() {
        return Err(ApiError::UserAlreadyExists);
    }

    // Hash password
    let password_hash = hash_password(&req.password)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Create user
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (email, password_hash, full_name)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
        req.email.to_lowercase(),
        password_hash,
        req.full_name
    )
    .fetch_one(state.db.pool())
    .await?;

    // Create default settings
    sqlx::query!(
        r#"INSERT INTO user_settings (user_id) VALUES ($1)"#,
        user.id
    )
    .execute(state.db.pool())
    .await?;

    // Generate device ID for new registration
    let device_id = uuid::Uuid::new_v4().to_string();

    // Create tokens
    let access_token = create_access_token(
        user.id,
        &user.email,
        &user.subscription_tier,
        user.token_version,
        &state.config.jwt_secret,
        state.config.jwt_access_expiry_secs,
    )?;

    let refresh_token = create_refresh_token(
        state.db.pool(),
        user.id,
        &device_id,
        Some("Web Registration"),
        user.token_version,
        state.config.jwt_refresh_expiry_days,
    )
    .await?;

    // Log activity
    sqlx::query!(
        r#"
        INSERT INTO activity_log (user_id, activity_type, metadata)
        VALUES ($1, 'register', '{}')
        "#,
        user.id
    )
    .execute(state.db.pool())
    .await?;

    Ok(Json(AuthResponse {
        user: UserInfo::from(user),
        access_token,
        refresh_token,
        expires_in: state.config.jwt_access_expiry_secs,
    }))
}

/// Login with email/password
async fn login(
    Extension(state): Extension<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
    req.validate().map_err(|e| ApiError::Validation(e.to_string()))?;

    // Find user
    let user = sqlx::query_as!(
        User,
        r#"SELECT * FROM users WHERE email = $1"#,
        req.email.to_lowercase()
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or(ApiError::InvalidCredentials)?;

    // Verify password
    let password_hash = user.password_hash.as_ref().ok_or(ApiError::InvalidCredentials)?;
    if !verify_password(&req.password, password_hash) {
        return Err(ApiError::InvalidCredentials);
    }

    // Create tokens
    let access_token = create_access_token(
        user.id,
        &user.email,
        &user.subscription_tier,
        user.token_version,
        &state.config.jwt_secret,
        state.config.jwt_access_expiry_secs,
    )?;

    let refresh_token = create_refresh_token(
        state.db.pool(),
        user.id,
        &req.device_id,
        req.device_name.as_deref(),
        user.token_version,
        state.config.jwt_refresh_expiry_days,
    )
    .await?;

    // Log activity
    sqlx::query!(
        r#"
        INSERT INTO activity_log (user_id, activity_type, metadata)
        VALUES ($1, 'login', $2)
        "#,
        user.id,
        serde_json::json!({ "device_id": req.device_id })
    )
    .execute(state.db.pool())
    .await?;

    Ok(Json(AuthResponse {
        user: UserInfo::from(user),
        access_token,
        refresh_token,
        expires_in: state.config.jwt_access_expiry_secs,
    }))
}

/// Refresh access token (with rotation)
async fn refresh(
    Extension(state): Extension<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Rotate refresh token
    let (user_id, new_refresh_token, token_version) = rotate_refresh_token(
        state.db.pool(),
        &req.refresh_token,
        &req.device_id,
        state.config.jwt_refresh_expiry_days,
    )
    .await?;

    // Get user
    let user = sqlx::query_as!(
        User,
        r#"SELECT * FROM users WHERE id = $1"#,
        user_id
    )
    .fetch_one(state.db.pool())
    .await?;

    // Create new access token
    let access_token = create_access_token(
        user.id,
        &user.email,
        &user.subscription_tier,
        token_version,
        &state.config.jwt_secret,
        state.config.jwt_access_expiry_secs,
    )?;

    Ok(Json(AuthResponse {
        user: UserInfo::from(user),
        access_token,
        refresh_token: new_refresh_token,
        expires_in: state.config.jwt_access_expiry_secs,
    }))
}

/// Logout (revoke refresh token for current device)
async fn logout(
    Extension(state): Extension<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<RefreshRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    revoke_refresh_token(state.db.pool(), auth.id, &req.device_id).await?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// Logout all devices (increment token_version)
async fn logout_all(
    Extension(state): Extension<Arc<AppState>>,
    auth: AuthUser,
) -> ApiResult<Json<serde_json::Value>> {
    revoke_all_refresh_tokens(state.db.pool(), auth.id).await?;

    // Log activity
    sqlx::query!(
        r#"
        INSERT INTO activity_log (user_id, activity_type)
        VALUES ($1, 'logout_all')
        "#,
        auth.id
    )
    .execute(state.db.pool())
    .await?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// Get current user info
async fn me(
    Extension(state): Extension<Arc<AppState>>,
    auth: AuthUser,
) -> ApiResult<Json<UserInfo>> {
    let user = sqlx::query_as!(
        User,
        r#"SELECT * FROM users WHERE id = $1"#,
        auth.id
    )
    .fetch_one(state.db.pool())
    .await?;

    Ok(Json(UserInfo::from(user)))
}
