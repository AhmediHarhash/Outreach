//! ElevenLabs TTS Integration
//!
//! Premium quality text-to-speech using ElevenLabs API.
//! Offers the most natural sounding voices.

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// ElevenLabs voice presets
#[derive(Debug, Clone)]
pub struct ElevenLabsVoice {
    pub id: String,
    pub name: String,
    pub category: String,
}

impl ElevenLabsVoice {
    /// Pre-made voices available to all users
    pub fn premade() -> Vec<Self> {
        vec![
            Self {
                id: "21m00Tcm4TlvDq8ikWAM".to_string(),
                name: "Rachel".to_string(),
                category: "premade".to_string(),
            },
            Self {
                id: "AZnzlk1XvdvUeBnXmlld".to_string(),
                name: "Domi".to_string(),
                category: "premade".to_string(),
            },
            Self {
                id: "EXAVITQu4vr4xnSDxMaL".to_string(),
                name: "Bella".to_string(),
                category: "premade".to_string(),
            },
            Self {
                id: "ErXwobaYiN019PkySvjV".to_string(),
                name: "Antoni".to_string(),
                category: "premade".to_string(),
            },
            Self {
                id: "MF3mGyEYCl7XYWbV9V6O".to_string(),
                name: "Elli".to_string(),
                category: "premade".to_string(),
            },
            Self {
                id: "TxGEqnHWrfWFTfGW9XjX".to_string(),
                name: "Josh".to_string(),
                category: "premade".to_string(),
            },
            Self {
                id: "VR6AewLTigWG4xSOukaG".to_string(),
                name: "Arnold".to_string(),
                category: "premade".to_string(),
            },
            Self {
                id: "pNInz6obpgDQGcFmaJgB".to_string(),
                name: "Adam".to_string(),
                category: "premade".to_string(),
            },
            Self {
                id: "yoZ06aMxZJJ28mfd3POQ".to_string(),
                name: "Sam".to_string(),
                category: "premade".to_string(),
            },
        ]
    }
}

/// ElevenLabs voice settings
#[derive(Debug, Clone, Serialize)]
pub struct VoiceSettings {
    pub stability: f32,        // 0.0 to 1.0
    pub similarity_boost: f32, // 0.0 to 1.0
    pub style: f32,            // 0.0 to 1.0 (only for some voices)
    pub use_speaker_boost: bool,
}

impl Default for VoiceSettings {
    fn default() -> Self {
        Self {
            stability: 0.5,
            similarity_boost: 0.75,
            style: 0.0,
            use_speaker_boost: true,
        }
    }
}

/// ElevenLabs API response for voices
#[derive(Debug, Deserialize)]
struct VoicesResponse {
    voices: Vec<VoiceInfo>,
}

#[derive(Debug, Deserialize)]
struct VoiceInfo {
    voice_id: String,
    name: String,
    category: Option<String>,
}

/// ElevenLabs TTS client
pub struct ElevenLabsTTS {
    api_key: String,
    client: Client,
    model_id: String,
    settings: VoiceSettings,
}

impl ElevenLabsTTS {
    /// Create a new ElevenLabs TTS client
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client: Client::new(),
            model_id: "eleven_monolingual_v1".to_string(),
            settings: VoiceSettings::default(),
        }
    }

    /// Use multilingual model (supports multiple languages)
    pub fn multilingual(mut self) -> Self {
        self.model_id = "eleven_multilingual_v2".to_string();
        self
    }

    /// Use turbo model (lower latency)
    pub fn turbo(mut self) -> Self {
        self.model_id = "eleven_turbo_v2".to_string();
        self
    }

    /// Set voice settings
    pub fn with_settings(mut self, settings: VoiceSettings) -> Self {
        self.settings = settings;
        self
    }

    /// Generate speech and play it
    pub async fn speak(&self, text: &str, voice_id: &str) -> Result<()> {
        let audio_data = self.generate(text, voice_id).await?;
        play_audio(&audio_data)?;
        Ok(())
    }

    /// Generate speech audio (returns MP3 bytes)
    pub async fn generate(&self, text: &str, voice_id: &str) -> Result<Vec<u8>> {
        let url = format!(
            "https://api.elevenlabs.io/v1/text-to-speech/{}",
            voice_id
        );

        let response = self
            .client
            .post(&url)
            .header("xi-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .header("Accept", "audio/mpeg")
            .json(&serde_json::json!({
                "text": text,
                "model_id": self.model_id,
                "voice_settings": self.settings
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("ElevenLabs error ({}): {}", status, body));
        }

        let audio_data = response.bytes().await?.to_vec();
        Ok(audio_data)
    }

    /// List available voices
    pub async fn list_voices(&self) -> Result<Vec<ElevenLabsVoice>> {
        let response = self
            .client
            .get("https://api.elevenlabs.io/v1/voices")
            .header("xi-api-key", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to list voices: HTTP {}",
                response.status()
            ));
        }

        let voices_response: VoicesResponse = response.json().await?;

        Ok(voices_response
            .voices
            .into_iter()
            .map(|v| ElevenLabsVoice {
                id: v.voice_id,
                name: v.name,
                category: v.category.unwrap_or_else(|| "custom".to_string()),
            })
            .collect())
    }

    /// Get user subscription info (quota remaining)
    pub async fn get_subscription_info(&self) -> Result<SubscriptionInfo> {
        let response = self
            .client
            .get("https://api.elevenlabs.io/v1/user/subscription")
            .header("xi-api-key", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to get subscription: HTTP {}",
                response.status()
            ));
        }

        let info: SubscriptionInfo = response.json().await?;
        Ok(info)
    }
}

/// Subscription info from ElevenLabs
#[derive(Debug, Deserialize)]
pub struct SubscriptionInfo {
    pub character_count: u32,
    pub character_limit: u32,
    pub tier: String,
}

/// Play audio bytes
fn play_audio(audio_data: &[u8]) -> Result<()> {
    // Save to temp file and play
    use std::process::Command;

    let temp_path = std::env::temp_dir().join("voice_copilot_elevenlabs.mp3");
    std::fs::write(&temp_path, audio_data)?;

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "/B", "", &temp_path.to_string_lossy()])
            .spawn()?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("afplay")
            .arg(&temp_path)
            .spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("mpv")
            .arg("--really-quiet")
            .arg(&temp_path)
            .spawn()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_premade_voices() {
        let voices = ElevenLabsVoice::premade();
        assert!(!voices.is_empty());
        assert!(voices.iter().any(|v| v.name == "Rachel"));
    }
}
