//! Recording Session
//!
//! Stores all data for a single conversation recording.

use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Speaker, SuggestionType, SessionEvent};

/// A complete recording session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSession {
    /// Unique session ID
    pub id: String,

    /// Session start time
    pub start_time: DateTime<Utc>,

    /// Session end time
    pub end_time: Option<DateTime<Utc>>,

    /// Conversation mode
    pub mode: String,

    /// All conversation turns
    pub turns: Vec<RecordedTurn>,

    /// All AI suggestions provided
    pub suggestions: Vec<RecordedSuggestion>,

    /// Session events (pause, resume, etc.)
    pub events: Vec<(DateTime<Utc>, SessionEvent)>,

    /// Current recording state
    pub state: RecordingState,

    /// Metadata
    pub metadata: SessionMetadata,
}

impl RecordingSession {
    pub fn new(mode: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            start_time: Utc::now(),
            end_time: None,
            mode: mode.to_string(),
            turns: Vec::new(),
            suggestions: Vec::new(),
            events: vec![(Utc::now(), SessionEvent::Started)],
            state: RecordingState::Recording,
            metadata: SessionMetadata::default(),
        }
    }

    /// Add a conversation turn
    pub fn add_turn(&mut self, turn: RecordedTurn) {
        // Update metadata
        match turn.speaker {
            Speaker::User => {
                self.metadata.user_word_count += turn.text.split_whitespace().count();
                self.metadata.user_talk_time_ms += turn.duration_ms;
            }
            Speaker::Other => {
                self.metadata.other_word_count += turn.text.split_whitespace().count();
                self.metadata.other_talk_time_ms += turn.duration_ms;
            }
            _ => {}
        }

        self.turns.push(turn);
    }

    /// Add an AI suggestion
    pub fn add_suggestion(&mut self, suggestion: RecordedSuggestion) {
        self.metadata.total_suggestions += 1;
        if suggestion.was_used {
            self.metadata.suggestions_used += 1;
        }
        self.suggestions.push(suggestion);
    }

    /// Add a session event
    pub fn add_event(&mut self, event: SessionEvent) {
        self.events.push((Utc::now(), event));
    }

    /// End the session
    pub fn end_session(&mut self) {
        self.end_time = Some(Utc::now());
        self.state = RecordingState::Completed;
        self.events.push((Utc::now(), SessionEvent::Ended));
    }

    /// Get session duration
    pub fn duration(&self) -> Duration {
        let end = self.end_time.unwrap_or_else(Utc::now);
        end - self.start_time
    }

    /// Get full transcript as string
    pub fn full_transcript(&self) -> String {
        self.turns
            .iter()
            .map(|t| format!("{}: {}", t.speaker.label(), t.text))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Get user turns only
    pub fn user_turns(&self) -> Vec<&RecordedTurn> {
        self.turns.iter().filter(|t| t.speaker == Speaker::User).collect()
    }

    /// Get other person's turns only
    pub fn other_turns(&self) -> Vec<&RecordedTurn> {
        self.turns.iter().filter(|t| t.speaker == Speaker::Other).collect()
    }

    /// Get questions asked by other person
    pub fn questions_asked(&self) -> Vec<&RecordedTurn> {
        self.turns
            .iter()
            .filter(|t| t.speaker == Speaker::Other && t.text.contains('?'))
            .collect()
    }

    /// Calculate talk ratio (user vs other)
    pub fn talk_ratio(&self) -> f32 {
        let total = self.metadata.user_talk_time_ms + self.metadata.other_talk_time_ms;
        if total == 0 {
            return 0.5;
        }
        self.metadata.user_talk_time_ms as f32 / total as f32
    }
}

/// Recording state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecordingState {
    Recording,
    Paused,
    Completed,
}

/// A single conversation turn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedTurn {
    pub timestamp: DateTime<Utc>,
    pub speaker: Speaker,
    pub text: String,
    pub duration_ms: u64,
}

impl RecordedTurn {
    pub fn word_count(&self) -> usize {
        self.text.split_whitespace().count()
    }

    pub fn is_question(&self) -> bool {
        self.text.trim().ends_with('?')
    }
}

/// A recorded AI suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedSuggestion {
    pub timestamp: DateTime<Utc>,
    pub suggestion_type: SuggestionType,
    pub content: String,
    pub was_used: bool,
}

/// Session metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// User's total word count
    pub user_word_count: usize,
    /// Other person's total word count
    pub other_word_count: usize,
    /// User's total talk time (ms)
    pub user_talk_time_ms: u64,
    /// Other person's total talk time (ms)
    pub other_talk_time_ms: u64,
    /// Total suggestions provided
    pub total_suggestions: usize,
    /// Suggestions actually used
    pub suggestions_used: usize,
    /// Number of pauses
    pub pause_count: usize,
}

impl SessionMetadata {
    /// Calculate suggestion usage rate
    pub fn suggestion_usage_rate(&self) -> f32 {
        if self.total_suggestions == 0 {
            return 0.0;
        }
        self.suggestions_used as f32 / self.total_suggestions as f32
    }

    /// Calculate words per minute for user
    pub fn user_wpm(&self) -> f32 {
        if self.user_talk_time_ms == 0 {
            return 0.0;
        }
        let minutes = self.user_talk_time_ms as f32 / 60000.0;
        self.user_word_count as f32 / minutes
    }
}
