//! GPT-4o-mini Integration
//!
//! Fallback fast model from OpenAI.
//! Slightly slower than Gemini Flash but very reliable.

use anyhow::Result;
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
        ResponseFormat, ResponseFormatType,
    },
    Client,
};

use super::bullet_extractor::FlashAnalysis;

/// GPT-4o-mini client
pub struct GPT4oMini {
    client: Client<OpenAIConfig>,
    model: String,
}

impl GPT4oMini {
    /// Create a new GPT-4o-mini client
    pub fn new(api_key: impl Into<String>) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        Self {
            client: Client::with_config(config),
            model: "gpt-4o-mini".to_string(),
        }
    }

    /// Use a specific model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Analyze transcript and extract quick response bullets
    pub async fn analyze(&self, transcript: &str, context: &str) -> Result<FlashAnalysis> {
        let system_prompt = r#"You are an instant analysis engine. Respond in <200ms.

OUTPUT: JSON only, no explanation

{
  "summary": "One sentence: what they're asking/saying",
  "bullets": [
    {"point": "Key thing to mention", "priority": 1},
    {"point": "Another point", "priority": 2},
    {"point": "Supporting detail", "priority": 3}
  ],
  "type": "question|objection|statement|buying_signal|technical|small_talk",
  "urgency": "answer_now|can_elaborate|just_listening"
}

Rules:
- Max 5 bullets
- Priority 1 = say this first (most important)
- Be specific, not generic
- Under 50 tokens total"#;

        let user_prompt = format!(
            "CONTEXT: {}\n\nTHEIR STATEMENT: \"{}\"",
            context, transcript
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(vec![
                ChatCompletionRequestMessage::System(
                    ChatCompletionRequestSystemMessageArgs::default()
                        .content(system_prompt)
                        .build()?,
                ),
                ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(user_prompt)
                        .build()?,
                ),
            ])
            .response_format(ResponseFormat {
                r#type: ResponseFormatType::JsonObject,
            })
            .max_tokens(200u32)
            .temperature(0.3)
            .build()?;

        let response = self.client.chat().create(request).await?;

        if let Some(choice) = response.choices.first() {
            if let Some(content) = &choice.message.content {
                let analysis: FlashAnalysis = serde_json::from_str(content)?;
                return Ok(analysis);
            }
        }

        Err(anyhow::anyhow!("No response from GPT-4o-mini"))
    }
}
