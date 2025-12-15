//! Flash Module - Stage 2 (Quick Response)
//!
//! Fast AI responses using Gemini 2.0 Flash, GPT-4o-mini, or local Ollama.
//! Provides instant bullet points within ~500-1000ms.

mod gemini;
mod gpt4o_mini;
mod ollama;
mod bullet_extractor;

pub use gemini::GeminiFlash;
pub use gpt4o_mini::GPT4oMini;
pub use ollama::{OllamaFlash, OllamaStatus, OllamaModel, check_ollama_status};
pub use bullet_extractor::{FlashAnalysis, Bullet, StatementType, Urgency, extract_bullets};
