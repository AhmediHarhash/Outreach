//! Application Settings
//!
//! Persistent configuration stored in the user's config directory.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main settings structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// API keys (stored securely)
    pub api_keys: ApiKeys,
    /// Audio capture settings
    pub audio: AudioSettings,
    /// AI model settings
    pub models: ModelSettings,
    /// UI preferences
    pub ui: UiSettings,
    /// Keyboard shortcuts
    pub hotkeys: HotkeySettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            api_keys: ApiKeys::default(),
            audio: AudioSettings::default(),
            models: ModelSettings::default(),
            ui: UiSettings::default(),
            hotkeys: HotkeySettings::default(),
        }
    }
}

/// API key storage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiKeys {
    /// OpenAI API key (for GPT-4o, o1, Whisper)
    pub openai: Option<String>,
    /// Anthropic API key (for Claude)
    pub anthropic: Option<String>,
    /// Google AI API key (for Gemini)
    pub google: Option<String>,
    /// Deepgram API key (for STT)
    pub deepgram: Option<String>,
}

impl ApiKeys {
    /// Check if any STT provider is configured
    pub fn has_stt(&self) -> bool {
        self.deepgram.is_some() || self.openai.is_some()
    }

    /// Check if any LLM provider is configured
    pub fn has_llm(&self) -> bool {
        self.openai.is_some() || self.anthropic.is_some() || self.google.is_some()
    }

    /// Save API keys securely using the OS keychain
    pub fn save_secure(&self) -> Result<()> {
        let keyring = keyring::Entry::new("voice-copilot", "api-keys")?;

        let json = serde_json::to_string(self)?;
        keyring.set_password(&json)?;

        Ok(())
    }

    /// Load API keys from the OS keychain
    pub fn load_secure() -> Result<Self> {
        let keyring = keyring::Entry::new("voice-copilot", "api-keys")?;

        match keyring.get_password() {
            Ok(json) => Ok(serde_json::from_str(&json)?),
            Err(_) => Ok(Self::default()),
        }
    }
}

/// Audio capture settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    /// Input device name (None = default)
    pub input_device: Option<String>,
    /// Sample rate for capture
    pub sample_rate: u32,
    /// Whether to capture system audio (loopback)
    pub capture_system_audio: bool,
    /// Whether to capture microphone
    pub capture_microphone: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            input_device: None,
            sample_rate: 16000,
            capture_system_audio: true,
            capture_microphone: false,
        }
    }
}

/// AI model settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSettings {
    /// STT provider
    pub stt_provider: SttProvider,
    /// Flash model (quick responses)
    pub flash_model: FlashModel,
    /// Deep model (detailed responses)
    pub deep_model: DeepModel,
    /// Whether to use o1 for complex questions
    pub use_o1_for_complex: bool,
}

impl Default for ModelSettings {
    fn default() -> Self {
        Self {
            stt_provider: SttProvider::Deepgram,
            flash_model: FlashModel::GeminiFlash,
            deep_model: DeepModel::ClaudeSonnet,
            use_o1_for_complex: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum SttProvider {
    #[default]
    Deepgram,
    OpenAIRealtime,
    LocalWhisper,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum FlashModel {
    #[default]
    GeminiFlash,
    GPT4oMini,
    /// Local Ollama (Llama 3.1 8B or other local models)
    LocalOllama,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum DeepModel {
    #[default]
    ClaudeSonnet,
    GPT4o,
    O1Preview,
}

/// UI preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    /// Window always on top
    pub always_on_top: bool,
    /// Window opacity (0.0 to 1.0)
    pub opacity: f32,
    /// Default mode
    pub default_mode: String,
    /// Show transcript section
    pub show_transcript: bool,
    /// Compact mode
    pub compact_mode: bool,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            always_on_top: true,
            opacity: 0.95,
            default_mode: "sales".to_string(),
            show_transcript: true,
            compact_mode: false,
        }
    }
}

/// Keyboard shortcuts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeySettings {
    /// Toggle listening
    pub toggle_listen: String,
    /// Hide/show window
    pub toggle_visibility: String,
    /// Switch mode
    pub switch_mode: String,
    /// Copy last suggestion
    pub copy_suggestion: String,
}

impl Default for HotkeySettings {
    fn default() -> Self {
        Self {
            toggle_listen: "Ctrl+Shift+S".to_string(),
            toggle_visibility: "Ctrl+Shift+H".to_string(),
            switch_mode: "Ctrl+Shift+M".to_string(),
            copy_suggestion: "Ctrl+Shift+C".to_string(),
        }
    }
}

impl Settings {
    /// Get the settings file path
    pub fn path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("voice-copilot");

        std::fs::create_dir_all(&config_dir).ok();

        config_dir.join("settings.json")
    }

    /// Load settings from disk
    pub fn load() -> Result<Self> {
        let path = Self::path();

        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let mut settings: Settings = serde_json::from_str(&content)?;

            // Load API keys from secure storage
            if let Ok(keys) = ApiKeys::load_secure() {
                settings.api_keys = keys;
            }

            Ok(settings)
        } else {
            Ok(Self::default())
        }
    }

    /// Save settings to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::path();

        // Save API keys securely
        self.api_keys.save_secure()?;

        // Save other settings (without API keys) to JSON
        let mut settings_to_save = self.clone();
        settings_to_save.api_keys = ApiKeys::default(); // Don't save keys in plain text

        let content = serde_json::to_string_pretty(&settings_to_save)?;
        std::fs::write(&path, content)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default() {
        let settings = Settings::default();
        assert!(settings.ui.always_on_top);
        assert_eq!(settings.models.stt_provider, SttProvider::Deepgram);
    }
}
