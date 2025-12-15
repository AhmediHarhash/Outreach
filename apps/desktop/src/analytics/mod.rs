//! Conversation Analytics Module
//!
//! Tracks and analyzes conversation metrics:
//! - Talk time ratio (you vs them)
//! - Question count and types
//! - Key topics mentioned
//! - Sentiment tracking
//! - Export to various formats

mod metrics;
mod sentiment;
mod export;

pub use metrics::{ConversationMetrics, SpeakerMetrics, TopicTracker};
pub use sentiment::{SentimentAnalyzer, Sentiment};
pub use export::{export_to_json, export_to_csv, export_to_markdown, AnalyticsExport};

use chrono::{DateTime, Utc, Duration};
use parking_lot::RwLock;
use std::sync::Arc;
use std::collections::HashMap;

/// Session analytics tracker
#[derive(Debug)]
pub struct SessionAnalytics {
    /// Session start time
    pub start_time: DateTime<Utc>,
    /// Session end time (if ended)
    pub end_time: Option<DateTime<Utc>>,
    /// Current mode (sales, interview, technical)
    pub mode: String,
    /// All conversation turns
    pub turns: Vec<ConversationTurn>,
    /// Real-time metrics
    pub metrics: ConversationMetrics,
    /// Topic tracker
    pub topics: TopicTracker,
    /// Sentiment over time
    pub sentiment_history: Vec<(DateTime<Utc>, Sentiment)>,
}

impl SessionAnalytics {
    /// Create a new session
    pub fn new(mode: impl Into<String>) -> Self {
        Self {
            start_time: Utc::now(),
            end_time: None,
            mode: mode.into(),
            turns: Vec::new(),
            metrics: ConversationMetrics::default(),
            topics: TopicTracker::new(),
            sentiment_history: Vec::new(),
        }
    }

    /// Add a conversation turn
    pub fn add_turn(&mut self, speaker: Speaker, text: &str, duration_ms: u64) {
        let turn = ConversationTurn {
            timestamp: Utc::now(),
            speaker: speaker.clone(),
            text: text.to_string(),
            duration_ms,
            word_count: text.split_whitespace().count(),
            is_question: text.trim().ends_with('?'),
        };

        // Update metrics
        match speaker {
            Speaker::User => {
                self.metrics.user.total_talk_time_ms += duration_ms;
                self.metrics.user.turn_count += 1;
                self.metrics.user.word_count += turn.word_count;
                if turn.is_question {
                    self.metrics.user.question_count += 1;
                }
            }
            Speaker::Other => {
                self.metrics.other.total_talk_time_ms += duration_ms;
                self.metrics.other.turn_count += 1;
                self.metrics.other.word_count += turn.word_count;
                if turn.is_question {
                    self.metrics.other.question_count += 1;
                }
            }
        }

        // Track topics
        self.topics.extract_topics(text);

        // Track sentiment
        let sentiment = SentimentAnalyzer::analyze(text);
        self.sentiment_history.push((Utc::now(), sentiment));

        self.turns.push(turn);
    }

    /// End the session
    pub fn end_session(&mut self) {
        self.end_time = Some(Utc::now());
    }

    /// Get session duration
    pub fn duration(&self) -> Duration {
        let end = self.end_time.unwrap_or_else(Utc::now);
        end - self.start_time
    }

    /// Get talk time ratio (user vs other)
    pub fn talk_ratio(&self) -> f32 {
        let total = self.metrics.user.total_talk_time_ms + self.metrics.other.total_talk_time_ms;
        if total == 0 {
            return 0.5;
        }
        self.metrics.user.total_talk_time_ms as f32 / total as f32
    }

