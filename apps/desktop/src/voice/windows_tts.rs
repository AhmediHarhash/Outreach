//! Windows SAPI TTS
//!
//! Local text-to-speech using Windows Speech API.
//! No API costs, works offline, uses system voices.

use anyhow::Result;

/// Windows TTS client using SAPI
pub struct WindowsTTS {
    voice: Option<String>,
    rate: i32, // -10 to 10
    volume: u32, // 0 to 100
}

impl WindowsTTS {
    /// Create a new Windows TTS client
    pub fn new() -> Self {
        Self {
            voice: None,
            rate: 0,
            volume: 100,
        }
    }

    /// Set voice by name
    pub fn with_voice(mut self, voice: impl Into<String>) -> Self {
        self.voice = Some(voice.into());
        self
    }

    /// Set speech rate (-10 slowest to 10 fastest)
    pub fn with_rate(mut self, rate: i32) -> Self {
        self.rate = rate.clamp(-10, 10);
        self
    }

    /// Set volume (0-100)
    pub fn with_volume(mut self, volume: u32) -> Self {
        self.volume = volume.min(100);
        self
    }

    /// Speak text synchronously
    #[cfg(target_os = "windows")]
    pub fn speak(&self, text: &str) -> Result<()> {
        use std::process::Command;

        // Use PowerShell to access SAPI
        let mut script = String::new();
        script.push_str("Add-Type -AssemblyName System.Speech; ");
        script.push_str("$synth = New-Object System.Speech.Synthesis.SpeechSynthesizer; ");

        if let Some(ref voice) = self.voice {
            script.push_str(&format!("$synth.SelectVoice('{}'); ", voice));
        }

        script.push_str(&format!("$synth.Rate = {}; ", self.rate));
        script.push_str(&format!("$synth.Volume = {}; ", self.volume));

        // Escape text for PowerShell
        let escaped_text = text
            .replace("'", "''")
            .replace("`", "``")
            .replace("$", "`$");

        script.push_str(&format!("$synth.Speak('{}'); ", escaped_text));

        Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output()?;

        Ok(())
    }

    /// Speak text (non-Windows fallback)
    #[cfg(not(target_os = "windows"))]
    pub fn speak(&self, _text: &str) -> Result<()> {
        Err(anyhow::anyhow!("Windows TTS only available on Windows"))
    }

    /// Speak text asynchronously
    #[cfg(target_os = "windows")]
    pub fn speak_async(&self, text: &str) -> Result<()> {
        use std::process::Command;

        let mut script = String::new();
        script.push_str("Add-Type -AssemblyName System.Speech; ");
        script.push_str("$synth = New-Object System.Speech.Synthesis.SpeechSynthesizer; ");

        if let Some(ref voice) = self.voice {
            script.push_str(&format!("$synth.SelectVoice('{}'); ", voice));
        }

        script.push_str(&format!("$synth.Rate = {}; ", self.rate));
        script.push_str(&format!("$synth.Volume = {}; ", self.volume));

        let escaped_text = text
            .replace("'", "''")
            .replace("`", "``")
            .replace("$", "`$");

        script.push_str(&format!("$synth.SpeakAsync('{}'); ", escaped_text));
        script.push_str("Start-Sleep -Milliseconds 100; "); // Brief pause to start

        Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .spawn()?;

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    pub fn speak_async(&self, _text: &str) -> Result<()> {
        Err(anyhow::anyhow!("Windows TTS only available on Windows"))
    }

    /// List available voices
    #[cfg(target_os = "windows")]
    pub fn list_voices() -> Result<Vec<String>> {
        use std::process::Command;

        let script = r#"
            Add-Type -AssemblyName System.Speech
            $synth = New-Object System.Speech.Synthesis.SpeechSynthesizer
            $synth.GetInstalledVoices() | ForEach-Object { $_.VoiceInfo.Name }
        "#;

        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let voices: Vec<String> = output_str
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(voices)
    }

    #[cfg(not(target_os = "windows"))]
    pub fn list_voices() -> Result<Vec<String>> {
        Ok(vec![])
    }

    /// Stop current speech
    #[cfg(target_os = "windows")]
    pub fn stop() -> Result<()> {
        use std::process::Command;

        let script = r#"
            Add-Type -AssemblyName System.Speech
            $synth = New-Object System.Speech.Synthesis.SpeechSynthesizer
            $synth.SpeakAsyncCancelAll()
        "#;

        Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output()?;

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    pub fn stop() -> Result<()> {
        Ok(())
    }
}

impl Default for WindowsTTS {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires Windows
    fn test_list_voices() {
        let voices = WindowsTTS::list_voices().unwrap();
        println!("Available voices: {:?}", voices);
        assert!(!voices.is_empty());
    }
}
