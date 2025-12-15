//! Voice Output Module
//!
//! Text-to-Speech (TTS) for speaking AI responses aloud.
//! Supports multiple backends:
//! - OpenAI TTS (high quality, requires API)
//! - Windows SAPI (local, no API needed)
//! - ElevenLabs (premium quality, requires API)

mod openai_tts;
mod windows_tts;
mod elevenlabs;

pub use openai_tts::{OpenAITTS, OpenAIVoice};
pub use windows_tts::WindowsTTS;
pub use elevenlabs::{ElevenLabsTTS, ElevenLabsVoice};

use anyhow::Result;
use tokio::sync::mpsc;

/// TTS Provider selection
#[derive(Debug, Clone, Default, PartialEq)]
pub enum TTSProvider {
    /// OpenAI TTS (default, good quality)
    #[default]
    OpenAI,
    /// Windows built-in TTS (free, offline)
    WindowsSAPI,
    /// ElevenLabs (premium quality)
    ElevenLabs,
    /// Disabled
    Disabled,
}

/// TTS Configuration
#[derive(Debug, Clone)]
pub struct TTSConfig {
    /// Provider to use
    pub provider: TTSProvider,
    /// Voice ID/name
    pub voice: String,
    /// Speech speed (0.5 to 2.0, default 1.0)
    pub speed: f32,
    /// Volume (0.0 to 1.0)
    pub volume: f32,
    /// API key (for cloud providers)
    pub api_key: Option<String>,
}

impl Default for TTSConfig {
    fn default() -> Self {
        Self {
            provider: TTSProvider::OpenAI,
            voice: "alloy".to_string(), // OpenAI default
            speed: 1.0,
            volume: 1.0,
            api_key: None,
        }
    }
}

/// TTS Engine trait for different providers
pub trait TTSEngine: Send + Sync {
    /// Speak text asynchronously
    fn speak(&self, text: &str) -> Result<()>;

    /// Stop current speech
    fn stop(&self) -> Result<()>;

    /// Check if currently speaking
    fn is_speaking(&self) -> bool;

    /// Get available voices
    fn list_voices(&self) -> Vec<String>;
}

/// Voice output manager
pub struct VoiceOutput {
    config: TTSConfig,
    is_enabled: bool,
    speech_queue: mpsc::Sender<String>,
}

impl VoiceOutput {
    /// Create a new voice output manager
    pub fn new(config: TTSConfig) -> Self {
        let (tx, mut rx) = mpsc::channel::<String>(10);

        let config_clone = config.clone();

        // Spawn speech processing task
        tokio::spawn(async move {
            while let Some(text) = rx.recv().await {
                match &config_clone.provider {
                    TTSProvider::OpenAI => {
                        if let Some(api_key) = &config_clone.api_key {
                            let tts = OpenAITTS::new(api_key.clone());
                            if let Err(e) = tts.speak(&text, &config_clone.voice).await {
                                tracing::warn!("TTS error: {}", e);
                            }
                        }
                    }
                    TTSProvider::WindowsSAPI => {
                        #[cfg(target_os = "windows")]
                        {
                            let tts = WindowsTTS::new();
                            if let Err(e) = tts.speak(&text) {
                                tracing::warn!("Windows TTS error: {}", e);
                            }
                        }
                    }
                    TTSProvider::ElevenLabs => {
                        if let Some(api_key) = &config_clone.api_key {
                            let tts = ElevenLabsTTS::new(api_key.clone());
                            if let Err(e) = tts.speak(&text, &config_clone.voice).await {
                                tracing::warn!("ElevenLabs TTS error: {}", e);
                            }
                        }
                    }
                    TTSProvider::Disabled => {}
                }
            }
        });

        Self {
            config,
            is_enabled: true,
            speech_queue: tx,
        }
    }

    /// Speak text
    pub async fn speak(&self, text: &str) -> Result<()> {
        if !self.is_enabled || self.config.provider == TTSProvider::Disabled {
            return Ok(());
        }

        self.speech_queue.send(text.to_string()).await?;
        Ok(())
    }

    /// Enable/disable voice output
    pub fn set_enabled(&mut self, enabled: bool) {
        self.is_enabled = enabled;
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    /// Get current provider
    pub fn provider(&self) -> &TTSProvider {
        &self.config.provider
    }
}

/// Check available TTS providers
pub fn check_tts_availability() -> TTSAvailability {
    let mut available = Vec::new();

    // Windows SAPI is always available on Windows
    #[cfg(target_os = "windows")]
    {
        available.push(TTSProvider::WindowsSAPI);
    }

    // Check for API keys in environment
    if std::env::var("OPENAI_API_KEY").is_ok() {
        available.push(TTSProvider::OpenAI);
    }

    if std::env::var("ELEVENLABS_API_KEY").is_ok() {
        available.push(TTSProvider::ElevenLabs);
    }

    TTSAvailability { available }
}

/// TTS availability info
#[derive(Debug, Clone)]
pub struct TTSAvailability {
    pub available: Vec<TTSProvider>,
}

impl TTSAvailability {
    pub fn has_any(&self) -> bool {
        !self.available.is_empty()
    }

    pub fn has_provider(&self, provider: &TTSProvider) -> bool {
        self.available.contains(provider)
    }
}
