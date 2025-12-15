//! Audio Capture Module
//!
//! Handles real-time audio capture from system audio (loopback) for transcription.
//! Uses WASAPI on Windows for low-latency capture.
//! Supports app-specific capture (Zoom, Discord, Teams, etc.)

mod audio;
mod app_audio;
mod deepgram;
mod openai_realtime;
mod local_whisper;
mod transcript;

pub use audio::{AudioCapture, AudioCaptureState, AudioConfig};
pub use app_audio::{AudioSource, AudioDevice, CaptureApp, detect_running_apps, list_audio_devices, get_available_sources};
pub use deepgram::{DeepgramClient, DeepgramConfig};
pub use openai_realtime::OpenAIRealtimeClient;
pub use local_whisper::{LocalWhisperClient, LocalWhisperConfig, WhisperModel, WhisperStatus, WhisperModelStatus, check_whisper_status};
pub use transcript::{TranscriptSegment, TranscriptBuffer};
