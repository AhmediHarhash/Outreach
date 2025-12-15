//! Hint synthesis using Claude/GPT-4
//!
//! Takes retrieved context and generates actionable hints for the user.

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::search::SearchResult;

/// Generated hints for a call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallHints {
    /// Brief company context
    pub company_brief: String,
    /// Key talking points
    pub talking_points: Vec<String>,
    /// Objection handlers
    pub objection_handlers: Vec<ObjectionHandler>,
    /// Questions to ask
    pub questions_to_ask: Vec<String>,
    /// Warnings or cautions
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectionHandler {
    pub objection: String,
    pub response: String,
}

/// Context for hint generation
#[derive(Debug)]
pub struct HintContext {
    pub lead_name: Option<String>,
    pub lead_industry: Option<String>,
    pub mode: String,
    pub conversation_snippet: String,
    pub retrieved_chunks: Vec<SearchResult>,
}

/// Generate hints using Claude
pub async fn generate_hints(
    context: &HintContext,
    api_key: &str,
) -> Result<CallHints> {
    let client = Client::new();

    // Build context from retrieved chunks
    let retrieved_context: String = context
        .retrieved_chunks
        .iter()
        .map(|c| format!("---\n{}\n", c.content))
        .collect();

    let system_prompt = format!(
        r#"You are an expert {} coach providing real-time assistance during a call.
Your job is to help the user succeed by providing:
1. Brief, actionable talking points
2. Specific objection handlers based on the context
3. Smart questions to ask
4. Any warnings or cautions

Be concise - this will be displayed as a sidebar during a live call.
Use the provided context to make your suggestions specific and relevant."#,
        context.mode
    );

    let user_prompt = format!(
        r#"LEAD: {} ({})

RELEVANT CONTEXT:
{}

CURRENT CONVERSATION:
{}

Generate hints in this JSON format:
{{
    "company_brief": "1-2 sentence company summary",
    "talking_points": ["point 1", "point 2", "point 3"],
    "objection_handlers": [
        {{"objection": "common objection", "response": "how to handle it"}}
    ],
    "questions_to_ask": ["question 1", "question 2"],
    "warnings": ["any cautions or things to avoid"]
}}"#,
        context.lead_name.as_deref().unwrap_or("Unknown"),
        context.lead_industry.as_deref().unwrap_or("Unknown industry"),
        retrieved_context,
        context.conversation_snippet
    );

    // Call Claude API
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "claude-3-5-sonnet-20241022",
            "max_tokens": 1024,
            "system": system_prompt,
            "messages": [
                {"role": "user", "content": user_prompt}
            ]
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Claude API error: {}", error_text);
    }

    let result: serde_json::Value = response.json().await?;

    // Extract content from response
    let content = result["content"][0]["text"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No content in response"))?;

    // Parse JSON from response
    let hints: CallHints = serde_json::from_str(content)
        .map_err(|e| anyhow::anyhow!("Failed to parse hints: {}", e))?;

    Ok(hints)
}

/// Generate quick flash hints (faster, simpler)
pub async fn generate_flash_hints(
    conversation_snippet: &str,
    mode: &str,
    api_key: &str,
) -> Result<Vec<String>> {
    let client = Client::new();

    let prompt = format!(
        r#"You're a {} coach. Based on this conversation snippet, give 3 quick bullet points for what to say/do next.

CONVERSATION:
{}

Respond with exactly 3 short bullet points, no JSON, just text."#,
        mode, conversation_snippet
    );

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "gpt-4o-mini",
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "max_tokens": 200,
            "temperature": 0.7
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("OpenAI API error: {}", error_text);
    }

    let result: serde_json::Value = response.json().await?;
    let content = result["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No content in response"))?;

    // Parse bullet points
    let bullets: Vec<String> = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim_start_matches(&['-', '*', '•', '1', '2', '3', '.', ' '][..]).trim().to_string())
        .filter(|line| !line.is_empty())
        .take(3)
        .collect();

    Ok(bullets)
}
