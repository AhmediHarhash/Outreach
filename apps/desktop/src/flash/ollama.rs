//! Ollama Local LLM Integration
//!
//! Uses locally running Ollama server with Llama 3.1 8B or other models.
//! No API costs, works offline, typically ~500-1000ms response time.

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::bullet_extractor::FlashAnalysis;

/// Default Ollama server URL
const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";

/// Ollama client for local LLM inference
pub struct OllamaFlash {
    base_url: String,
    client: Client,
    model: String,
}

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    format: Option<String>,
    options: OllamaOptions,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: i32,
    top_p: f32,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
    done: bool,
    #[serde(default)]
    total_duration: u64,
    #[serde(default)]
    eval_count: u32,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: u64,
    pub modified_at: String,
}

impl OllamaFlash {
    /// Create a new Ollama client with default settings
    pub fn new() -> Self {
        Self::with_config(DEFAULT_OLLAMA_URL, "llama3.1:8b")
    }

    /// Create with custom URL and model
    pub fn with_config(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            model: model.into(),
        }
    }

    /// Set a different model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Check if Ollama server is running
    pub async fn is_available(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url);
        self.client
            .get(&url)
            .timeout(Duration::from_secs(2))
            .send()
            .await
            .is_ok()
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<OllamaModel>> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self.client.get(&url).send().await?;
        let tags: OllamaTagsResponse = response.json().await?;
        Ok(tags.models)
    }

    /// Check if a specific model is available
    pub async fn has_model(&self, model_name: &str) -> bool {
        if let Ok(models) = self.list_models().await {
            models.iter().any(|m| m.name.contains(model_name))
        } else {
            false
        }
    }

    /// Analyze transcript and extract quick response bullets
    pub async fn analyze(&self, transcript: &str, context: &str) -> Result<FlashAnalysis> {
        let prompt = format!(
            r#"You are an instant analysis engine for a voice assistant. Be extremely concise.

INPUT: What someone just said in a conversation
CONTEXT: {context}

THEIR STATEMENT: "{transcript}"

Respond with ONLY valid JSON, no explanation, no markdown:

{{
  "summary": "One short sentence: what they're asking/saying",
  "bullets": [
    {{"point": "Most important thing to say", "priority": 1}},
    {{"point": "Second point", "priority": 2}},
    {{"point": "Third point if needed", "priority": 3}}
  ],
  "type": "question",
  "urgency": "answer_now"
}}

Rules:
- type must be one of: question, objection, statement, buying_signal, technical, small_talk
- urgency must be one of: answer_now, can_elaborate, just_listening
- Max 4 bullets, keep each under 15 words
- Priority 1 = most important
- Be specific to their actual words
- Output ONLY the JSON, nothing else"#
        );

        let request = OllamaRequest {
            model: self.model.clone(),
            prompt,
            stream: false,
            format: Some("json".to_string()),
            options: OllamaOptions {
                temperature: 0.3,
                num_predict: 300,
                top_p: 0.9,
            },
        };

        let url = format!("{}/api/generate", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Ollama request failed ({}): {}",
                status,
                body
            ));
        }

        let ollama_response: OllamaResponse = response.json().await?;

        // Parse the JSON response
        let analysis: FlashAnalysis = serde_json::from_str(&ollama_response.response)
            .map_err(|e| {
                tracing::warn!(
                    "Failed to parse Ollama response as JSON: {}\nRaw response: {}",
                    e,
                    ollama_response.response
                );
                anyhow::anyhow!("Invalid JSON from Ollama: {}", e)
            })?;

        tracing::debug!(
            "Ollama analysis completed in {}ms, {} tokens",
            ollama_response.total_duration / 1_000_000,
            ollama_response.eval_count
        );

        Ok(analysis)
    }

    /// Simple completion without JSON parsing (for testing)
    pub async fn complete(&self, prompt: &str) -> Result<String> {
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
            format: None,
            options: OllamaOptions {
                temperature: 0.7,
                num_predict: 500,
                top_p: 0.9,
            },
        };

        let url = format!("{}/api/generate", self.base_url);
        let response = self.client.post(&url).json(&request).send().await?;
        let ollama_response: OllamaResponse = response.json().await?;

        Ok(ollama_response.response)
    }
}

impl Default for OllamaFlash {
    fn default() -> Self {
        Self::new()
    }
}

/// Check Ollama server status and available models
pub async fn check_ollama_status() -> OllamaStatus {
    let client = OllamaFlash::new();

    if !client.is_available().await {
        return OllamaStatus::NotRunning;
    }

    match client.list_models().await {
        Ok(models) => {
            if models.is_empty() {
                OllamaStatus::NoModels
            } else {
                // Check for recommended models
                let has_llama = models.iter().any(|m| m.name.contains("llama"));
                let has_mistral = models.iter().any(|m| m.name.contains("mistral"));
                let has_phi = models.iter().any(|m| m.name.contains("phi"));

                OllamaStatus::Ready {
                    models,
                    recommended: if has_llama {
                        Some("llama3.1:8b".to_string())
                    } else if has_mistral {
                        Some("mistral:7b".to_string())
                    } else if has_phi {
                        Some("phi3:mini".to_string())
                    } else {
                        None
                    },
                }
            }
        }
        Err(_) => OllamaStatus::NotRunning,
    }
}

/// Ollama server status
#[derive(Debug, Clone)]
pub enum OllamaStatus {
    /// Server is not running or unreachable
    NotRunning,
    /// Server running but no models installed
    NoModels,
    /// Server ready with available models
    Ready {
        models: Vec<OllamaModel>,
        recommended: Option<String>,
    },
}

impl OllamaStatus {
    pub fn is_ready(&self) -> bool {
        matches!(self, OllamaStatus::Ready { .. })
    }

    pub fn message(&self) -> &'static str {
        match self {
            OllamaStatus::NotRunning => "Ollama not running. Start with: ollama serve",
            OllamaStatus::NoModels => "No models installed. Run: ollama pull llama3.1:8b",
            OllamaStatus::Ready { .. } => "Ollama ready",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Ollama running
    async fn test_ollama_available() {
        let client = OllamaFlash::new();
        let available = client.is_available().await;
        println!("Ollama available: {}", available);
    }

    #[tokio::test]
    #[ignore] // Requires Ollama running
    async fn test_ollama_list_models() {
        let client = OllamaFlash::new();
        let models = client.list_models().await.unwrap();
        println!("Available models: {:?}", models);
    }

    #[tokio::test]
    #[ignore] // Requires Ollama running with llama3.1
    async fn test_ollama_analyze() {
        let client = OllamaFlash::new();
        let result = client
            .analyze(
                "How much does your enterprise plan cost?",
                "Sales call for SaaS product",
            )
            .await;

        println!("Result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires Ollama running
    async fn test_ollama_status() {
        let status = check_ollama_status().await;
        println!("Ollama status: {:?}", status);
    }
}
