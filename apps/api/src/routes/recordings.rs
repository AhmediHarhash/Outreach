//! Recording routes

use axum::{
    extract::{Extension, Path, Query},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    models::{
        CreateRecordingRequest, Recording, RecordingListQuery, RecordingListResponse,
        RecordingSummary, UploadRecordingRequest,
    },
    AppState,
};

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_recordings).post(create_recording))
        .route("/:id", get(get_recording))
        .route("/:id/upload", post(upload_recording_data))
        .route("/:id/presigned-url", get(get_presigned_upload_url))
}

/// List recordings with filtering and pagination
async fn list_recordings(
    Extension(state): Extension<Arc<AppState>>,
    auth: AuthUser,
    Query(query): Query<RecordingListQuery>,
) -> ApiResult<Json<RecordingListResponse>> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    // Get recordings with lead name join
    let recordings = sqlx::query!(
        r#"
        SELECT
            r.id, r.lead_id, r.mode, r.status, r.start_time,
            r.duration_seconds, r.summary, r.outcome, r.sentiment_score,
            l.company_name as lead_name
        FROM recordings r
        LEFT JOIN leads l ON l.id = r.lead_id
        WHERE r.user_id = $1
          AND ($2::uuid IS NULL OR r.lead_id = $2)
          AND ($3::text IS NULL OR r.mode = $3)
          AND ($4::text IS NULL OR r.status = $4)
          AND ($5::timestamptz IS NULL OR r.start_time >= $5)
          AND ($6::timestamptz IS NULL OR r.start_time <= $6)
        ORDER BY r.start_time DESC
        LIMIT $7 OFFSET $8
        "#,
        auth.id,
        query.lead_id,
        query.mode,
        query.status,
        query.from_date,
        query.to_date,
        per_page as i64,
        offset as i64
    )
    .fetch_all(state.db.pool())
    .await?;

    let summaries: Vec<RecordingSummary> = recordings
        .into_iter()
        .map(|r| RecordingSummary {
            id: r.id,
            lead_id: r.lead_id,
            lead_name: r.lead_name,
            mode: r.mode,
            status: r.status,
            start_time: r.start_time,
            duration_seconds: r.duration_seconds,
            summary: r.summary,
            outcome: r.outcome,
            sentiment_score: r.sentiment_score,
        })
        .collect();

    // Get total count
    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM recordings
        WHERE user_id = $1
          AND ($2::uuid IS NULL OR lead_id = $2)
          AND ($3::text IS NULL OR mode = $3)
          AND ($4::text IS NULL OR status = $4)
        "#,
        auth.id,
        query.lead_id,
        query.mode,
        query.status
    )
    .fetch_one(state.db.pool())
    .await?
    .unwrap_or(0);

    Ok(Json(RecordingListResponse {
        recordings: summaries,
        total,
        page,
        per_page,
    }))
}

/// Create a new recording (called when recording starts)
async fn create_recording(
    Extension(state): Extension<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<CreateRecordingRequest>,
) -> ApiResult<Json<Recording>> {
    let recording = sqlx::query_as!(
        Recording,
        r#"
        INSERT INTO recordings (user_id, lead_id, mode, status, start_time, transcript_turns)
        VALUES ($1, $2, $3, 'recording', $4, $5)
        RETURNING *
        "#,
        auth.id,
        req.lead_id,
        req.mode,
        req.start_time,
        req.transcript_turns
    )
    .fetch_one(state.db.pool())
    .await?;

    // Log activity
    sqlx::query!(
        r#"
        INSERT INTO activity_log (user_id, activity_type, entity_type, entity_id, metadata)
        VALUES ($1, 'call_started', 'recording', $2, $3)
        "#,
        auth.id,
        recording.id,
        serde_json::json!({ "mode": req.mode, "lead_id": req.lead_id })
    )
    .execute(state.db.pool())
    .await?;

    Ok(Json(recording))
}

/// Get a single recording
async fn get_recording(
    Extension(state): Extension<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Recording>> {
    let recording = sqlx::query_as!(
        Recording,
        r#"SELECT * FROM recordings WHERE id = $1 AND user_id = $2"#,
        id,
        auth.id
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or(ApiError::NotFound("Recording not found".to_string()))?;

    Ok(Json(recording))
}

/// Upload recording data (called when recording ends)
async fn upload_recording_data(
    Extension(state): Extension<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UploadRecordingRequest>,
) -> ApiResult<Json<Recording>> {
    let recording = sqlx::query_as!(
        Recording,
        r#"
        UPDATE recordings SET
            status = 'processing',
            transcript_turns = $3,
            end_time = $4,
            duration_seconds = $5,
            talk_ratio = $6,
            user_word_count = $7,
            other_word_count = $8,
            user_wpm = $9
        WHERE id = $1 AND user_id = $2
        RETURNING *
        "#,
        id,
        auth.id,
        req.transcript_turns,
        req.end_time,
        req.duration_seconds,
        req.talk_ratio,
        req.user_word_count,
        req.other_word_count,
        req.user_wpm
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or(ApiError::NotFound("Recording not found".to_string()))?;

    // Queue summary generation job
    sqlx::query!(
        r#"
        INSERT INTO jobs (user_id, job_type, input_params)
        VALUES ($1, 'generate_summary', $2)
        "#,
        auth.id,
        serde_json::json!({ "recording_id": id })
    )
    .execute(state.db.pool())
    .await?;

    // Log activity
    sqlx::query!(
        r#"
        INSERT INTO activity_log (user_id, activity_type, entity_type, entity_id, metadata)
        VALUES ($1, 'call_ended', 'recording', $2, $3)
        "#,
        auth.id,
        id,
        serde_json::json!({ "duration_seconds": req.duration_seconds })
    )
    .execute(state.db.pool())
    .await?;

    Ok(Json(recording))
}

/// Get presigned URL for audio upload
async fn get_presigned_upload_url(
    Extension(state): Extension<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    // Verify ownership
    let _recording = sqlx::query!(
        r#"SELECT id FROM recordings WHERE id = $1 AND user_id = $2"#,
        id,
        auth.id
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or(ApiError::NotFound("Recording not found".to_string()))?;

    // Generate R2 key
    let r2_key = format!("recordings/{}/{}.webm", auth.id, id);

    // TODO: Generate presigned URL using aws-sdk-s3
    // For now, return placeholder using config
    let r2_url = std::env::var("R2_PUBLIC_URL").unwrap_or_else(|_| "https://storage.hekax.com".to_string());
    Ok(Json(serde_json::json!({
        "upload_url": format!("{}/{}", r2_url, r2_key),
        "r2_key": r2_key,
        "expires_in": 3600
    })))
}
