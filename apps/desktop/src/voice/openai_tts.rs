//! OpenAI TTS Integration
//!
//! High-quality text-to-speech using OpenAI's TTS API.
//! Supports multiple voices: alloy, echo, fable, onyx, nova, shimmer

use anyhow::Result;
use reqwest::Client;
use std::io::Cursor;

/// OpenAI voices
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpenAIVoice {
    /// Neutral, balanced
    Alloy,
    /// Warm, engaging
    Echo,
    /// Expressive, dramatic
    Fable,
    /// Deep, authoritative
    Onyx,
    /// Warm, friendly female
    Nova,
    /// Clear, positive
    Shimmer,
}

impl OpenAIVoice {
    pub fn as_str(&self) -> &'static str {
        match self {
            OpenAIVoice::Alloy => "alloy",
            OpenAIVoice::Echo => "echo",
            OpenAIVoice::Fable => "fable",
            OpenAIVoice::Onyx => "onyx",
            OpenAIVoice::Nova => "nova",
            OpenAIVoice::Shimmer => "shimmer",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "echo" => OpenAIVoice::Echo,
            "fable" => OpenAIVoice::Fable,
            "onyx" => OpenAIVoice::Onyx,
            "nova" => OpenAIVoice::Nova,
            "shimmer" => OpenAIVoice::Shimmer,
            _ => OpenAIVoice::Alloy,
        }
    }

    pub fn all() -> Vec<OpenAIVoice> {
        vec![
            OpenAIVoice::Alloy,
            OpenAIVoice::Echo,
            OpenAIVoice::Fable,
            OpenAIVoice::Onyx,
            OpenAIVoice::Nova,
            OpenAIVoice::Shimmer,
        ]
    }
}

/// OpenAI TTS client
pub struct OpenAITTS {
    api_key: String,
    client: Client,
    model: String,
}

impl OpenAITTS {
    /// Create a new OpenAI TTS client
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client: Client::new(),
            model: "tts-1".to_string(), // tts-1 or tts-1-hd
        }
    }

    /// Use HD model (higher quality, higher latency)
    pub fn with_hd(mut self) -> Self {
        self.model = "tts-1-hd".to_string();
        self
    }

    /// Generate speech and play it
    pub async fn speak(&self, text: &str, voice: &str) -> Result<()> {
        let audio_data = self.generate(text, voice).await?;
        play_audio(&audio_data)?;
        Ok(())
    }

    /// Generate speech audio (returns MP3 bytes)
    pub async fn generate(&self, text: &str, voice: &str) -> Result<Vec<u8>> {
        let response = self
            .client
            .post("https://api.openai.com/v1/audio/speech")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": self.model,
                "input": text,
                "voice": voice,
                "response_format": "mp3"
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("OpenAI TTS error ({}): {}", status, body));
        }

        let audio_data = response.bytes().await?.to_vec();
        Ok(audio_data)
    }

    /// Stream speech (for longer text)
    pub async fn stream(&self, _text: &str, _voice: &str) -> Result<()> {
        // OpenAI TTS doesn't support streaming yet
        // For now, use generate() for all text
        Ok(())
    }
}

/// Play audio bytes (MP3 format)
fn play_audio(audio_data: &[u8]) -> Result<()> {
    // Use rodio for cross-platform audio playback
    // Note: This requires the rodio crate to be added to dependencies

    #[cfg(feature = "audio_playback")]
    {
        use rodio::{Decoder, OutputStream, Sink};

        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        let cursor = Cursor::new(audio_data.to_vec());
        let source = Decoder::new(cursor)?;

        sink.append(source);
        sink.sleep_until_end();
    }

    #[cfg(not(feature = "audio_playback"))]
    {
        // Fallback: Save to temp file and use system player
        use std::process::Command;

        let temp_path = std::env::temp_dir().join("voice_copilot_tts.mp3");
        std::fs::write(&temp_path, audio_data)?;

        #[cfg(target_os = "windows")]
        {
            // Use Windows Media Player or default app
            Command::new("cmd")
                .args(["/C", "start", "", &temp_path.to_string_lossy()])
                .spawn()?;
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("afplay")
                .arg(&temp_path)
                .spawn()?
                .wait()?;
        }

        #[cfg(target_os = "linux")]
        {
            Command::new("mpv")
                .arg("--really-quiet")
                .arg(&temp_path)
                .spawn()?
                .wait()?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_from_str() {
        assert_eq!(OpenAIVoice::from_str("nova"), OpenAIVoice::Nova);
        assert_eq!(OpenAIVoice::from_str("ONYX"), OpenAIVoice::Onyx);
        assert_eq!(OpenAIVoice::from_str("unknown"), OpenAIVoice::Alloy);
    }
}
