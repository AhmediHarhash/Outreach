//! Recording model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Recording database model
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Recording {
    pub id: Uuid,
    pub user_id: Uuid,
    pub lead_id: Option<Uuid>,

    pub mode: String,
    pub status: String,

    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i32>,

    pub transcript_turns: Option<serde_json::Value>,
    pub summary: Option<String>,
    pub key_points: Option<serde_json::Value>,
    pub action_items: Option<serde_json::Value>,

    pub talk_ratio: Option<f64>,
    pub user_word_count: Option<i32>,
    pub other_word_count: Option<i32>,
    pub user_wpm: Option<f64>,
    pub question_count: Option<i32>,
    pub objection_count: Option<i32>,
    pub sentiment_score: Option<f64>,

    pub performance_score: Option<serde_json::Value>,
    pub outcome: Option<String>,

    pub audio_r2_key: Option<String>,
    pub transcript_r2_key: Option<String>,

    pub created_at: DateTime<Utc>,
}

/// Create recording request
#[derive(Debug, Deserialize)]
pub struct CreateRecordingRequest {
    pub lead_id: Option<Uuid>,
    pub mode: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub transcript_turns: Option<serde_json::Value>,
}

/// Upload recording request (after recording is created)
#[derive(Debug, Deserialize)]
pub struct UploadRecordingRequest {
    pub transcript_turns: serde_json::Value,
    pub end_time: DateTime<Utc>,
    pub duration_seconds: i32,
    pub talk_ratio: f64,
    pub user_word_count: i32,
    pub other_word_count: i32,
    pub user_wpm: f64,
}

/// Recording list query parameters
#[derive(Debug, Deserialize)]
pub struct RecordingListQuery {
    pub lead_id: Option<Uuid>,
    pub mode: Option<String>,
    pub status: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

/// Recording list response
#[derive(Debug, Serialize)]
pub struct RecordingListResponse {
    pub recordings: Vec<RecordingSummary>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

/// Recording summary for list view
#[derive(Debug, Clone, Serialize)]
pub struct RecordingSummary {
    pub id: Uuid,
    pub lead_id: Option<Uuid>,
    pub lead_name: Option<String>,
    pub mode: String,
    pub status: String,
    pub start_time: DateTime<Utc>,
    pub duration_seconds: Option<i32>,
    pub summary: Option<String>,
    pub outcome: Option<String>,
    pub sentiment_score: Option<f64>,
}

/// Conversation turn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptTurn {
    pub speaker: String,  // "user" or "other"
    pub text: String,
    pub timestamp_ms: i64,
    pub duration_ms: i64,
}

/// Performance score breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceScore {
    pub overall: i32,
    pub listening: i32,
    pub response_quality: i32,
    pub delivery: i32,
    pub suggestion_usage: i32,
    pub outcome: i32,
    pub grade: String,
    pub assessment: String,
}
