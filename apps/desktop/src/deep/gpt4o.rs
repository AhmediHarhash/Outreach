//! GPT-4o Integration
//!
//! OpenAI's flagship model - great alternative to Claude.
//! Fast and reliable with excellent instruction following.

use anyhow::Result;
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use futures::StreamExt;
use tokio::sync::mpsc;

use super::streaming::{StreamChunk, StreamingResponse, build_deep_prompt};

/// GPT-4o client
pub struct GPT4o {
    client: Client<OpenAIConfig>,
    model: String,
}

impl GPT4o {
    /// Create a new GPT-4o client
    pub fn new(api_key: impl Into<String>) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        Self {
            client: Client::with_config(config),
            model: "gpt-4o".to_string(),
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

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(vec![ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessageArgs::default()
                    .content(prompt)
                    .build()?,
            )])
            .max_tokens(1024u32)
            .temperature(0.7)
            .stream(true)
            .build()?;

        let (tx, rx) = mpsc::channel(100);

        let client = self.client.clone();

        tokio::spawn(async move {
            match client.chat().create_stream(request).await {
                Ok(mut stream) => {
                    while let Some(result) = stream.next().await {
                        match result {
                            Ok(response) => {
                                for choice in response.choices {
                                    if let Some(content) = choice.delta.content {
                                        if tx.send(StreamChunk::Content(content)).await.is_err() {
                                            return;
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

    /// Generate a response without streaming
    pub async fn analyze(&self, transcript: &str, context: &str) -> Result<String> {
        let prompt = build_deep_prompt(transcript, context, &[], "");

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(vec![ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessageArgs::default()
                    .content(prompt)
                    .build()?,
            )])
            .max_tokens(1024u32)
            .temperature(0.7)
            .build()?;

        let response = self.client.chat().create(request).await?;

        Ok(response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default())
    }
}
