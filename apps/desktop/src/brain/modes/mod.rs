//! Context Modes
//!
//! Different AI behaviors based on the type of conversation.
//! Supports 13+ modes for different professional scenarios.

mod sales;
mod interview;
mod technical;

pub use sales::SalesMode;
pub use interview::InterviewMode;
pub use technical::TechnicalMode;

use serde::{Deserialize, Serialize};

/// Common trait for all modes
pub trait CopilotMode {
    /// Get the mode name
    fn name(&self) -> &'static str;

    /// Get the context description for prompts
    fn context_description(&self) -> String;

    /// Get mode-specific system prompt additions
    fn prompt_additions(&self) -> String;

    /// Customize bullet extraction for this mode
    fn customize_bullets(&self, bullets: &mut Vec<String>);
}

/// All available conversation modes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConversationMode {
    // Core modes
    Sales,
    Interview,
    Technical,

    // Additional modes
    Negotiation,
    CustomerSupport,
    Presentation,
    Meeting,
    Networking,
    Coaching,
    Medical,
    Legal,
    RealEstate,
    Recruitment,
    Custom(String),
}

impl ConversationMode {
    /// Get all available modes
    pub fn all() -> Vec<Self> {
        vec![
            Self::Sales,
            Self::Interview,
            Self::Technical,
            Self::Negotiation,
            Self::CustomerSupport,
            Self::Presentation,
            Self::Meeting,
            Self::Networking,
            Self::Coaching,
            Self::Medical,
            Self::Legal,
            Self::RealEstate,
            Self::Recruitment,
        ]
    }

    /// Get mode from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "sales" => Self::Sales,
            "interview" => Self::Interview,
            "technical" | "tech" => Self::Technical,
            "negotiation" | "negotiate" => Self::Negotiation,
            "support" | "customer_support" | "customer support" => Self::CustomerSupport,
            "presentation" | "present" => Self::Presentation,
            "meeting" => Self::Meeting,
            "networking" | "network" => Self::Networking,
            "coaching" | "coach" => Self::Coaching,
            "medical" | "health" => Self::Medical,
            "legal" | "law" => Self::Legal,
            "real_estate" | "real estate" | "realestate" => Self::RealEstate,
            "recruitment" | "recruiting" | "hr" => Self::Recruitment,
            other => Self::Custom(other.to_string()),
        }
    }

    /// Get display name
    pub fn name(&self) -> &str {
        match self {
            Self::Sales => "Sales",
            Self::Interview => "Interview",
            Self::Technical => "Technical",
            Self::Negotiation => "Negotiation",
            Self::CustomerSupport => "Support",
            Self::Presentation => "Presentation",
            Self::Meeting => "Meeting",
            Self::Networking => "Networking",
            Self::Coaching => "Coaching",
            Self::Medical => "Medical",
            Self::Legal => "Legal",
            Self::RealEstate => "Real Estate",
            Self::Recruitment => "Recruitment",
            Self::Custom(name) => name,
        }
    }

    /// Get emoji for mode
    pub fn emoji(&self) -> &str {
        match self {
            Self::Sales => "💰",
            Self::Interview => "👔",
            Self::Technical => "🔧",
            Self::Negotiation => "🤝",
            Self::CustomerSupport => "🎧",
            Self::Presentation => "📊",
            Self::Meeting => "📅",
            Self::Networking => "🌐",
            Self::Coaching => "🎯",
            Self::Medical => "⚕️",
            Self::Legal => "⚖️",
            Self::RealEstate => "🏠",
            Self::Recruitment => "👥",
            Self::Custom(_) => "⚙️",
        }
    }

    /// Get description
    pub fn description(&self) -> &str {
        match self {
            Self::Sales => "Close deals, handle objections, build value",
            Self::Interview => "Answer questions, showcase skills, ask smart questions",
            Self::Technical => "Debug issues, discuss architecture, explain concepts",
            Self::Negotiation => "Find win-win, manage concessions, close agreements",
            Self::CustomerSupport => "Resolve issues, maintain satisfaction, de-escalate",
            Self::Presentation => "Engage audience, handle Q&A, stay on track",
            Self::Meeting => "Stay productive, track action items, manage time",
            Self::Networking => "Build rapport, exchange value, follow up",
            Self::Coaching => "Ask powerful questions, provide feedback, motivate",
            Self::Medical => "Gather symptoms, explain clearly, ensure understanding",
            Self::Legal => "Clarify terms, document carefully, protect interests",
            Self::RealEstate => "Showcase properties, handle concerns, close deals",
            Self::Recruitment => "Assess candidates, sell opportunity, qualify fit",
            Self::Custom(_) => "Custom conversation mode",
        }
    }

    /// Get context prompt for this mode
    pub fn context_prompt(&self) -> String {
        match self {
            Self::Sales => "Sales call - focus on value, handle objections, close deals".to_string(),
            Self::Interview => "Job interview - use STAR method, show enthusiasm, ask questions".to_string(),
            Self::Technical => "Technical discussion - be precise, consider trade-offs, explain clearly".to_string(),
            Self::Negotiation => "Negotiation - find win-win, protect interests, build rapport".to_string(),
            Self::CustomerSupport => "Customer support - empathize, resolve issues, ensure satisfaction".to_string(),
            Self::Presentation => "Presentation - engage audience, handle Q&A, strong CTA".to_string(),
            Self::Meeting => "Meeting - stay on track, capture action items, manage time".to_string(),
            Self::Networking => "Networking - build rapport, find value, memorable connection".to_string(),
            Self::Coaching => "Coaching - ask questions, provide feedback, motivate action".to_string(),
            Self::Medical => "Medical consultation - gather info, explain clearly, ensure understanding".to_string(),
            Self::Legal => "Legal discussion - precise terms, document carefully, protect interests".to_string(),
            Self::RealEstate => "Real estate - match needs, handle objections, create urgency".to_string(),
            Self::Recruitment => "Recruitment - assess fit, sell opportunity, move forward".to_string(),
            Self::Custom(name) => format!("{} conversation", name),
        }
    }
}

impl Default for ConversationMode {
    fn default() -> Self {
        Self::Sales
    }
}

impl std::fmt::Display for ConversationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
