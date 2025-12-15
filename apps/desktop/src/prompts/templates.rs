//! Prompt Templates
//!
//! Pre-built prompt templates for common scenarios.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A prompt template with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Category (sales, interview, technical, custom)
    pub category: PromptCategory,
    /// The prompt template text
    pub template: String,
    /// Available variables
    pub variables: Vec<String>,
    /// Is this a built-in template
    pub is_builtin: bool,
}

/// Prompt categories
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PromptCategory {
    Sales,
    Interview,
    Technical,
    Custom,
}

impl PromptCategory {
    pub fn label(&self) -> &'static str {
        match self {
            PromptCategory::Sales => "Sales",
            PromptCategory::Interview => "Interview",
            PromptCategory::Technical => "Technical",
            PromptCategory::Custom => "Custom",
        }
    }

    pub fn all() -> Vec<PromptCategory> {
        vec![
            PromptCategory::Sales,
            PromptCategory::Interview,
            PromptCategory::Technical,
            PromptCategory::Custom,
        ]
    }
}

/// Library of prompt templates
pub struct PromptLibrary {
    templates: HashMap<String, PromptTemplate>,
}

impl PromptLibrary {
    pub fn new() -> Self {
        let mut library = Self {
            templates: HashMap::new(),
        };
        library.load_builtins();
        library
    }

    fn load_builtins(&mut self) {
        // Sales Templates
        self.add(PromptTemplate {
            id: "sales_discovery".to_string(),
            name: "Discovery Call".to_string(),
            description: "Help navigate discovery calls to uncover prospect needs".to_string(),
            category: PromptCategory::Sales,
            template: r#"You're assisting in a discovery call. Help identify pain points and needs.

Context: {{context}}
They said: "{{transcript}}"

Guide the conversation toward:
- Understanding their current challenges
- Identifying decision-making process
- Uncovering budget and timeline
- Finding the key stakeholders"#.to_string(),
            variables: vec!["context".to_string(), "transcript".to_string()],
            is_builtin: true,
        });

        self.add(PromptTemplate {
            id: "sales_objection".to_string(),
            name: "Objection Handling".to_string(),
            description: "Help handle common sales objections effectively".to_string(),
            category: PromptCategory::Sales,
            template: r#"You're helping handle a sales objection.

Context: {{context}}
Their objection: "{{transcript}}"

Use the LAER framework:
- Listen: Acknowledge their concern
- Acknowledge: Show understanding
- Explore: Ask clarifying questions
- Respond: Address with value, not features"#.to_string(),
            variables: vec!["context".to_string(), "transcript".to_string()],
            is_builtin: true,
        });

        self.add(PromptTemplate {
            id: "sales_closing".to_string(),
            name: "Closing Techniques".to_string(),
            description: "Help close deals with confidence".to_string(),
            category: PromptCategory::Sales,
            template: r#"You're assisting with closing a deal.

Context: {{context}}
History: {{history}}
They said: "{{transcript}}"

Suggest closing approaches:
- Assumptive close if signals are positive
- Summary close to recap value
- Alternative close with options
- Trial close to test readiness"#.to_string(),
            variables: vec!["context".to_string(), "history".to_string(), "transcript".to_string()],
            is_builtin: true,
        });

        // Interview Templates
        self.add(PromptTemplate {
            id: "interview_behavioral".to_string(),
            name: "Behavioral Questions".to_string(),
            description: "Help answer behavioral interview questions using STAR".to_string(),
            category: PromptCategory::Interview,
            template: r#"Help answer this behavioral question using STAR method.

Question: "{{transcript}}"

Structure the response:
- Situation: Set the context briefly
- Task: Explain your responsibility
- Action: Detail specific steps YOU took
- Result: Quantify the outcome

Keep it under 2 minutes when spoken."#.to_string(),
            variables: vec!["transcript".to_string()],
            is_builtin: true,
        });

        self.add(PromptTemplate {
            id: "interview_technical".to_string(),
            name: "Technical Questions".to_string(),
            description: "Help answer technical interview questions clearly".to_string(),
            category: PromptCategory::Interview,
            template: r#"Help answer this technical question.

Context: {{context}}
Question: "{{transcript}}"

Structure the response:
1. Restate to confirm understanding
2. Think aloud through your approach
3. Provide the core answer
4. Discuss trade-offs or alternatives
5. Mention relevant experience"#.to_string(),
            variables: vec!["context".to_string(), "transcript".to_string()],
            is_builtin: true,
        });

        self.add(PromptTemplate {
            id: "interview_questions".to_string(),
            name: "Questions to Ask".to_string(),
            description: "Suggest intelligent questions to ask the interviewer".to_string(),
            category: PromptCategory::Interview,
            template: r#"Suggest questions to ask the interviewer.

Role: {{context}}
Discussion so far: {{history}}

Suggest questions that:
- Show genuine interest in the role
- Demonstrate research about the company
- Explore team dynamics and growth
- Clarify expectations and success metrics"#.to_string(),
            variables: vec!["context".to_string(), "history".to_string()],
            is_builtin: true,
        });

        // Technical Templates
        self.add(PromptTemplate {
            id: "tech_architecture".to_string(),
            name: "Architecture Discussion".to_string(),
            description: "Help discuss system architecture and design".to_string(),
            category: PromptCategory::Technical,
            template: r#"You're assisting in an architecture discussion.

Context: {{context}}
Topic: "{{transcript}}"

Consider:
- Scalability and performance
- Reliability and fault tolerance
- Security implications
- Cost and operational complexity
- Trade-offs between options"#.to_string(),
            variables: vec!["context".to_string(), "transcript".to_string()],
            is_builtin: true,
        });

        self.add(PromptTemplate {
            id: "tech_debugging".to_string(),
            name: "Debugging Session".to_string(),
            description: "Help during debugging and troubleshooting".to_string(),
            category: PromptCategory::Technical,
            template: r#"You're assisting in a debugging session.

Problem description: "{{transcript}}"
Context: {{context}}

Help by:
- Identifying potential root causes
- Suggesting diagnostic steps
- Proposing quick fixes vs proper solutions
- Recommending preventive measures"#.to_string(),
            variables: vec!["transcript".to_string(), "context".to_string()],
            is_builtin: true,
        });

        self.add(PromptTemplate {
            id: "tech_code_review".to_string(),
            name: "Code Review".to_string(),
            description: "Help provide constructive code review feedback".to_string(),
            category: PromptCategory::Technical,
            template: r#"You're assisting in a code review discussion.

Discussion: "{{transcript}}"
Context: {{context}}

Provide feedback on:
- Code clarity and maintainability
- Performance considerations
- Security implications
- Testing coverage
- Best practices and patterns

Be constructive and specific."#.to_string(),
            variables: vec!["transcript".to_string(), "context".to_string()],
            is_builtin: true,
        });
    }

