//! Gemini 2.0 Flash Integration
//!
//! The fastest smart model available - perfect for instant bullet extraction.
//! Provides responses in ~200-300ms.

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::bullet_extractor::FlashAnalysis;

/// Gemini 2.0 Flash client
pub struct GeminiFlash {
    api_key: String,
    client: Client,
    model: String,
}

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    generation_config: GenerationConfig,
}

#[derive(Debug, Serialize)]
struct Content {
    parts: Vec<Part>,
    role: Option<String>,
}

#[derive(Debug, Serialize)]
struct Part {
    text: String,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    temperature: f32,
    max_output_tokens: u32,
    response_mime_type: String,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: CandidateContent,
}

#[derive(Debug, Deserialize)]
struct CandidateContent {
    parts: Vec<ResponsePart>,
}

#[derive(Debug, Deserialize)]
struct ResponsePart {
    text: String,
}

impl GeminiFlash {
    /// Create a new Gemini Flash client
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client: Client::new(),
            model: "gemini-2.0-flash-exp".to_string(), // Latest experimental Flash
        }
    }

    /// Use a specific model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Analyze transcript and extract quick response bullets
    pub async fn analyze(&self, transcript: &str, context: &str) -> Result<FlashAnalysis> {
        let prompt = format!(
            r#"You are an instant analysis engine. Respond in <200ms.

INPUT: What someone just said in a conversation
CONTEXT: {}

THEIR STATEMENT: "{}"

OUTPUT: JSON only, no explanation

{{
  "summary": "One sentence: what they're asking/saying",
  "bullets": [
    {{"point": "Key thing to mention", "priority": 1}},
    {{"point": "Another point", "priority": 2}},
    {{"point": "Supporting detail", "priority": 3}}
  ],
  "type": "question|objection|statement|buying_signal|technical|small_talk",
  "urgency": "answer_now|can_elaborate|just_listening"
}}

Rules:
- Max 5 bullets
- Priority 1 = say this first (most important)
- Be specific, not generic
- Under 50 tokens total
- Match the context (sales/interview/technical)"#,
            context, transcript
        );

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part { text: prompt }],
                role: Some("user".to_string()),
            }],
            generation_config: GenerationConfig {
                temperature: 0.3, // Lower for more consistent outputs
                max_output_tokens: 200,
                response_mime_type: "application/json".to_string(),
            },
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        let gemini_response: GeminiResponse = response.json().await?;

        // Extract the JSON from the response
        if let Some(candidate) = gemini_response.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                let analysis: FlashAnalysis = serde_json::from_str(&part.text)?;
                return Ok(analysis);
            }
        }

        Err(anyhow::anyhow!("No response from Gemini"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires API key
    async fn test_gemini_analyze() {
        let client = GeminiFlash::new("YOUR_API_KEY");
        let result = client
            .analyze(
                "How much does your enterprise plan cost?",
                "Sales call for SaaS product",
            )
            .await;

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(!analysis.summary.is_empty());
        assert!(!analysis.bullets.is_empty());
    }
}
