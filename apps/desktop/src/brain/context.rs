//! Conversation Context
//!
//! Tracks the ongoing conversation for better AI responses.

use chrono::{DateTime, Utc};
use std::collections::VecDeque;

/// A single turn in the conversation
#[derive(Debug, Clone)]
pub struct ConversationTurn {
    /// Who spoke ("them" or "me")
    pub speaker: Speaker,
    /// What was said
    pub text: String,
    /// When it was said
    pub timestamp: DateTime<Utc>,
    /// Detected intent (if available)
    pub intent: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Speaker {
    Them, // The other person
    Me,   // The user
}

impl Speaker {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Them => "Them",
            Self::Me => "Me",
        }
    }
}

/// Manages conversation history and context
#[derive(Debug)]
pub struct ConversationContext {
    /// Conversation turns
    turns: VecDeque<ConversationTurn>,
    /// Maximum turns to keep
    max_turns: usize,
    /// Current mode/context description
    mode_context: String,
    /// Key facts extracted from conversation
    key_facts: Vec<String>,
    /// Objections that have been raised
    objections_raised: Vec<String>,
}

impl Default for ConversationContext {
    fn default() -> Self {
        Self::new(20)
    }
}

impl ConversationContext {
    /// Create a new conversation context
    pub fn new(max_turns: usize) -> Self {
        Self {
            turns: VecDeque::with_capacity(max_turns),
            max_turns,
            mode_context: String::new(),
            key_facts: Vec::new(),
            objections_raised: Vec::new(),
        }
    }

    /// Set the mode context (sales, interview, technical)
    pub fn set_mode_context(&mut self, context: impl Into<String>) {
        self.mode_context = context.into();
    }

    /// Add a turn from the other person
    pub fn add_their_turn(&mut self, text: impl Into<String>, intent: Option<String>) {
        self.add_turn(ConversationTurn {
            speaker: Speaker::Them,
            text: text.into(),
            timestamp: Utc::now(),
            intent,
        });
    }

    /// Add a turn from the user
    pub fn add_my_turn(&mut self, text: impl Into<String>) {
        self.add_turn(ConversationTurn {
            speaker: Speaker::Me,
            text: text.into(),
            timestamp: Utc::now(),
            intent: None,
        });
    }

    /// Add a turn
    fn add_turn(&mut self, turn: ConversationTurn) {
        self.turns.push_back(turn);
        while self.turns.len() > self.max_turns {
            self.turns.pop_front();
        }
    }

    /// Record an objection
    pub fn record_objection(&mut self, objection: impl Into<String>) {
        self.objections_raised.push(objection.into());
    }

    /// Add a key fact
    pub fn add_key_fact(&mut self, fact: impl Into<String>) {
        self.key_facts.push(fact.into());
    }

    /// Get conversation history as a string for prompts
    pub fn get_history_string(&self) -> String {
        self.turns
            .iter()
            .map(|turn| format!("{}: {}", turn.speaker.label(), turn.text))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get recent history (last N turns)
    pub fn get_recent_history(&self, n: usize) -> String {
        self.turns
            .iter()
            .rev()
            .take(n)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|turn| format!("{}: {}", turn.speaker.label(), turn.text))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get the full context string for prompts
    pub fn get_full_context(&self) -> String {
        let mut context = self.mode_context.clone();

        if !self.key_facts.is_empty() {
            context.push_str("\n\nKey facts established:");
            for fact in &self.key_facts {
                context.push_str(&format!("\n- {}", fact));
            }
        }

        if !self.objections_raised.is_empty() {
            context.push_str("\n\nObjections raised so far:");
            for obj in &self.objections_raised {
                context.push_str(&format!("\n- {}", obj));
            }
        }

        context
    }

    /// Get the last thing they said
    pub fn get_last_their_turn(&self) -> Option<&ConversationTurn> {
        self.turns.iter().rev().find(|t| t.speaker == Speaker::Them)
    }

    /// Clear the conversation
    pub fn clear(&mut self) {
        self.turns.clear();
        self.key_facts.clear();
        self.objections_raised.clear();
    }

    /// Get turn count
    pub fn turn_count(&self) -> usize {
        self.turns.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_context() {
        let mut ctx = ConversationContext::new(5);
        ctx.set_mode_context("Sales call for SaaS product");

        ctx.add_their_turn("How much does it cost?", Some("pricing".to_string()));
        ctx.add_my_turn("It depends on your needs. What's your team size?");
        ctx.add_their_turn("About 50 people", None);

        assert_eq!(ctx.turn_count(), 3);

        let history = ctx.get_history_string();
        assert!(history.contains("How much does it cost?"));
        assert!(history.contains("About 50 people"));
    }
}
