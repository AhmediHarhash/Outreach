//! Prompt Editor
//!
//! Utilities for editing and validating prompts.

use std::collections::HashSet;
use regex::Regex;

/// Prompt editor utilities
pub struct PromptEditor;

impl PromptEditor {
    /// Extract variables from a prompt template
    pub fn extract_variables(template: &str) -> Vec<String> {
        let re = Regex::new(r"\{\{(\w+)\}\}").unwrap();
        let mut variables: HashSet<String> = HashSet::new();

        for cap in re.captures_iter(template) {
            if let Some(var) = cap.get(1) {
                variables.insert(var.as_str().to_string());
            }
        }

        variables.into_iter().collect()
    }

    /// Validate a prompt template
    pub fn validate(template: &str) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check for empty template
        if template.trim().is_empty() {
            errors.push("Template cannot be empty".to_string());
            return ValidationResult { is_valid: false, errors, warnings };
        }

        // Check for unclosed braces
        let open_count = template.matches("{{").count();
        let close_count = template.matches("}}").count();
        if open_count != close_count {
            errors.push(format!(
                "Mismatched braces: {} opening, {} closing",
                open_count, close_count
            ));
        }

        // Check for common variables
        let variables = Self::extract_variables(template);
        let common_vars = ["transcript", "context", "history", "bullets"];

        let has_transcript = variables.iter().any(|v| v == "transcript");
        if !has_transcript {
            warnings.push("Template doesn't use {{transcript}} variable - won't include user speech".to_string());
        }

        // Check for very short prompts
        if template.len() < 50 {
            warnings.push("Template is very short - consider adding more context".to_string());
        }

        // Check for very long prompts
        if template.len() > 4000 {
            warnings.push("Template is very long - may affect response latency".to_string());
        }

        // Check for JSON output instruction (for flash prompts)
        if template.to_lowercase().contains("json") {
            if !template.contains("{") || !template.contains("}") {
                warnings.push("JSON mentioned but no example JSON structure provided".to_string());
            }
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// Preview a prompt with sample data
    pub fn preview(template: &str) -> String {
        let mut result = template.to_string();

        // Replace with sample data
        result = result.replace("{{transcript}}", "[What they said...]");
        result = result.replace("{{context}}", "[Sales call for SaaS product]");
        result = result.replace("{{history}}", "[Previous conversation turns...]");
        result = result.replace("{{bullets}}", "- Key point 1\n- Key point 2");

        result
    }

    /// Format a prompt for display
    pub fn format(template: &str) -> String {
        // Highlight variables
        let re = Regex::new(r"\{\{(\w+)\}\}").unwrap();
        re.replace_all(template, "【$1】").to_string()
    }

    /// Count tokens (approximate)
    pub fn estimate_tokens(text: &str) -> usize {
        // Rough estimate: ~4 characters per token for English
        text.len() / 4
    }
}

/// Prompt variable metadata
#[derive(Debug, Clone)]
pub struct PromptVariable {
    pub name: String,
    pub description: String,
    pub example: String,
    pub required: bool,
}

impl PromptVariable {
    /// Get standard variables
    pub fn standard_variables() -> Vec<Self> {
        vec![
            Self {
                name: "transcript".to_string(),
                description: "What the other person just said".to_string(),
                example: "How much does your enterprise plan cost?".to_string(),
                required: true,
            },
            Self {
                name: "context".to_string(),
                description: "Current conversation context/mode".to_string(),
                example: "Sales call for SaaS product".to_string(),
                required: true,
            },
            Self {
                name: "history".to_string(),
                description: "Previous conversation turns".to_string(),
                example: "You: Hello...\nThem: Hi, I'm interested in...".to_string(),
                required: false,
            },
            Self {
                name: "bullets".to_string(),
                description: "Quick bullet points from flash analysis".to_string(),
                example: "- Pricing question\n- Enterprise tier".to_string(),
                required: false,
            },
            Self {
                name: "mode".to_string(),
                description: "Current mode (sales/interview/technical)".to_string(),
                example: "sales".to_string(),
                required: false,
            },
        ]
    }
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_variables() {
        let template = "Hello {{name}}, your {{item}} is ready.";
        let vars = PromptEditor::extract_variables(template);
        assert!(vars.contains(&"name".to_string()));
        assert!(vars.contains(&"item".to_string()));
    }

    #[test]
    fn test_validate_valid() {
        let template = r#"Analyze this: "{{transcript}}"
Context: {{context}}
Respond in JSON."#;
        let result = PromptEditor::validate(template);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_mismatched_braces() {
        let template = "Hello {{name}, your item is ready.";
        let result = PromptEditor::validate(template);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("Mismatched")));
    }

    #[test]
    fn test_validate_empty() {
        let result = PromptEditor::validate("");
        assert!(!result.is_valid);
    }

    #[test]
    fn test_estimate_tokens() {
        let text = "This is a test sentence with about forty characters.";
        let tokens = PromptEditor::estimate_tokens(text);
        assert!(tokens > 5 && tokens < 20);
    }
}
