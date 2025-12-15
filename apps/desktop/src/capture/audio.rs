//! System Audio Capture
//!
//! Captures audio from the system output (loopback) to transcribe what others are saying.
//! On Windows, uses WASAPI loopback mode.
//! On macOS, requires a virtual audio device like BlackHole.

use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, SampleFormat, SampleRate, Stream, StreamConfig};
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Audio capture configuration
#[derive(Debug, Clone)]
pub struct AudioConfig {
    /// Sample rate for capture (16000 Hz is optimal for STT)
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u16,
    /// Buffer size in samples
    pub buffer_size: usize,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            buffer_size: 1024,
        }
    }
}

/// Current state of audio capture
#[derive(Debug, Clone, Default, PartialEq)]
pub enum AudioCaptureState {
    #[default]
    Stopped,
    Starting,
    Running,
    Error(String),
}

/// Audio capture handle
pub struct AudioCapture {
    config: AudioConfig,
    state: Arc<Mutex<AudioCaptureState>>,
    stream: Option<Stream>,
    audio_tx: Option<mpsc::Sender<Vec<f32>>>,
}

impl AudioCapture {
    /// Create a new audio capture instance
    pub fn new(config: AudioConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(AudioCaptureState::Stopped)),
            stream: None,
            audio_tx: None,
        }
    }

    /// Get the current capture state
    pub fn state(&self) -> AudioCaptureState {
        self.state.lock().clone()
    }

    /// List available audio devices
    pub fn list_devices() -> Result<Vec<String>> {
        let host = cpal::default_host();
        let mut devices = Vec::new();

        // List output devices (for loopback capture)
        if let Ok(output_devices) = host.output_devices() {
            for device in output_devices {
                if let Ok(name) = device.name() {
                    devices.push(format!("[Output/Loopback] {}", name));
                }
            }
        }

        // List input devices (microphones)
        if let Ok(input_devices) = host.input_devices() {
            for device in input_devices {
                if let Ok(name) = device.name() {
                    devices.push(format!("[Input] {}", name));
                }
            }
        }

        Ok(devices)
    }

    /// Get the default loopback device for capturing system audio
    #[cfg(target_os = "windows")]
    pub fn get_loopback_device() -> Result<Device> {
        let host = cpal::default_host();

        // On Windows, we need to use WASAPI loopback
        // The default output device can be opened in loopback mode
        host.default_output_device()
            .ok_or_else(|| anyhow!("No default output device found"))
    }

    #[cfg(not(target_os = "windows"))]
    pub fn get_loopback_device() -> Result<Device> {
        let host = cpal::default_host();

        // On macOS/Linux, loopback requires virtual audio device
        // User must configure BlackHole or similar
        host.default_input_device()
            .ok_or_else(|| anyhow!("No default input device found. On macOS, install BlackHole for system audio capture."))
    }

    /// Start capturing audio
    ///
    /// Returns a channel receiver that will receive audio chunks
    pub fn start(&mut self) -> Result<mpsc::Receiver<Vec<f32>>> {
        *self.state.lock() = AudioCaptureState::Starting;

        let device = Self::get_loopback_device()?;
        tracing::info!("Using audio device: {:?}", device.name());

        // Get supported config
        let supported_config = device.default_output_config()?;
        tracing::info!("Default config: {:?}", supported_config);

        // Create stream config targeting 16kHz mono
        let stream_config = StreamConfig {
            channels: self.config.channels,
            sample_rate: SampleRate(self.config.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        // Channel for sending audio data
        let (tx, rx) = mpsc::channel::<Vec<f32>>(100);
        self.audio_tx = Some(tx.clone());

        let state = self.state.clone();
        let error_state = self.state.clone();

        // Build the input stream
        // Note: For true WASAPI loopback on Windows, we'd need to use the windows crate directly
        // cpal's loopback support varies by platform
        let stream = device.build_input_stream(
            &stream_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // Send audio chunk to processing pipeline
                let chunk: Vec<f32> = data.to_vec();
                if tx.blocking_send(chunk).is_err() {
                    tracing::warn!("Audio channel closed");
                }
            },
            move |err| {
                tracing::error!("Audio stream error: {}", err);
                *error_state.lock() = AudioCaptureState::Error(err.to_string());
            },
            None,
        )?;

        stream.play()?;
        self.stream = Some(stream);
        *state.lock() = AudioCaptureState::Running;

        tracing::info!("Audio capture started");
        Ok(rx)
    }

    /// Stop capturing audio
    pub fn stop(&mut self) {
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }
        self.audio_tx = None;
        *self.state.lock() = AudioCaptureState::Stopped;
        tracing::info!("Audio capture stopped");
    }
}

impl Drop for AudioCapture {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Resample audio from one sample rate to another
pub fn resample(input: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return input.to_vec();
    }

    use rubato::{FftFixedIn, Resampler};

    let ratio = to_rate as f64 / from_rate as f64;
    let chunk_size = 1024;

    let mut resampler = FftFixedIn::<f32>::new(
        from_rate as usize,
        to_rate as usize,
        chunk_size,
        2,
        1,
    ).expect("Failed to create resampler");

    let mut output = Vec::new();
    let input_vec = vec![input.to_vec()];

    if let Ok(resampled) = resampler.process(&input_vec, None) {
        if !resampled.is_empty() {
            output.extend_from_slice(&resampled[0]);
        }
    }

    output
}

/// Convert f32 samples to i16 for PCM encoding
pub fn f32_to_i16(samples: &[f32]) -> Vec<i16> {
    samples
        .iter()
        .map(|&s| {
            let clamped = s.clamp(-1.0, 1.0);
            (clamped * 32767.0) as i16
        })
        .collect()
}

/// Convert f32 samples to bytes (16-bit PCM, little-endian)
pub fn f32_to_pcm_bytes(samples: &[f32]) -> Vec<u8> {
    let i16_samples = f32_to_i16(samples);
    let mut bytes = Vec::with_capacity(i16_samples.len() * 2);

    for sample in i16_samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }

    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_devices() {
        let devices = AudioCapture::list_devices();
        assert!(devices.is_ok());
        println!("Available devices: {:?}", devices.unwrap());
    }

    #[test]
    fn test_f32_to_i16() {
        let samples = vec![0.0, 0.5, -0.5, 1.0, -1.0];
        let converted = f32_to_i16(&samples);
        assert_eq!(converted[0], 0);
        assert_eq!(converted[1], 16383); // ~0.5 * 32767
        assert_eq!(converted[2], -16383);
        assert_eq!(converted[3], 32767);
        assert_eq!(converted[4], -32767);
    }
}
