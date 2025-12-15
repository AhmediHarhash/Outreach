//! Deepgram Streaming Client
//!
//! Real-time speech-to-text using Deepgram's Nova-2 model.
//! Provides the fastest and most accurate streaming transcription.

use anyhow::{anyhow, Result};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

use super::transcript::TranscriptSegment;

/// Deepgram client for streaming STT
pub struct DeepgramClient {
    api_key: String,
    model: String,
    language: String,
}

/// Deepgram streaming configuration
#[derive(Debug, Clone)]
pub struct DeepgramConfig {
    pub model: String,
    pub language: String,
    pub punctuate: bool,
    pub interim_results: bool,
    pub smart_format: bool,
    pub diarize: bool,
}

impl Default for DeepgramConfig {
    fn default() -> Self {
        Self {
            model: "nova-2".to_string(),
            language: "en".to_string(),
            punctuate: true,
            interim_results: true,
            smart_format: true,
            diarize: false, // Speaker diarization (adds latency)
        }
    }
}

/// Deepgram transcription response
#[derive(Debug, Deserialize)]
pub struct DeepgramResponse {
    #[serde(rename = "type")]
    pub response_type: String,
    pub channel: Option<ChannelResult>,
    pub is_final: Option<bool>,
    pub speech_final: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ChannelResult {
    pub alternatives: Vec<Alternative>,
}

#[derive(Debug, Deserialize)]
pub struct Alternative {
    pub transcript: String,
    pub confidence: f32,
    pub words: Option<Vec<Word>>,
}

#[derive(Debug, Deserialize)]
pub struct Word {
    pub word: String,
    pub start: f32,
    pub end: f32,
    pub confidence: f32,
}

impl DeepgramClient {
    /// Create a new Deepgram client
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: "nova-2".to_string(),
            language: "en".to_string(),
        }
    }

    /// Set the model to use
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set the language
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = language.into();
        self
    }

    /// Start a streaming transcription session
    ///
    /// Returns:
    /// - A sender to push audio data
    /// - A receiver to get transcript segments
    pub async fn start_streaming(
        &self,
        config: DeepgramConfig,
    ) -> Result<(mpsc::Sender<Vec<u8>>, mpsc::Receiver<TranscriptSegment>)> {
        // Build WebSocket URL with query parameters
        let mut url = Url::parse("wss://api.deepgram.com/v1/listen")?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("model", &config.model);
            query.append_pair("language", &config.language);
            query.append_pair("punctuate", &config.punctuate.to_string());
            query.append_pair("interim_results", &config.interim_results.to_string());
            query.append_pair("smart_format", &config.smart_format.to_string());
            query.append_pair("encoding", "linear16");
            query.append_pair("sample_rate", "16000");
            query.append_pair("channels", "1");

            if config.diarize {
                query.append_pair("diarize", "true");
            }
        }

        tracing::info!("Connecting to Deepgram: {}", url);

        // Connect with authorization header
        let request = http::Request::builder()
            .uri(url.as_str())
            .header("Authorization", format!("Token {}", self.api_key))
            .header("Host", "api.deepgram.com")
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", tungstenite_key())
            .body(())?;

        let (ws_stream, _) = connect_async(request).await?;
        let (mut write, mut read) = ws_stream.split();

        tracing::info!("Connected to Deepgram");

        // Channels for audio input and transcript output
        let (audio_tx, mut audio_rx) = mpsc::channel::<Vec<u8>>(100);
        let (transcript_tx, transcript_rx) = mpsc::channel::<TranscriptSegment>(100);

        // Task to send audio data
        tokio::spawn(async move {
            while let Some(audio_data) = audio_rx.recv().await {
                if write.send(Message::Binary(audio_data)).await.is_err() {
                    tracing::warn!("Failed to send audio to Deepgram");
                    break;
                }
            }

            // Send close frame
            let _ = write.send(Message::Close(None)).await;
        });

        // Task to receive transcripts
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(response) = serde_json::from_str::<DeepgramResponse>(&text) {
                            if let Some(segment) = parse_deepgram_response(response) {
                                if transcript_tx.send(segment).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        tracing::info!("Deepgram connection closed");
                        break;
                    }
                    Err(e) => {
                        tracing::error!("Deepgram WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok((audio_tx, transcript_rx))
    }
}

/// Parse Deepgram response into a transcript segment
fn parse_deepgram_response(response: DeepgramResponse) -> Option<TranscriptSegment> {
    if response.response_type != "Results" {
        return None;
    }

    let channel = response.channel?;
    let alternative = channel.alternatives.first()?;

    if alternative.transcript.is_empty() {
        return None;
    }

    Some(TranscriptSegment {
        text: alternative.transcript.clone(),
        confidence: alternative.confidence,
        is_final: response.is_final.unwrap_or(false),
        speaker: None, // Would be populated if diarization is enabled
        timestamp: chrono::Utc::now(),
    })
}

/// Generate a random WebSocket key
fn tungstenite_key() -> String {
    use base64::Engine;
    let mut key = [0u8; 16];
    getrandom::getrandom(&mut key).unwrap();
    base64::engine::general_purpose::STANDARD.encode(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_response() {
        let json = r#"{
            "type": "Results",
            "channel": {
                "alternatives": [{
                    "transcript": "Hello, how are you?",
                    "confidence": 0.95,
                    "words": []
                }]
            },
            "is_final": true
        }"#;

        let response: DeepgramResponse = serde_json::from_str(json).unwrap();
        let segment = parse_deepgram_response(response).unwrap();

        assert_eq!(segment.text, "Hello, how are you?");
        assert!(segment.is_final);
        assert!(segment.confidence > 0.9);
    }
}
