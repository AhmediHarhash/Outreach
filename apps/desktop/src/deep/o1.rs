//! o1-preview Integration
//!
//! OpenAI's reasoning model - for complex questions that require deep thinking.
//! Slower (5-10s) but much better at complex reasoning.

use anyhow::Result;
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};

use super::streaming::build_deep_prompt;

/// o1-preview client
pub struct O1Preview {
    client: Client<OpenAIConfig>,
    model: String,
}

impl O1Preview {
    /// Create a new o1-preview client
    pub fn new(api_key: impl Into<String>) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        Self {
            client: Client::with_config(config),
            model: "o1-preview".to_string(),
        }
    }

    /// Use a specific model (o1, o1-mini)
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Generate a response for complex questions
    ///
    /// Note: o1 doesn't support streaming, so this is always a blocking call
    pub async fn analyze(&self, transcript: &str, context: &str) -> Result<String> {
        // o1 works best with detailed prompts
        let prompt = format!(
            r#"You are helping someone respond in a live conversation. Think deeply about the best response.

CONTEXT: {}

THEY SAID: "{}"

Provide a thoughtful, well-reasoned response that:
1. Directly addresses their question/concern
2. Shows deep understanding of the topic
3. Provides specific, actionable information
4. Ends with a question to advance the conversation

Be concise but thorough. The user needs to be able to speak this response naturally."#,
            context, transcript
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(vec![ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessageArgs::default()
                    .content(prompt)
                    .build()?,
            )])
            .build()?;

        let response = self.client.chat().create(request).await?;

        Ok(response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default())
    }
}

/// Determine if a question is complex enough to warrant o1
pub fn should_use_o1(transcript: &str, statement_type: &str) -> bool {
    // Use o1 for:
    // 1. Long, multi-part questions
    // 2. Technical architecture questions
    // 3. Complex comparisons
    // 4. Strategic/philosophical questions

    let word_count = transcript.split_whitespace().count();

    // Complex indicators
    let complex_keywords = [
        "how would you architect",
        "what's your approach to",
        "compare and contrast",
        "trade-offs between",
        "pros and cons",
        "in your experience",
        "walk me through",
        "explain the reasoning",
        "why did you choose",
        "what if",
    ];

    let has_complex_keywords = complex_keywords
        .iter()
        .any(|kw| transcript.to_lowercase().contains(kw));

    // Use o1 if:
    // - Question is long (>30 words)
    // - Contains complex keywords
    // - Is a technical question
    word_count > 30 || has_complex_keywords || statement_type == "technical"
}