    /// Get average sentiment
    pub fn average_sentiment(&self) -> Sentiment {
        if self.sentiment_history.is_empty() {
            return Sentiment::Neutral;
        }

        let mut positive = 0;
        let mut negative = 0;
        let mut neutral = 0;

        for (_, sentiment) in &self.sentiment_history {
            match sentiment {
                Sentiment::Positive => positive += 1,
                Sentiment::Negative => negative += 1,
                Sentiment::Neutral => neutral += 1,
                _ => {}
            }
        }

        if positive > negative && positive > neutral {
            Sentiment::Positive
        } else if negative > positive && negative > neutral {
            Sentiment::Negative
        } else {
            Sentiment::Neutral
        }
    }

    /// Get top N topics
    pub fn top_topics(&self, n: usize) -> Vec<(&String, usize)> {
        self.topics.top_topics(n)
    }

    /// Generate summary statistics
    pub fn summary(&self) -> SessionSummary {
        SessionSummary {
            duration_minutes: self.duration().num_minutes() as f32,
            talk_ratio_percent: (self.talk_ratio() * 100.0).round() as u32,
            total_turns: self.turns.len(),
            user_questions: self.metrics.user.question_count,
            other_questions: self.metrics.other.question_count,
            top_topics: self.top_topics(5).into_iter().map(|(t, c)| (t.clone(), c)).collect(),
            average_sentiment: self.average_sentiment(),
            words_per_minute_user: self.metrics.user.words_per_minute(),
            words_per_minute_other: self.metrics.other.words_per_minute(),
        }
    }
}

impl Default for SessionAnalytics {
    fn default() -> Self {
        Self::new("general")
    }
}

/// Speaker identifier
#[derive(Debug, Clone, PartialEq)]
pub enum Speaker {
    User,
    Other,
}

/// A single conversation turn
#[derive(Debug, Clone)]
pub struct ConversationTurn {
    pub timestamp: DateTime<Utc>,
    pub speaker: Speaker,
    pub text: String,
    pub duration_ms: u64,
    pub word_count: usize,
    pub is_question: bool,
}

/// Summary of a session
#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub duration_minutes: f32,
    pub talk_ratio_percent: u32,
    pub total_turns: usize,
    pub user_questions: usize,
    pub other_questions: usize,
    pub top_topics: Vec<(String, usize)>,
    pub average_sentiment: Sentiment,
    pub words_per_minute_user: f32,
    pub words_per_minute_other: f32,
}

/// Thread-safe analytics manager
pub struct AnalyticsManager {
    current_session: Arc<RwLock<Option<SessionAnalytics>>>,
    past_sessions: Arc<RwLock<Vec<SessionAnalytics>>>,
}

impl AnalyticsManager {
    pub fn new() -> Self {
        Self {
            current_session: Arc::new(RwLock::new(None)),
            past_sessions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start a new session
    pub fn start_session(&self, mode: &str) {
        let mut current = self.current_session.write();
        if let Some(old_session) = current.take() {
            self.past_sessions.write().push(old_session);
        }
        *current = Some(SessionAnalytics::new(mode));
    }

    /// End current session
    pub fn end_session(&self) {
        let mut current = self.current_session.write();
        if let Some(ref mut session) = *current {
            session.end_session();
        }
    }

    /// Add a turn to current session
    pub fn add_turn(&self, speaker: Speaker, text: &str, duration_ms: u64) {
        if let Some(ref mut session) = *self.current_session.write() {
            session.add_turn(speaker, text, duration_ms);
        }
    }

    /// Get current session summary
    pub fn current_summary(&self) -> Option<SessionSummary> {
        self.current_session.read().as_ref().map(|s| s.summary())
    }

    /// Get current session metrics
    pub fn current_metrics(&self) -> Option<ConversationMetrics> {
        self.current_session.read().as_ref().map(|s| s.metrics.clone())
    }

    /// Export current session
    pub fn export_current(&self, format: ExportFormat) -> Option<String> {
        self.current_session.read().as_ref().map(|session| {
            match format {
                ExportFormat::Json => export_to_json(session),
                ExportFormat::Csv => export_to_csv(session),
                ExportFormat::Markdown => export_to_markdown(session),
            }
        })
    }
}

impl Default for AnalyticsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Export format options
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Json,
    Csv,
    Markdown,
}