    /// Add a template
    pub fn add(&mut self, template: PromptTemplate) {
        self.templates.insert(template.id.clone(), template);
    }

    /// Get a template by ID
    pub fn get(&self, id: &str) -> Option<&PromptTemplate> {
        self.templates.get(id)
    }

    /// List templates by category
    pub fn by_category(&self, category: PromptCategory) -> Vec<&PromptTemplate> {
        self.templates
            .values()
            .filter(|t| t.category == category)
            .collect()
    }

    /// List all templates
    pub fn all(&self) -> Vec<&PromptTemplate> {
        self.templates.values().collect()
    }

    /// Search templates
    pub fn search(&self, query: &str) -> Vec<&PromptTemplate> {
        let query_lower = query.to_lowercase();
        self.templates
            .values()
            .filter(|t| {
                t.name.to_lowercase().contains(&query_lower) ||
                t.description.to_lowercase().contains(&query_lower)
            })
            .collect()
    }
}

impl Default for PromptLibrary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_builtins() {
        let library = PromptLibrary::new();
        assert!(!library.templates.is_empty());

        let sales = library.by_category(PromptCategory::Sales);
        assert!(!sales.is_empty());
    }

    #[test]
    fn test_search() {
        let library = PromptLibrary::new();
        let results = library.search("objection");
        assert!(results.iter().any(|t| t.id == "sales_objection"));
    }
}
