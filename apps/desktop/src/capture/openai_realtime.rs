//! OpenAI Realtime API Client
//!
//! Alternative to Deepgram using OpenAI's native realtime API.
//! Provides integrated STT with the option for voice responses.

use anyhow::Result;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

use super::transcript::TranscriptSegment;

/// OpenAI Realtime client for streaming STT
pub struct OpenAIRealtimeClient {
    api_key: String,
    model: String,
}

/// OpenAI Realtime session configuration
#[derive(Debug, Clone, Serialize)]
pub struct RealtimeConfig {
    pub modalities: Vec<String>,
    pub instructions: String,
    pub voice: String,
    pub input_audio_format: String,
    pub output_audio_format: String,
    pub input_audio_transcription: Option<InputTranscriptionConfig>,
    pub turn_detection: Option<TurnDetectionConfig>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InputTranscriptionConfig {
    pub model: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TurnDetectionConfig {
    #[serde(rename = "type")]
    pub detection_type: String,
    pub threshold: f32,
    pub prefix_padding_ms: u32,
    pub silence_duration_ms: u32,
}

impl Default for RealtimeConfig {
    fn default() -> Self {
        Self {
            modalities: vec!["text".to_string(), "audio".to_string()],
            instructions: "You are a helpful assistant. Listen and transcribe.".to_string(),
            voice: "alloy".to_string(),
            input_audio_format: "pcm16".to_string(),
            output_audio_format: "pcm16".to_string(),
            input_audio_transcription: Some(InputTranscriptionConfig {
                model: "whisper-1".to_string(),
            }),
            turn_detection: Some(TurnDetectionConfig {
                detection_type: "server_vad".to_string(),
                threshold: 0.5,
                prefix_padding_ms: 300,
                silence_duration_ms: 500,
            }),
        }
    }
}

/// OpenAI Realtime events
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ClientEvent {
    #[serde(rename = "session.update")]
    SessionUpdate { session: RealtimeConfig },

    #[serde(rename = "input_audio_buffer.append")]
    InputAudioBufferAppend { audio: String },

    #[serde(rename = "input_audio_buffer.commit")]
    InputAudioBufferCommit {},
}

#[derive(Debug, Deserialize)]
pub struct ServerEvent {
    #[serde(rename = "type")]
    pub event_type: String,

    // For transcription events
    pub transcript: Option<String>,

    // For error events
    pub error: Option<ErrorDetail>,
}

#[derive(Debug, Deserialize)]
pub struct ErrorDetail {
    pub message: String,
    pub code: Option<String>,
}

impl OpenAIRealtimeClient {
    /// Create a new OpenAI Realtime client
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: "gpt-4o-realtime-preview-2024-12-17".to_string(),
        }
    }

    /// Set the model to use
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Start a realtime session for transcription
    ///
    /// Returns:
    /// - A sender to push audio data (base64 encoded PCM16)
    /// - A receiver to get transcript segments
    pub async fn start_streaming(
        &self,
        config: RealtimeConfig,
    ) -> Result<(mpsc::Sender<Vec<u8>>, mpsc::Receiver<TranscriptSegment>)> {
        let url = format!(
            "wss://api.openai.com/v1/realtime?model={}",
            self.model
        );

        tracing::info!("Connecting to OpenAI Realtime API");

        let request = http::Request::builder()
            .uri(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("OpenAI-Beta", "realtime=v1")
            .header("Host", "api.openai.com")
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", generate_ws_key())
            .body(())?;

        let (ws_stream, _) = connect_async(request).await?;
        let (mut write, mut read) = ws_stream.split();

        tracing::info!("Connected to OpenAI Realtime API");

        // Send session configuration
        let session_update = ClientEvent::SessionUpdate { session: config };
        let msg = serde_json::to_string(&session_update)?;
        write.send(Message::Text(msg)).await?;

        // Channels for audio input and transcript output
        let (audio_tx, mut audio_rx) = mpsc::channel::<Vec<u8>>(100);
        let (transcript_tx, transcript_rx) = mpsc::channel::<TranscriptSegment>(100);

        // Task to send audio data
        tokio::spawn(async move {
            while let Some(audio_data) = audio_rx.recv().await {
                // OpenAI expects base64 encoded audio
                use base64::Engine;
                let audio_base64 = base64::engine::general_purpose::STANDARD.encode(&audio_data);

                let event = ClientEvent::InputAudioBufferAppend { audio: audio_base64 };
                if let Ok(msg) = serde_json::to_string(&event) {
                    if write.send(Message::Text(msg)).await.is_err() {
                        tracing::warn!("Failed to send audio to OpenAI");
                        break;
                    }
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
                        if let Ok(event) = serde_json::from_str::<ServerEvent>(&text) {
                            match event.event_type.as_str() {
                                "conversation.item.input_audio_transcription.completed" => {
                                    if let Some(transcript) = event.transcript {
                                        let segment = TranscriptSegment {
                                            text: transcript,
                                            confidence: 1.0, // OpenAI doesn't provide confidence
                                            is_final: true,
                                            speaker: None,
                                            timestamp: chrono::Utc::now(),
                                        };
                                        if transcript_tx.send(segment).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                                "error" => {
                                    if let Some(error) = event.error {
                                        tracing::error!("OpenAI Realtime error: {}", error.message);
                                    }
                                }
                                _ => {
                                    tracing::trace!("Received event: {}", event.event_type);
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        tracing::info!("OpenAI Realtime connection closed");
                        break;
                    }
                    Err(e) => {
                        tracing::error!("OpenAI Realtime WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok((audio_tx, transcript_rx))
    }
}

/// Generate a random WebSocket key
fn generate_ws_key() -> String {
    use base64::Engine;
    let mut key = [0u8; 16];
    getrandom::getrandom(&mut key).unwrap();
    base64::engine::general_purpose::STANDARD.encode(key)
}
