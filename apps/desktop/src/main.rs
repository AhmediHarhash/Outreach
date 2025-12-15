//! Voice Copilot - Real-time AI assistant for voice calls
//!
//! A native desktop application that captures system audio, transcribes in real-time,
//! and provides AI-powered suggestions during sales calls, interviews, and technical discussions.

mod capture;
mod flash;
mod deep;
mod brain;
mod ui;
mod config;
mod voice;
mod analytics;
mod prompts;
mod recording;
pub mod updater;

use anyhow::Result;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() -> Result<()> {
    // Load .env file if present
    if let Ok(env_path) = std::env::current_dir() {
        let dotenv_path = env_path.join(".env");
        if dotenv_path.exists() {
            load_dotenv(&dotenv_path);
        }
    }

    // Initialize logging
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("voice_copilot=debug".parse()?))
        .init();

    tracing::info!("Starting Voice Copilot v{}", env!("CARGO_PKG_VERSION"));

    // Check for API keys
    let has_deepgram = std::env::var("DEEPGRAM_API_KEY").is_ok();
    let has_openai = std::env::var("OPENAI_API_KEY").is_ok();

    if has_deepgram {
        tracing::info!("Deepgram API key found");
    }
    if has_openai {
        tracing::info!("OpenAI API key found");
    }

    if !has_deepgram && !has_openai {
        tracing::warn!("No STT API key found! Please add DEEPGRAM_API_KEY or OPENAI_API_KEY to .env");
    }

    // Launch the Dioxus desktop application
    ui::launch_app();

    Ok(())
}

/// Simple .env file loader
fn load_dotenv(path: &std::path::Path) {
    if let Ok(content) = std::fs::read_to_string(path) {
        for line in content.lines() {
            let line = line.trim();
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // Parse KEY=VALUE
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                // Don't override existing env vars
                if std::env::var(key).is_err() {
                    std::env::set_var(key, value);
                }
            }
        }
    }
}
