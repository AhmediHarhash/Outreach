//! Lead model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Lead database model
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Lead {
    pub id: Uuid,
    pub user_id: Uuid,

    // Company
    pub company_name: String,
    pub company_domain: Option<String>,
    pub company_linkedin: Option<String>,
    pub company_size: Option<String>,
    pub industry: Option<String>,
    pub location: Option<String>,

    // Contact
    pub contact_name: Option<String>,
    pub contact_title: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_linkedin: Option<String>,

    // Status
    pub status: String,
    pub priority: i32,
    pub estimated_value: Option<sqlx::types::BigDecimal>,

    // Enrichment (JSONB)
    pub tech_stack: Option<serde_json::Value>,
    pub funding_info: Option<serde_json::Value>,
    pub recent_news: Option<serde_json::Value>,
    pub employee_count: Option<i32>,

    // Metadata
    pub source: Option<String>,
    pub tags: Option<Vec<String>>,
    pub notes: Option<String>,
    pub custom_fields: Option<serde_json::Value>,

    // Timeline
    pub last_contacted_at: Option<DateTime<Utc>>,
    pub next_followup_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create lead request
#[derive(Debug, Deserialize, Validate)]
pub struct CreateLeadRequest {
    #[validate(length(min = 1, max = 255))]
    pub company_name: String,

    pub company_domain: Option<String>,
    pub company_linkedin: Option<String>,
    pub company_size: Option<String>,
    pub industry: Option<String>,
    pub location: Option<String>,

    pub contact_name: Option<String>,
    pub contact_title: Option<String>,
    #[validate(email)]
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_linkedin: Option<String>,

    pub status: Option<String>,
    pub priority: Option<i32>,
    pub estimated_value: Option<f64>,

    pub source: Option<String>,
    pub tags: Option<Vec<String>>,
    pub notes: Option<String>,

    pub next_followup_at: Option<DateTime<Utc>>,
}

/// Update lead request
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateLeadRequest {
    #[validate(length(min = 1, max = 255))]
    pub company_name: Option<String>,

    pub company_domain: Option<String>,
    pub company_linkedin: Option<String>,
    pub company_size: Option<String>,
    pub industry: Option<String>,
    pub location: Option<String>,

    pub contact_name: Option<String>,
    pub contact_title: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_linkedin: Option<String>,

    pub status: Option<String>,
    pub priority: Option<i32>,
    pub estimated_value: Option<f64>,

    pub tags: Option<Vec<String>>,
    pub notes: Option<String>,

    pub last_contacted_at: Option<DateTime<Utc>>,
    pub next_followup_at: Option<DateTime<Utc>>,
}

/// Lead list query parameters
#[derive(Debug, Deserialize)]
pub struct LeadListQuery {
    pub status: Option<String>,
    pub priority: Option<i32>,
    pub search: Option<String>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

/// Lead list response
#[derive(Debug, Serialize)]
pub struct LeadListResponse {
    pub leads: Vec<Lead>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

/// Lead status enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeadStatus {
    New,
    Researching,
    Contacted,
    Qualified,
    Proposal,
    Negotiation,
    Won,
    Lost,
}

impl LeadStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Researching => "researching",
            Self::Contacted => "contacted",
            Self::Qualified => "qualified",
            Self::Proposal => "proposal",
            Self::Negotiation => "negotiation",
            Self::Won => "won",
            Self::Lost => "lost",
        }
    }
}
