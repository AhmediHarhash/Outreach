//! Technical Mode
//!
//! Optimized for technical discussions - focuses on:
//! - Accurate technical explanations
//! - Trade-off analysis
//! - Architecture discussions
//! - Code/system design

use super::CopilotMode;

pub struct TechnicalMode {
    /// Technical domain
    pub domain: String,
    /// Technologies/stack being discussed
    pub technologies: Vec<String>,
    /// Level of technical depth
    pub depth: TechnicalDepth,
}

#[derive(Debug, Clone, Default)]
pub enum TechnicalDepth {
    /// High-level overview
    Overview,
    #[default]
    /// Standard technical discussion
    Standard,
    /// Deep dive with implementation details
    Deep,
}

impl Default for TechnicalMode {
    fn default() -> Self {
        Self {
            domain: "software engineering".to_string(),
            technologies: vec![],
            depth: TechnicalDepth::Standard,
        }
    }
}

impl CopilotMode for TechnicalMode {
    fn name(&self) -> &'static str {
        "Technical Discussion"
    }

    fn context_description(&self) -> String {
        let depth_desc = match self.depth {
            TechnicalDepth::Overview => "Keep explanations high-level and accessible.",
            TechnicalDepth::Standard => "Balance clarity with technical accuracy.",
            TechnicalDepth::Deep => "Go deep into implementation details when relevant.",
        };

        format!(
            r#"This is a technical discussion about {}.

{}

The goal is to:
1. Explain concepts clearly and accurately
2. Discuss trade-offs when relevant
3. Provide concrete examples
4. Acknowledge uncertainty when appropriate

{}"#,
            self.domain,
            if !self.technologies.is_empty() {
                format!("Technologies involved: {}", self.technologies.join(", "))
            } else {
                String::new()
            },
            depth_desc
        )
    }

    fn prompt_additions(&self) -> String {
        let mut additions = String::new();

        additions.push_str("\n\nTechnical discussion guidance:");
        additions.push_str("\n- Lead with the 'what' before the 'how'");
        additions.push_str("\n- Use analogies for complex concepts");
        additions.push_str("\n- Mention trade-offs (pros/cons) proactively");
        additions.push_str("\n- Say 'I don't know' rather than guessing");
        additions.push_str("\n- Reference industry standards/best practices");

        match self.depth {
            TechnicalDepth::Overview => {
                additions.push_str("\n- Keep it high-level, avoid jargon");
                additions.push_str("\n- Focus on business impact, not implementation");
            }
            TechnicalDepth::Standard => {
                additions.push_str("\n- Balance technical accuracy with clarity");
                additions.push_str("\n- Include one concrete example per concept");
            }
            TechnicalDepth::Deep => {
                additions.push_str("\n- Include implementation details");
                additions.push_str("\n- Discuss edge cases and failure modes");
                additions.push_str("\n- Reference specific algorithms/patterns");
            }
        }

        additions
    }

    fn customize_bullets(&self, bullets: &mut Vec<String>) {
        // Ensure technical responses include trade-offs
        let has_tradeoff = bullets.iter().any(|b| {
            b.contains("trade") || b.contains("but") || b.contains("however") || b.contains("downside")
        });

        if !has_tradeoff && bullets.len() > 1 {
            bullets.push("Consider the trade-offs...".to_string());
        }
    }
}
