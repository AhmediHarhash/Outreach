//! Sales Mode
//!
//! Optimized for sales calls - focuses on:
//! - Objection handling
//! - Value proposition
//! - Closing techniques
//! - Qualification questions

use super::CopilotMode;

pub struct SalesMode {
    /// Product/service being sold
    pub product: String,
    /// Key value propositions
    pub value_props: Vec<String>,
    /// Common objections and responses
    pub objection_handlers: Vec<(String, String)>,
    /// Pricing information
    pub pricing_info: Option<String>,
}

impl Default for SalesMode {
    fn default() -> Self {
        Self {
            product: "your product/service".to_string(),
            value_props: vec![
                "Save time and increase efficiency".to_string(),
                "Reduce costs".to_string(),
                "Better customer experience".to_string(),
            ],
            objection_handlers: vec![
                ("too expensive".to_string(), "Focus on ROI and value, not cost".to_string()),
                ("need to think about it".to_string(), "Understand what specific concerns they have".to_string()),
                ("already have a solution".to_string(), "Ask what's missing from their current solution".to_string()),
            ],
            pricing_info: None,
        }
    }
}

impl CopilotMode for SalesMode {
    fn name(&self) -> &'static str {
        "Sales Call"
    }

    fn context_description(&self) -> String {
        format!(
            r#"This is a sales call for {}.

Key value propositions:
{}

The goal is to:
1. Understand their needs and pain points
2. Present relevant value propositions
3. Handle objections professionally
4. Move toward a clear next step (demo, trial, contract)

Be confident but not pushy. Focus on their problems, not your features."#,
            self.product,
            self.value_props.iter().map(|v| format!("- {}", v)).collect::<Vec<_>>().join("\n")
        )
    }

    fn prompt_additions(&self) -> String {
        let mut additions = String::new();

        if let Some(pricing) = &self.pricing_info {
            additions.push_str(&format!("\nPricing info: {}\n", pricing));
        }

        additions.push_str("\nSales-specific guidance:");
        additions.push_str("\n- Always tie features to business outcomes");
        additions.push_str("\n- Ask qualifying questions (budget, timeline, decision makers)");
        additions.push_str("\n- When facing objections, acknowledge first, then reframe");
        additions.push_str("\n- End responses with a question to keep control");

        additions
    }

    fn customize_bullets(&self, bullets: &mut Vec<String>) {
        // Ensure there's always a qualifying question
        let has_question = bullets.iter().any(|b| b.contains('?'));
        if !has_question && !bullets.is_empty() {
            bullets.push("What's driving your evaluation right now?".to_string());
        }
    }
}
