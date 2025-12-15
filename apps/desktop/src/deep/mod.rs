//! Deep Module - Stage 3 (Detailed Response)
//!
//! Intelligent AI responses using Claude 3.5 Sonnet, GPT-4o, or o1.
//! Provides comprehensive, structured answers that stream in while you talk.

mod claude;
mod gpt4o;
mod o1;
mod router;
mod streaming;

pub use claude::ClaudeSonnet;
pub use gpt4o::GPT4o;
pub use o1::O1Preview;
pub use router::{ModelRouter, ModelChoice};
pub use streaming::{DeepAnalysis, StreamingResponse};
