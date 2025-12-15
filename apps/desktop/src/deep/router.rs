//! Model Router
//!
//! Intelligently routes requests to the appropriate deep model based on:
//! - Question complexity
//! - Context mode
//! - Latency requirements

use super::{ClaudeSonnet, GPT4o, O1Preview};
use super::streaming::StreamingResponse;
use crate::flash::StatementType;
use anyhow::Result;

/// Available deep models
#[derive(Debug, Clone, PartialEq)]
pub enum ModelChoice {
    /// Claude 3.5 Sonnet - Best for most cases
    ClaudeSonnet,
    /// GPT-4o - Fast and reliable alternative
    GPT4o,
    /// o1-preview - For complex reasoning (slower)
    O1Preview,
}

impl ModelChoice {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ClaudeSonnet => "Claude 3.5 Sonnet",
            Self::GPT4o => "GPT-4o",
            Self::O1Preview => "o1-preview",
        }
    }

    pub fn expected_latency(&self) -> &'static str {
        match self {
            Self::ClaudeSonnet => "1-2s",
            Self::GPT4o => "1-2s",
            Self::O1Preview => "5-10s",
        }
    }
}

/// Router for selecting and using deep models
pub struct ModelRouter {
    claude: Option<ClaudeSonnet>,
    gpt4o: Option<GPT4o>,
    o1: Option<O1Preview>,
    default_model: ModelChoice,
}

impl ModelRouter {
    /// Create a new model router
    pub fn new() -> Self {
        Self {
            claude: None,
            gpt4o: None,
            o1: None,
            default_model: ModelChoice::ClaudeSonnet,
        }
    }

    /// Configure Claude
    pub fn with_claude(mut self, api_key: impl Into<String>) -> Self {
        self.claude = Some(ClaudeSonnet::new(api_key));
        self
    }

    /// Configure GPT-4o
    pub fn with_gpt4o(mut self, api_key: impl Into<String>) -> Self {
        self.gpt4o = Some(GPT4o::new(api_key));
        self
    }

    /// Configure o1
    pub fn with_o1(mut self, api_key: impl Into<String>) -> Self {
        self.o1 = Some(O1Preview::new(api_key));
        self
    }

    /// Set the default model
    pub fn with_default(mut self, model: ModelChoice) -> Self {
        self.default_model = model;
        self
    }

    /// Automatically select the best model for the given input
    pub fn select_model(
        &self,
        transcript: &str,
        statement_type: &StatementType,
        force_model: Option<ModelChoice>,
    ) -> ModelChoice {
        // If forced, use that model
        if let Some(model) = force_model {
            return model;
        }

        // Check if o1 should be used for complex questions
        if super::o1::should_use_o1(transcript, &format!("{:?}", statement_type)) {
            if self.o1.is_some() {
                return ModelChoice::O1Preview;
            }
        }

        // Default selection
        match self.default_model {
            ModelChoice::ClaudeSonnet if self.claude.is_some() => ModelChoice::ClaudeSonnet,
            ModelChoice::GPT4o if self.gpt4o.is_some() => ModelChoice::GPT4o,
            _ => {
                // Fallback to whatever is available
                if self.claude.is_some() {
                    ModelChoice::ClaudeSonnet
                } else if self.gpt4o.is_some() {
                    ModelChoice::GPT4o
                } else {
                    ModelChoice::O1Preview
                }
            }
        }
    }

    /// Generate a streaming response using the selected model
    pub async fn analyze_streaming(
        &self,
        transcript: &str,
        context: &str,
        flash_bullets: &[String],
        conversation_history: &str,
        model_choice: ModelChoice,
    ) -> Result<StreamingResponse> {
        match model_choice {
            ModelChoice::ClaudeSonnet => {
                let claude = self.claude.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("Claude not configured")
                })?;
                claude.analyze_streaming(transcript, context, flash_bullets, conversation_history).await
            }
            ModelChoice::GPT4o => {
                let gpt4o = self.gpt4o.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("GPT-4o not configured")
                })?;
                gpt4o.analyze_streaming(transcript, context, flash_bullets, conversation_history).await
            }
            ModelChoice::O1Preview => {
                // o1 doesn't support streaming, so we wrap the response
                let o1 = self.o1.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("o1 not configured")
                })?;
                let response = o1.analyze(transcript, context).await?;

                let (tx, rx) = tokio::sync::mpsc::channel(10);
                tokio::spawn(async move {
                    let _ = tx.send(super::streaming::StreamChunk::Content(response)).await;
                    let _ = tx.send(super::streaming::StreamChunk::Done).await;
                });

                Ok(StreamingResponse::new(rx))
            }
        }
    }
}

impl Default for ModelRouter {
    fn default() -> Self {
        Self::new()
    }
}
