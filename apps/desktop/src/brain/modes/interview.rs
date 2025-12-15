//! Interview Mode
//!
//! Optimized for job interviews - focuses on:
//! - STAR method responses
//! - Showcasing relevant experience
//! - Asking smart questions
//! - Professional confidence

use super::CopilotMode;

pub struct InterviewMode {
    /// Role being interviewed for
    pub role: String,
    /// Key skills to highlight
    pub key_skills: Vec<String>,
    /// Past experiences to reference
    pub experiences: Vec<String>,
    /// Questions to ask the interviewer
    pub questions_to_ask: Vec<String>,
}

impl Default for InterviewMode {
    fn default() -> Self {
        Self {
            role: "the position".to_string(),
            key_skills: vec![
                "Problem solving".to_string(),
                "Technical expertise".to_string(),
                "Communication".to_string(),
                "Leadership".to_string(),
            ],
            experiences: vec![],
            questions_to_ask: vec![
                "What does success look like in this role?".to_string(),
                "What are the biggest challenges facing the team?".to_string(),
                "How would you describe the team culture?".to_string(),
            ],
        }
    }
}

impl CopilotMode for InterviewMode {
    fn name(&self) -> &'static str {
        "Job Interview"
    }

    fn context_description(&self) -> String {
        format!(
            r#"This is a job interview for {}.

Key skills to highlight:
{}

The goal is to:
1. Answer questions with specific examples (STAR method)
2. Show enthusiasm for the role and company
3. Demonstrate relevant expertise
4. Ask thoughtful questions

Be confident and professional. Use "we" for team accomplishments, "I" for individual contributions."#,
            self.role,
            self.key_skills.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")
        )
    }

    fn prompt_additions(&self) -> String {
        let mut additions = String::new();

        if !self.experiences.is_empty() {
            additions.push_str("\nRelevant experiences to reference:");
            for exp in &self.experiences {
                additions.push_str(&format!("\n- {}", exp));
            }
        }

        additions.push_str("\n\nInterview-specific guidance:");
        additions.push_str("\n- Use STAR format: Situation, Task, Action, Result");
        additions.push_str("\n- Quantify results when possible (%, $, time saved)");
        additions.push_str("\n- Show growth mindset - talk about what you learned");
        additions.push_str("\n- Be honest about gaps, but frame them as growth areas");

        additions
    }

    fn customize_bullets(&self, bullets: &mut Vec<String>) {
        // Ensure responses are structured
        if bullets.len() > 2 {
            // Reorder to put the strongest point first
            bullets.sort_by(|a, b| {
                // Prioritize concrete examples
                let a_concrete = a.contains("example") || a.contains("when I") || a.contains("at");
                let b_concrete = b.contains("example") || b.contains("when I") || b.contains("at");
                b_concrete.cmp(&a_concrete)
            });
        }
    }
}
