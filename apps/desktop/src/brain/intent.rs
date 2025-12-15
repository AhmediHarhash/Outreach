//! Intent Analysis
//!
//! Analyzes transcripts to detect buyer intent, objections, and signals.

use crate::flash::StatementType;

/// Detected intent from analysis
#[derive(Debug, Clone)]
pub struct DetectedIntent {
    /// Primary intent category
    pub category: IntentCategory,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Whether this needs immediate response
    pub needs_response: bool,
    /// Specific keywords that triggered detection
    pub triggers: Vec<String>,
}

/// Categories of intent
#[derive(Debug, Clone, PartialEq)]
pub enum IntentCategory {
    /// Price/cost questions
    Pricing,
    /// Security/compliance questions
    Security,
    /// Implementation timeline
    Timeline,
    /// Comparison to competitors
    Competition,
    /// Technical capability questions
    Technical,
    /// Buying readiness signals
    BuyingSignal,
    /// Objection/resistance
    Objection,
    /// Request to delay/stall
    Stalling,
    /// Decision maker/process questions
    Procurement,
    /// General small talk
    SmallTalk,
    /// Unknown/other
    Other,
}

impl IntentCategory {
    pub fn from_statement_type(st: &StatementType) -> Self {
        match st {
            StatementType::Question => Self::Other,
            StatementType::Objection => Self::Objection,
            StatementType::BuyingSignal => Self::BuyingSignal,
            StatementType::Technical => Self::Technical,
            StatementType::SmallTalk => Self::SmallTalk,
            _ => Self::Other,
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Pricing => "💰",
            Self::Security => "🔒",
            Self::Timeline => "⏱️",
            Self::Competition => "⚔️",
            Self::Technical => "🔧",
            Self::BuyingSignal => "🎯",
            Self::Objection => "⚠️",
            Self::Stalling => "⏸️",
            Self::Procurement => "📋",
            Self::SmallTalk => "👋",
            Self::Other => "❔",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Pricing => "Pricing Question",
            Self::Security => "Security/Compliance",
            Self::Timeline => "Timeline Question",
            Self::Competition => "Competitor Comparison",
            Self::Technical => "Technical Question",
            Self::BuyingSignal => "Buying Signal!",
            Self::Objection => "Objection",
            Self::Stalling => "Stalling/Delay",
            Self::Procurement => "Procurement Process",
            Self::SmallTalk => "Small Talk",
            Self::Other => "General",
        }
    }
}

/// Analyzes text to detect intent
pub struct IntentAnalyzer {
    /// Keyword patterns for each intent category
    patterns: Vec<(IntentCategory, Vec<&'static str>)>,
}

impl Default for IntentAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl IntentAnalyzer {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                (IntentCategory::Pricing, vec![
                    "how much", "cost", "price", "pricing", "budget", "expensive",
                    "afford", "discount", "payment", "subscription", "per user",
                    "per seat", "annual", "monthly", "fee", "charge",
                ]),
                (IntentCategory::Security, vec![
                    "security", "secure", "soc2", "soc 2", "gdpr", "hipaa", "compliance",
                    "compliant", "data protection", "encryption", "privacy", "audit",
                    "penetration test", "vulnerability", "certification",
                ]),
                (IntentCategory::Timeline, vec![
                    "how long", "timeline", "when can", "how soon", "implementation",
                    "onboarding", "setup time", "go live", "deploy", "migrate",
                    "transition", "deadline", "by when", "urgent",
                ]),
                (IntentCategory::Competition, vec![
                    "compared to", "vs", "versus", "competitor", "alternative",
                    "different from", "better than", "why not use", "already using",
                    "switch from", "salesforce", "hubspot", "zendesk", // Add common competitors
                ]),
                (IntentCategory::Technical, vec![
                    "integrate", "integration", "api", "sdk", "webhook", "technical",
                    "architecture", "scalability", "performance", "uptime", "sla",
                    "latency", "database", "infrastructure", "stack",
                ]),
                (IntentCategory::BuyingSignal, vec![
                    "next steps", "how do we start", "get started", "sign up",
                    "contract", "agreement", "pilot", "trial", "proof of concept",
                    "let's do it", "sounds good", "i'm interested", "move forward",
                    "ready to", "when can we",
                ]),
                (IntentCategory::Objection, vec![
                    "too expensive", "not sure", "concern", "worried", "hesitant",
                    "don't think", "not convinced", "problem with", "issue with",
                    "can't", "won't work", "doesn't fit", "not ready",
                ]),
                (IntentCategory::Stalling, vec![
                    "think about it", "get back to you", "send me info", "email me",
                    "send a proposal", "need to discuss", "talk to my team",
                    "check internally", "not the right time", "maybe later",
                    "circle back", "follow up",
                ]),
                (IntentCategory::Procurement, vec![
                    "who else", "decision maker", "sign off", "approval", "procurement",
                    "purchasing", "legal review", "it review", "security review",
                    "vendor", "rfp", "rfi", "evaluation", "committee",
                ]),
            ],
        }
    }

    /// Analyze text and detect intent
    pub fn analyze(&self, text: &str) -> DetectedIntent {
        let text_lower = text.to_lowercase();

        let mut best_match: Option<(IntentCategory, f32, Vec<String>)> = None;

        for (category, keywords) in &self.patterns {
            let mut matched_keywords = Vec::new();

            for keyword in keywords {
                if text_lower.contains(keyword) {
                    matched_keywords.push(keyword.to_string());
                }
            }

            if !matched_keywords.is_empty() {
                // Score based on number of matches and keyword specificity
                let score = matched_keywords.len() as f32 / keywords.len() as f32;

                if best_match.is_none() || score > best_match.as_ref().unwrap().1 {
                    best_match = Some((category.clone(), score, matched_keywords));
                }
            }
        }

        match best_match {
            Some((category, confidence, triggers)) => DetectedIntent {
                needs_response: !matches!(category, IntentCategory::SmallTalk),
                category,
                confidence: confidence.min(0.95), // Cap confidence
                triggers,
            },
            None => DetectedIntent {
                category: IntentCategory::Other,
                confidence: 0.0,
                needs_response: true,
                triggers: vec![],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_detection() {
        let analyzer = IntentAnalyzer::new();

        let pricing = analyzer.analyze("How much does the enterprise plan cost?");
        assert_eq!(pricing.category, IntentCategory::Pricing);

        let security = analyzer.analyze("Are you SOC2 compliant?");
        assert_eq!(security.category, IntentCategory::Security);

        let buying = analyzer.analyze("What are the next steps to get started?");
        assert_eq!(buying.category, IntentCategory::BuyingSignal);

        let stalling = analyzer.analyze("Let me think about it and get back to you");
        assert_eq!(stalling.category, IntentCategory::Stalling);
    }
}
