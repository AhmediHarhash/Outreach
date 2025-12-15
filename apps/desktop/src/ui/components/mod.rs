//! UI Components
//!
//! Reusable UI building blocks for the Voice Copilot interface.

mod transcript_view;
mod flash_card;
mod deep_card;
mod status_bar;
mod mode_selector;

pub use transcript_view::TranscriptView;
pub use flash_card::FlashCard;
pub use deep_card::DeepCard;
pub use status_bar::StatusBar;
pub use mode_selector::ModeSelector;
