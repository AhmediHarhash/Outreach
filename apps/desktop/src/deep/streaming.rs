//! Streaming Response Types
//!
//! Common types for the Deep response stage.

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Deep analysis result - streams in over time
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeepAnalysis {
    /// The main detailed response
    pub content: String,

    /// Whether we're still receiving content
    pub is_streaming: bool,

    /// Question to ask them (appears at end)
    pub question_to_ask: Option<String>,

    /// How to handle pushback
    pub if_they_push_back: Option<String>,

    /// Key points extracted (for quick reference)
    pub key_points: Vec<String>,
}

/// A chunk of streaming response
#[derive(Debug, Clone)]
pub enum StreamChunk {
    /// New content to append
    Content(String),
    /// The question to ask them
    Question(String),
    /// Pushback handler
    Pushback(String),
    /// A key point
    KeyPoint(String),
    /// Stream completed
    Done,
    /// Error occurred
    Error(String),
}

/// Handle for receiving streaming responses
pub struct StreamingResponse {
    pub receiver: mpsc::Receiver<StreamChunk>,
}

impl StreamingResponse {
    /// Create a new streaming response handle
    pub fn new(receiver: mpsc::Receiver<StreamChunk>) -> Self {
        Self { receiver }
    }

    /// Collect all chunks into a DeepAnalysis
    pub async fn collect(mut self) -> DeepAnalysis {
        let mut analysis = DeepAnalysis {
            is_streaming: true,
            ..Default::default()
        };

        while let Some(chunk) = self.receiver.recv().await {
            match chunk {
                StreamChunk::Content(text) => {
                    analysis.content.push_str(&text);
                }
                StreamChunk::Question(q) => {
                    analysis.question_to_ask = Some(q);
                }
                StreamChunk::Pushback(p) => {
                    analysis.if_they_push_back = Some(p);
                }
                StreamChunk::KeyPoint(kp) => {
                    analysis.key_points.push(kp);
                }
                StreamChunk::Done => {
                    analysis.is_streaming = false;
                    break;
                }
                StreamChunk::Error(e) => {
                    analysis.is_streaming = false;
                    analysis.content = format!("Error: {}", e);
                    break;
                }
            }
        }

        analysis
    }
}

/// Deep prompt template for generating detailed responses
pub fn build_deep_prompt(
    transcript: &str,
    context: &str,
    flash_bullets: &[String],
    conversation_history: &str,
) -> String {
    let bullets_str = flash_bullets
        .iter()
        .enumerate()
        .map(|(i, b)| format!("{}. {}", i + 1, b))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"You are a real-time conversation advisor. The user is currently in a live call and needs a complete, well-structured response.

CONTEXT: {context}

CONVERSATION SO FAR:
{conversation_history}

THEIR LATEST STATEMENT: "{transcript}"

QUICK BULLETS ALREADY SHOWN:
{bullets_str}

YOUR TASK:
Provide a comprehensive response the user can speak or reference. The user is ALREADY talking using the bullets above, so your response should expand on those points with specifics.

FORMAT YOUR RESPONSE EXACTLY LIKE THIS:

## Direct Answer
[2-3 sentences that directly address what was asked. Be specific and confident.]

## Key Points
• [Expand on bullet 1 with concrete details, numbers, or examples]
• [Expand on bullet 2 with specifics]
• [Expand on bullet 3 if applicable]

## If They Push Back
[One sentence on how to handle likely objection or follow-up]

## Question to Ask Them
[A strategic question to regain control or qualify further]

RULES:
- Be conversational, not robotic
- Use specific examples when possible
- Match the tone to the context (sales = confident, interview = professional, technical = precise)
- Keep the total response under 200 words
- The "Question to Ask" should advance the conversation"#
    )
}
