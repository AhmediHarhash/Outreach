//! Bullet Extraction Types
//!
//! Common types and utilities for the Flash response stage.

use serde::{Deserialize, Serialize};

/// Flash analysis result from the fast model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashAnalysis {
    /// One-sentence summary of what they said
    pub summary: String,

    /// Key bullet points to mention
    pub bullets: Vec<Bullet>,

    /// Type of statement detected
    #[serde(rename = "type")]
    pub statement_type: StatementType,

    /// How urgently you need to respond
    pub urgency: Urgency,
}

/// A single bullet point suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bullet {
    /// The suggestion text
    pub point: String,

    /// Priority (1 = highest, say first)
    pub priority: u8,
}

/// Type of statement detected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StatementType {
    Question,
    Objection,
    Statement,
    BuyingSignal,
    Technical,
    SmallTalk,
    #[serde(other)]
    Unknown,
}

impl StatementType {
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Question => "❓",
            Self::Objection => "⚠️",
            Self::Statement => "💬",
            Self::BuyingSignal => "🎯",
            Self::Technical => "🔧",
            Self::SmallTalk => "👋",
            Self::Unknown => "❔",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Question => "Question",
            Self::Objection => "Objection",
            Self::Statement => "Statement",
            Self::BuyingSignal => "Buying Signal",
            Self::Technical => "Technical",
            Self::SmallTalk => "Small Talk",
            Self::Unknown => "Unknown",
        }
    }
}

/// Response urgency level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Urgency {
    AnswerNow,
    CanElaborate,
    JustListening,
    #[serde(other)]
    Unknown,
}

impl Urgency {
    pub fn should_show_flash(&self) -> bool {
        matches!(self, Self::AnswerNow | Self::CanElaborate)
    }
}

/// Extract bullets from a FlashAnalysis, sorted by priority
pub fn extract_bullets(analysis: &FlashAnalysis) -> Vec<&Bullet> {
    let mut bullets: Vec<&Bullet> = analysis.bullets.iter().collect();
    bullets.sort_by_key(|b| b.priority);
    bullets
}

/// Get the top bullet (priority 1)
pub fn get_top_bullet(analysis: &FlashAnalysis) -> Option<&Bullet> {
    analysis.bullets.iter().find(|b| b.priority == 1)
}

impl Default for FlashAnalysis {
    fn default() -> Self {
        Self {
            summary: String::new(),
            bullets: Vec::new(),
            statement_type: StatementType::Unknown,
            urgency: Urgency::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_flash_analysis() {
        let json = r#"{
            "summary": "They're asking about pricing",
            "bullets": [
                {"point": "Mention value first", "priority": 1},
                {"point": "Ask about budget", "priority": 2}
            ],
            "type": "question",
            "urgency": "answer_now"
        }"#;

        let analysis: FlashAnalysis = serde_json::from_str(json).unwrap();
        assert_eq!(analysis.statement_type, StatementType::Question);
        assert_eq!(analysis.urgency, Urgency::AnswerNow);
        assert_eq!(analysis.bullets.len(), 2);
    }

    #[test]
    fn test_extract_bullets() {
        let analysis = FlashAnalysis {
            summary: "Test".to_string(),
            bullets: vec![
                Bullet { point: "Third".to_string(), priority: 3 },
                Bullet { point: "First".to_string(), priority: 1 },
                Bullet { point: "Second".to_string(), priority: 2 },
            ],
            statement_type: StatementType::Question,
            urgency: Urgency::AnswerNow,
        };

        let sorted = extract_bullets(&analysis);
        assert_eq!(sorted[0].point, "First");
        assert_eq!(sorted[1].point, "Second");
        assert_eq!(sorted[2].point, "Third");
    }
}
