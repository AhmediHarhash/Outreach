//! User routes

use axum::{
    extract::Extension,
    routing::{get, put},
    Json, Router,
};
use std::sync::Arc;

use crate::{
    auth::AuthUser,
    error::ApiResult,
    models::{UpdateSettingsRequest, UserSettings},
    AppState,
};

pub fn router() -> Router {
    Router::new()
        .route("/settings", get(get_settings).put(update_settings))
}

/// Get user settings
async fn get_settings(
    Extension(state): Extension<Arc<AppState>>,
    auth: AuthUser,
) -> ApiResult<Json<UserSettings>> {
    let settings = sqlx::query_as!(
        UserSettings,
        r#"
        SELECT * FROM user_settings WHERE user_id = $1
        "#,
        auth.id
    )
    .fetch_optional(state.db.pool())
    .await?;

    match settings {
        Some(s) => Ok(Json(s)),
        None => {
            // Create default settings if not exists
            let settings = sqlx::query_as!(
                UserSettings,
                r#"
                INSERT INTO user_settings (user_id)
                VALUES ($1)
                RETURNING *
                "#,
                auth.id
            )
            .fetch_one(state.db.pool())
            .await?;
            Ok(Json(settings))
        }
    }
}

/// Update user settings
async fn update_settings(
    Extension(state): Extension<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<UpdateSettingsRequest>,
) -> ApiResult<Json<UserSettings>> {
    let settings = sqlx::query_as!(
        UserSettings,
        r#"
        UPDATE user_settings
        SET
            default_mode = COALESCE($2, default_mode),
            auto_record = COALESCE($3, auto_record),
            stealth_mode_default = COALESCE($4, stealth_mode_default),
            theme = COALESCE($5, theme),
            updated_at = NOW()
        WHERE user_id = $1
        RETURNING *
        "#,
        auth.id,
        req.default_mode,
        req.auto_record,
        req.stealth_mode_default,
        req.theme
    )
    .fetch_one(state.db.pool())
    .await?;

    Ok(Json(settings))
}
