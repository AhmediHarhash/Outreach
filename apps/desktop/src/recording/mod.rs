//! Session Recording Module
//!
//! Records conversations with full transcript, AI suggestions, and metadata.
//! Provides AI-powered call summary and self-analysis.

mod session;
mod summary;
mod storage;

pub use session::{RecordingSession, RecordingState, RecordedTurn, RecordedSuggestion};
pub use summary::{CallSummary, SelfAnalysis, PerformanceScore, generate_call_summary};
pub use storage::{save_recording, load_recording, list_recordings, delete_recording};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

/// Recording manager - controls recording state
pub struct RecordingManager {
    current_session: Arc<RwLock<Option<RecordingSession>>>,
    is_recording: Arc<RwLock<bool>>,
    is_paused: Arc<RwLock<bool>>,
    auto_record: bool,
}

impl RecordingManager {
    pub fn new() -> Self {
        Self {
            current_session: Arc::new(RwLock::new(None)),
            is_recording: Arc::new(RwLock::new(false)),
            is_paused: Arc::new(RwLock::new(false)),
            auto_record: false,
        }
    }

    /// Enable auto-recording for all sessions
    pub fn set_auto_record(&mut self, enabled: bool) {
        self.auto_record = enabled;
    }

    /// Start a new recording session
    pub fn start_recording(&self, mode: &str) {
        let mut session = self.current_session.write();
        *session = Some(RecordingSession::new(mode));
        *self.is_recording.write() = true;
        *self.is_paused.write() = false;
        tracing::info!("Recording started for mode: {}", mode);
    }

    /// Pause recording (during sensitive parts)
    pub fn pause(&self) {
        if *self.is_recording.read() {
            *self.is_paused.write() = true;
            if let Some(ref mut session) = *self.current_session.write() {
                session.add_event(SessionEvent::Paused);
            }
            tracing::info!("Recording paused");
        }
    }

    /// Resume recording
    pub fn resume(&self) {
        if *self.is_recording.read() {
            *self.is_paused.write() = false;
            if let Some(ref mut session) = *self.current_session.write() {
                session.add_event(SessionEvent::Resumed);
            }
            tracing::info!("Recording resumed");
        }
    }

    /// Toggle pause state
    pub fn toggle_pause(&self) {
        if *self.is_paused.read() {
            self.resume();
        } else {
            self.pause();
        }
    }

    /// Check if recording
    pub fn is_recording(&self) -> bool {
        *self.is_recording.read()
    }

    /// Check if paused
    pub fn is_paused(&self) -> bool {
        *self.is_paused.read()
    }

    /// Add a turn to the recording
    pub fn add_turn(&self, speaker: Speaker, text: &str, duration_ms: u64) {
        if !*self.is_recording.read() || *self.is_paused.read() {
            return;
        }

        if let Some(ref mut session) = *self.current_session.write() {
            session.add_turn(RecordedTurn {
                timestamp: Utc::now(),
                speaker,
                text: text.to_string(),
                duration_ms,
            });
        }
    }

    /// Add AI suggestion to recording
    pub fn add_suggestion(&self, suggestion_type: SuggestionType, content: &str, was_used: bool) {
        if !*self.is_recording.read() || *self.is_paused.read() {
            return;
        }

        if let Some(ref mut session) = *self.current_session.write() {
            session.add_suggestion(RecordedSuggestion {
                timestamp: Utc::now(),
                suggestion_type,
                content: content.to_string(),
                was_used,
            });
        }
    }

    /// Stop recording and return the session
    pub fn stop_recording(&self) -> Option<RecordingSession> {
        *self.is_recording.write() = false;
        *self.is_paused.write() = false;

        let mut session = self.current_session.write();
        if let Some(ref mut s) = *session {
            s.end_session();
        }
        session.take()
    }

    /// Get current session duration
    pub fn current_duration(&self) -> Option<chrono::Duration> {
        self.current_session.read().as_ref().map(|s| s.duration())
    }

    /// Get current turn count
    pub fn turn_count(&self) -> usize {
        self.current_session.read().as_ref().map(|s| s.turns.len()).unwrap_or(0)
    }
}

impl Default for RecordingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Speaker identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Speaker {
    /// The user (you)
    User,
    /// The other person (them)
    Other,
    /// Unknown or system
    System,
}

impl Speaker {
    pub fn label(&self) -> &'static str {
        match self {
            Self::User => "You",
            Self::Other => "Them",
            Self::System => "System",
        }
    }
}

/// Type of AI suggestion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SuggestionType {
    /// Flash bullet point
    Flash,
    /// Deep response
    Deep,
    /// Question to ask
    Question,
    /// Warning/caution
    Warning,
}

/// Session events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionEvent {
    Started,
    Paused,
    Resumed,
    ModeChanged(String),
    Ended,
}
