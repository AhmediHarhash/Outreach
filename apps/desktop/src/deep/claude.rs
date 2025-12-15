//! Claude 3.5 Sonnet Integration
//!
//! The best reasoning model for detailed, nuanced responses.
//! Excellent at structured output and following complex instructions.

use anyhow::Result;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use super::streaming::{StreamChunk, StreamingResponse, build_deep_prompt};

/// Claude 3.5 Sonnet client
pub struct ClaudeSonnet {
    api_key: String,
    client: Client,
    model: String,
}

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct StreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    delta: Option<Delta>,
}

#[derive(Debug, Deserialize)]
struct Delta {
    #[serde(rename = "type")]
    delta_type: Option<String>,
    text: Option<String>,
}

impl ClaudeSonnet {
    /// Create a new Claude Sonnet client
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client: Client::new(),
            model: "claude-sonnet-4-20250514".to_string(), // Claude 3.5 Sonnet
        }
    }

    /// Use a specific model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Generate a detailed response with streaming
    pub async fn analyze_streaming(
        &self,
        transcript: &str,
        context: &str,
        flash_bullets: &[String],
        conversation_history: &str,
    ) -> Result<StreamingResponse> {
        let prompt = build_deep_prompt(transcript, context, flash_bullets, conversation_history);

        let request = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 1024,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            stream: true,
        };

        let (tx, rx) = mpsc::channel(100);

        let client = self.client.clone();
        let api_key = self.api_key.clone();

        tokio::spawn(async move {
            let result = client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&request)
                .send()
                .await;

            match result {
                Ok(response) => {
                    let mut stream = response.bytes_stream();
                    let mut buffer = String::new();

                    while let Some(chunk) = stream.next().await {
                        match chunk {
                            Ok(bytes) => {
                                buffer.push_str(&String::from_utf8_lossy(&bytes));

                                // Parse SSE events from buffer
                                while let Some(event_end) = buffer.find("\n\n") {
                                    let event_str = buffer[..event_end].to_string();
                                    buffer = buffer[event_end + 2..].to_string();

                                    // Parse the event
                                    if let Some(data) = event_str.strip_prefix("data: ") {
                                        if let Ok(event) = serde_json::from_str::<StreamEvent>(data) {
                                            if let Some(delta) = event.delta {
                                                if let Some(text) = delta.text {
                                                    if tx.send(StreamChunk::Content(text)).await.is_err() {
                                                        return;
                                                    }
                                                }
                                            }

                                            if event.event_type == "message_stop" {
                                                let _ = tx.send(StreamChunk::Done).await;
                                                return;
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(StreamChunk::Error(e.to_string())).await;
                                return;
                            }
                        }
                    }

                    let _ = tx.send(StreamChunk::Done).await;
                }
                Err(e) => {
                    let _ = tx.send(StreamChunk::Error(e.to_string())).await;
                }
            }
        });

        Ok(StreamingResponse::new(rx))
    }

    /// Generate a response without streaming (for simpler use cases)
    pub async fn analyze(&self, transcript: &str, context: &str) -> Result<String> {
        let prompt = build_deep_prompt(transcript, context, &[], "");

        let request = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 1024,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            stream: false,
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        #[derive(Deserialize)]
        struct NonStreamResponse {
            content: Vec<ContentBlock>,
        }

        #[derive(Deserialize)]
        struct ContentBlock {
            text: String,
        }

        let result: NonStreamResponse = response.json().await?;

        Ok(result.content.first().map(|c| c.text.clone()).unwrap_or_default())
    }
}
