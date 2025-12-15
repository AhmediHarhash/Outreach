//! Analytics Export
//!
//! Export conversation analytics to various formats.

use super::{SessionAnalytics, Speaker};
use chrono::{DateTime, Utc};
use serde::Serialize;

/// Exportable analytics data
#[derive(Debug, Serialize)]
pub struct AnalyticsExport {
    pub session_start: String,
    pub session_end: Option<String>,
    pub duration_minutes: f32,
    pub mode: String,
    pub talk_ratio_percent: u32,
    pub total_turns: usize,
    pub user_metrics: ExportedMetrics,
    pub other_metrics: ExportedMetrics,
    pub top_topics: Vec<TopicExport>,
    pub average_sentiment: String,
    pub turns: Vec<TurnExport>,
}

#[derive(Debug, Serialize)]
pub struct ExportedMetrics {
    pub talk_time_seconds: f32,
    pub turn_count: usize,
    pub word_count: usize,
    pub question_count: usize,
    pub words_per_minute: f32,
}

#[derive(Debug, Serialize)]
pub struct TopicExport {
    pub topic: String,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct TurnExport {
    pub timestamp: String,
    pub speaker: String,
    pub text: String,
    pub duration_ms: u64,
    pub word_count: usize,
}

impl From<&SessionAnalytics> for AnalyticsExport {
    fn from(session: &SessionAnalytics) -> Self {
        Self {
            session_start: session.start_time.to_rfc3339(),
            session_end: session.end_time.map(|t| t.to_rfc3339()),
            duration_minutes: session.duration().num_minutes() as f32,
            mode: session.mode.clone(),
            talk_ratio_percent: (session.talk_ratio() * 100.0).round() as u32,
            total_turns: session.turns.len(),
            user_metrics: ExportedMetrics {
                talk_time_seconds: session.metrics.user.total_talk_time_ms as f32 / 1000.0,
                turn_count: session.metrics.user.turn_count,
                word_count: session.metrics.user.word_count,
                question_count: session.metrics.user.question_count,
                words_per_minute: session.metrics.user.words_per_minute(),
            },
            other_metrics: ExportedMetrics {
                talk_time_seconds: session.metrics.other.total_talk_time_ms as f32 / 1000.0,
                turn_count: session.metrics.other.turn_count,
                word_count: session.metrics.other.word_count,
                question_count: session.metrics.other.question_count,
                words_per_minute: session.metrics.other.words_per_minute(),
            },
            top_topics: session.top_topics(10).into_iter().map(|(t, c)| TopicExport {
                topic: t.clone(),
                count: c,
            }).collect(),
            average_sentiment: session.average_sentiment().label().to_string(),
            turns: session.turns.iter().map(|t| TurnExport {
                timestamp: t.timestamp.to_rfc3339(),
                speaker: match t.speaker {
                    Speaker::User => "You".to_string(),
                    Speaker::Other => "Them".to_string(),
                },
                text: t.text.clone(),
                duration_ms: t.duration_ms,
                word_count: t.word_count,
            }).collect(),
        }
    }
}

/// Export to JSON format
pub fn export_to_json(session: &SessionAnalytics) -> String {
    let export = AnalyticsExport::from(session);
    serde_json::to_string_pretty(&export).unwrap_or_else(|_| "{}".to_string())
}

/// Export to CSV format
pub fn export_to_csv(session: &SessionAnalytics) -> String {
    let mut csv = String::new();

    // Header
    csv.push_str("Timestamp,Speaker,Text,Duration (ms),Word Count\n");

    // Turns
    for turn in &session.turns {
        let speaker = match turn.speaker {
            Speaker::User => "You",
            Speaker::Other => "Them",
        };
        let text = turn.text.replace("\"", "\"\""); // Escape quotes
        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",{},{}\n",
            turn.timestamp.to_rfc3339(),
            speaker,
            text,
            turn.duration_ms,
            turn.word_count
        ));
    }

    csv
}

/// Export to Markdown format
pub fn export_to_markdown(session: &SessionAnalytics) -> String {
    let summary = session.summary();
    let mut md = String::new();

    md.push_str("# Conversation Analytics\n\n");
    md.push_str(&format!("**Date:** {}\n\n", session.start_time.format("%Y-%m-%d %H:%M")));
    md.push_str(&format!("**Mode:** {}\n\n", session.mode));
    md.push_str(&format!("**Duration:** {:.1} minutes\n\n", summary.duration_minutes));

    md.push_str("## Summary\n\n");
    md.push_str(&format!("| Metric | You | Them |\n"));
    md.push_str("|--------|-----|------|\n");
    md.push_str(&format!(
        "| Talk Time | {:.1}s | {:.1}s |\n",
        session.metrics.user.total_talk_time_ms as f32 / 1000.0,
        session.metrics.other.total_talk_time_ms as f32 / 1000.0
    ));
    md.push_str(&format!(
        "| Turns | {} | {} |\n",
        session.metrics.user.turn_count,
        session.metrics.other.turn_count
    ));
    md.push_str(&format!(
        "| Words | {} | {} |\n",
        session.metrics.user.word_count,
        session.metrics.other.word_count
    ));
    md.push_str(&format!(
        "| Questions | {} | {} |\n",
        session.metrics.user.question_count,
        session.metrics.other.question_count
    ));
    md.push_str(&format!(
        "| WPM | {:.0} | {:.0} |\n",
        summary.words_per_minute_user,
        summary.words_per_minute_other
    ));

    md.push_str(&format!("\n**Talk Ratio:** {}% you, {}% them\n\n",
        summary.talk_ratio_percent,
        100 - summary.talk_ratio_percent
    ));

    md.push_str(&format!("**Overall Sentiment:** {} {}\n\n",
        summary.average_sentiment.emoji(),
        summary.average_sentiment.label()
    ));

    if !summary.top_topics.is_empty() {
        md.push_str("## Top Topics\n\n");
        for (topic, count) in &summary.top_topics {
            md.push_str(&format!("- **{}** (mentioned {} times)\n", topic, count));
        }
        md.push_str("\n");
    }

    md.push_str("## Conversation Transcript\n\n");
    for turn in &session.turns {
        let speaker = match turn.speaker {
            Speaker::User => "**You**",
            Speaker::Other => "**Them**",
        };
        md.push_str(&format!(
            "{} ({}): {}\n\n",
            speaker,
            turn.timestamp.format("%H:%M:%S"),
            turn.text
        ));
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_export() {
        let mut session = SessionAnalytics::new("test");
        session.add_turn(Speaker::User, "Hello, how are you?", 2000);
        session.add_turn(Speaker::Other, "I'm good, thanks!", 1500);

        let json = export_to_json(&session);
        assert!(json.contains("test"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_csv_export() {
        let mut session = SessionAnalytics::new("test");
        session.add_turn(Speaker::User, "Test message", 1000);

        let csv = export_to_csv(&session);
        assert!(csv.contains("Timestamp,Speaker,Text"));
        assert!(csv.contains("Test message"));
    }

    #[test]
    fn test_markdown_export() {
        let mut session = SessionAnalytics::new("sales");
        session.add_turn(Speaker::User, "What's the pricing?", 2000);
        session.add_turn(Speaker::Other, "Let me explain our plans.", 3000);

        let md = export_to_markdown(&session);
        assert!(md.contains("# Conversation Analytics"));
        assert!(md.contains("**Mode:** sales"));
    }
}
