//! Prompt Templates Module
//!
//! Manages customizable prompt templates for AI responses.
//! Users can edit prompts for different modes and stages.

mod templates;
mod editor;

pub use templates::{PromptTemplate, PromptLibrary, PromptCategory};
pub use editor::{PromptEditor, PromptVariable};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Custom prompts configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPrompts {
    /// Flash stage prompts by mode
    pub flash: HashMap<String, String>,
    /// Deep stage prompts by mode
    pub deep: HashMap<String, String>,
    /// System prompts
    pub system: HashMap<String, String>,
}

impl Default for CustomPrompts {
    fn default() -> Self {
        let mut flash = HashMap::new();
        let mut deep = HashMap::new();
        let mut system = HashMap::new();

        // Default Flash prompt (for quick analysis)
        flash.insert("sales".to_string(), r#"You are an instant analysis engine for a sales call. Be extremely concise.

INPUT: What the prospect just said
CONTEXT: {{context}}

THEIR STATEMENT: "{{transcript}}"

Respond with ONLY valid JSON:
{
  "summary": "One sentence: what they're asking/saying",
  "bullets": [
    {"point": "Key thing to mention", "priority": 1},
    {"point": "Another point", "priority": 2}
  ],
  "type": "question|objection|statement|buying_signal|technical|small_talk",
  "urgency": "answer_now|can_elaborate|just_listening"
}

Rules:
- Max 4 bullets, priority 1 = most important
- Be specific to their actual words
- Focus on sales outcomes"#.to_string());

        flash.insert("interview".to_string(), r#"You are an instant analysis engine for a job interview. Be extremely concise.

INPUT: What the interviewer just said
CONTEXT: {{context}}

THEIR STATEMENT: "{{transcript}}"

Respond with ONLY valid JSON:
{
  "summary": "One sentence summary",
  "bullets": [
    {"point": "Key thing to address", "priority": 1},
    {"point": "Supporting point", "priority": 2}
  ],
  "type": "question|follow_up|behavioral|technical|small_talk",
  "urgency": "answer_now|can_elaborate|just_listening"
}

Rules:
- Use STAR method hints where applicable
- Be specific and relevant"#.to_string());

        flash.insert("technical".to_string(), r#"You are an instant analysis engine for a technical discussion. Be extremely concise.

INPUT: What they just said
CONTEXT: {{context}}

THEIR STATEMENT: "{{transcript}}"

Respond with ONLY valid JSON:
{
  "summary": "One sentence technical summary",
  "bullets": [
    {"point": "Key technical point", "priority": 1},
    {"point": "Supporting detail", "priority": 2}
  ],
  "type": "question|clarification|suggestion|concern|technical",
  "urgency": "answer_now|can_elaborate|just_listening"
}

Rules:
- Focus on technical accuracy
- Include relevant terminology"#.to_string());

        // Default Deep prompts (for detailed responses)
        deep.insert("sales".to_string(), r#"You are a world-class sales coach providing real-time guidance.

Context: {{context}}
Conversation history:
{{history}}

They just said: "{{transcript}}"

Quick analysis suggested: {{bullets}}

Provide a detailed response that:
1. Addresses their specific concern/question
2. Builds value before discussing price
3. Uses social proof where relevant
4. Ends with a discovery question

Keep it conversational and natural. Max 150 words."#.to_string());

        deep.insert("interview".to_string(), r#"You are an expert interview coach providing real-time guidance.

Context: {{context}}
Conversation history:
{{history}}

Interviewer asked: "{{transcript}}"

Quick analysis: {{bullets}}

Craft a response that:
1. Uses the STAR method where applicable
2. Shows specific, quantified achievements
3. Relates experience to the role
4. Shows enthusiasm and cultural fit

Keep it natural and confident. Max 150 words."#.to_string());

        deep.insert("technical".to_string(), r#"You are a senior technical expert providing real-time guidance.

Context: {{context}}
Conversation history:
{{history}}

They said: "{{transcript}}"

Quick analysis: {{bullets}}

Provide a response that:
1. Is technically accurate and precise
2. Addresses the core question/concern
3. Suggests best practices where relevant
4. Asks clarifying questions if needed

Keep it clear and professional. Max 150 words."#.to_string());

        // System prompts
        system.insert("default".to_string(),
            "You are an AI assistant helping users during voice conversations. Be concise, helpful, and natural.".to_string());

        Self {
            flash,
            deep,
            system,
        }
    }
}

impl CustomPrompts {
    /// Get the prompts file path
    pub fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("voice-copilot")
            .join("prompts.json")
    }

    /// Load prompts from disk
    pub fn load() -> Result<Self> {
        let path = Self::path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    /// Save prompts to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Get flash prompt for mode
    pub fn get_flash(&self, mode: &str) -> &str {
        self.flash.get(mode).map(|s| s.as_str()).unwrap_or_else(|| {
            self.flash.get("sales").map(|s| s.as_str()).unwrap_or("")
        })
    }

    /// Get deep prompt for mode
    pub fn get_deep(&self, mode: &str) -> &str {
        self.deep.get(mode).map(|s| s.as_str()).unwrap_or_else(|| {
            self.deep.get("sales").map(|s| s.as_str()).unwrap_or("")
        })
    }

    /// Set flash prompt for mode
    pub fn set_flash(&mut self, mode: &str, prompt: &str) {
        self.flash.insert(mode.to_string(), prompt.to_string());
    }

    /// Set deep prompt for mode
    pub fn set_deep(&mut self, mode: &str, prompt: &str) {
        self.deep.insert(mode.to_string(), prompt.to_string());
    }

    /// Reset to defaults
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Reset specific mode to default
    pub fn reset_mode(&mut self, mode: &str) {
        let defaults = Self::default();
        if let Some(flash) = defaults.flash.get(mode) {
            self.flash.insert(mode.to_string(), flash.clone());
        }
        if let Some(deep) = defaults.deep.get(mode) {
            self.deep.insert(mode.to_string(), deep.clone());
        }
    }
}

/// Apply variables to a prompt template
pub fn apply_variables(template: &str, variables: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in variables {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_prompts() {
        let prompts = CustomPrompts::default();
        assert!(!prompts.flash.is_empty());
        assert!(!prompts.deep.is_empty());
        assert!(prompts.flash.contains_key("sales"));
    }

    #[test]
    fn test_apply_variables() {
        let template = "Hello {{name}}, your score is {{score}}.";
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("score".to_string(), "100".to_string());

        let result = apply_variables(template, &vars);
        assert_eq!(result, "Hello Alice, your score is 100.");
    }
}
