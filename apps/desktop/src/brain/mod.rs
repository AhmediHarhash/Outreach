//! Brain Module - Orchestration Layer
//!
//! Coordinates the entire pipeline:
//! 1. Audio capture → STT
//! 2. STT → Flash model (quick bullets)
//! 3. STT → Deep model (detailed response)
//!
//! Manages conversation context and mode-specific behavior.
//! Includes hybrid AI routing for optimal speed/quality balance.

pub mod pipeline;
mod context;
mod intent;
pub mod modes;
pub mod hybrid_router;

pub use pipeline::{CopilotPipeline, PipelineConfig, CopilotState, PipelineEvent, FlashModelChoice};
pub use context::{ConversationContext, ConversationTurn};
pub use intent::{IntentAnalyzer, DetectedIntent};
pub use hybrid_router::{HybridRouter, HybridRouterConfig, RoutingStrategy, Complexity, AIProvider};
