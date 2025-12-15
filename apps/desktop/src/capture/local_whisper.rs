//! Local Whisper STT
//!
//! Offline speech-to-text using whisper.cpp via whisper-rs.
//! No API costs, works without internet, runs on CPU/GPU.

use anyhow::{anyhow, Result};
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

use super::transcript::TranscriptSegment;

/// Default model to use (smaller = faster, larger = more accurate)
const DEFAULT_MODEL: &str = "base.en";

/// Whisper model sizes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WhisperModel {
    /// Tiny - ~75MB, fastest, lower accuracy
    Tiny,
    /// Base - ~142MB, good balance
    Base,
    /// Small - ~466MB, better accuracy
    Small,
    /// Medium - ~1.5GB, high accuracy
    Medium,
    /// Large - ~2.9GB, highest accuracy (requires more VRAM)
    Large,
}

impl WhisperModel {
    pub fn filename(&self) -> &'static str {
        match self {
            WhisperModel::Tiny => "ggml-tiny.en.bin",
            WhisperModel::Base => "ggml-base.en.bin",
            WhisperModel::Small => "ggml-small.en.bin",
            WhisperModel::Medium => "ggml-medium.en.bin",
            WhisperModel::Large => "ggml-large-v3.bin",
        }
    }

    pub fn size_mb(&self) -> u32 {
        match self {
            WhisperModel::Tiny => 75,
            WhisperModel::Base => 142,
            WhisperModel::Small => 466,
            WhisperModel::Medium => 1500,
            WhisperModel::Large => 2900,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "tiny" => WhisperModel::Tiny,
            "base" => WhisperModel::Base,
            "small" => WhisperModel::Small,
            "medium" => WhisperModel::Medium,
            "large" | "large-v3" => WhisperModel::Large,
            _ => WhisperModel::Base,
        }
    }
}

/// Local Whisper client configuration
#[derive(Debug, Clone)]
pub struct LocalWhisperConfig {
    /// Model to use
    pub model: WhisperModel,
    /// Language (empty = auto-detect, "en" = English only)
    pub language: String,
    /// Number of threads to use (0 = auto)
    pub threads: u32,
    /// Whether to translate to English
    pub translate: bool,
    /// Maximum segment length in milliseconds
    pub max_segment_len: u32,
    /// Use GPU acceleration if available
    pub use_gpu: bool,
}

impl Default for LocalWhisperConfig {
    fn default() -> Self {
        Self {
            model: WhisperModel::Base,
            language: "en".to_string(),
            threads: 0, // Auto-detect
            translate: false,
            max_segment_len: 5000, // 5 seconds
            use_gpu: true,
        }
    }
}

/// Whisper status for UI
#[derive(Debug, Clone)]
pub enum WhisperStatus {
    /// Model not downloaded
    NotDownloaded,
    /// Currently downloading
    Downloading(u8), // percentage
    /// Model loading
    Loading,
    /// Ready to transcribe
    Ready,
    /// Currently transcribing
    Transcribing,
    /// Error occurred
    Error(String),
}

/// Local Whisper STT client
///
/// Note: This is a simplified implementation that processes audio in chunks.
/// For production, consider using whisper-rs with proper VAD (Voice Activity Detection).
pub struct LocalWhisperClient {
    config: LocalWhisperConfig,
    model_path: Option<PathBuf>,
    status: Arc<Mutex<WhisperStatus>>,
}

impl LocalWhisperClient {
    /// Create a new Local Whisper client
    pub fn new(config: LocalWhisperConfig) -> Self {
        Self {
            config,
            model_path: None,
            status: Arc::new(Mutex::new(WhisperStatus::NotDownloaded)),
        }
    }

    /// Get the models directory
    pub fn models_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("voice-copilot")
            .join("whisper-models")
    }

    /// Get the model file path
    pub fn model_path(&self) -> PathBuf {
        Self::models_dir().join(self.config.model.filename())
    }

    /// Check if model is downloaded
    pub fn is_model_downloaded(&self) -> bool {
        self.model_path().exists()
    }

    /// Get current status
    pub fn status(&self) -> WhisperStatus {
        self.status.lock().clone()
    }

    /// Download model from Hugging Face
    pub async fn download_model(&self, progress_callback: impl Fn(u8) + Send + 'static) -> Result<()> {
        let model_dir = Self::models_dir();
        std::fs::create_dir_all(&model_dir)?;

        let model_path = self.model_path();
        if model_path.exists() {
            return Ok(());
        }

        *self.status.lock() = WhisperStatus::Downloading(0);

        // Hugging Face model URLs
        let url = format!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}",
            self.config.model.filename()
        );

        tracing::info!("Downloading Whisper model from: {}", url);

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to download model: HTTP {}", response.status()));
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;

        let mut file = tokio::fs::File::create(&model_path).await?;
        let mut stream = response.bytes_stream();

        use futures::StreamExt;
        use tokio::io::AsyncWriteExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;

            downloaded += chunk.len() as u64;
            if total_size > 0 {
                let progress = ((downloaded as f64 / total_size as f64) * 100.0) as u8;
                *self.status.lock() = WhisperStatus::Downloading(progress);
                progress_callback(progress);
            }
        }

        file.flush().await?;
        *self.status.lock() = WhisperStatus::NotDownloaded; // Will be Loading when init is called

        tracing::info!("Downloaded Whisper model to: {:?}", model_path);
        Ok(())
    }

    /// Initialize the model (load into memory)
    ///
    /// Note: Actual whisper-rs initialization would happen here.
    /// This is a placeholder that simulates the interface.
    pub async fn init(&mut self) -> Result<()> {
        let model_path = self.model_path();
        if !model_path.exists() {
            return Err(anyhow!("Model not downloaded. Call download_model() first."));
        }

        *self.status.lock() = WhisperStatus::Loading;

        // In a real implementation, we would load the model:
        // let ctx = WhisperContext::new(&model_path.to_string_lossy())?;

        self.model_path = Some(model_path);
        *self.status.lock() = WhisperStatus::Ready;

        Ok(())
    }

    /// Start streaming transcription
    ///
    /// Returns:
    /// - A sender to push audio samples (f32, 16kHz)
    /// - A receiver to get transcript segments
    pub async fn start_streaming(&self) -> Result<(mpsc::Sender<Vec<f32>>, mpsc::Receiver<TranscriptSegment>)> {
        if !matches!(*self.status.lock(), WhisperStatus::Ready) {
            return Err(anyhow!("Model not initialized. Call init() first."));
        }

        let (audio_tx, mut audio_rx) = mpsc::channel::<Vec<f32>>(100);
        let (transcript_tx, transcript_rx) = mpsc::channel::<TranscriptSegment>(100);

        let status = self.status.clone();
        let config = self.config.clone();
        let model_path = self.model_path.clone();

        // Spawn transcription task
        tokio::spawn(async move {
            let mut audio_buffer: Vec<f32> = Vec::new();
            let sample_rate = 16000;
            let chunk_samples = sample_rate * 3; // Process every 3 seconds

            while let Some(samples) = audio_rx.recv().await {
                audio_buffer.extend(samples);

                // Process when we have enough audio
                if audio_buffer.len() >= chunk_samples {
                    *status.lock() = WhisperStatus::Transcribing;

                    // In a real implementation, we would call whisper:
                    // let mut state = ctx.create_state()?;
                    // let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
                    // params.set_language(Some(&config.language));
                    // state.full(params, &audio_buffer)?;
                    // let text = state.full_get_segment_text(0)?;

                    // For now, emit a placeholder segment
                    // This would be replaced with actual whisper transcription
                    let segment = TranscriptSegment {
                        text: "[Local Whisper - model integration pending]".to_string(),
                        confidence: 0.0,
                        is_final: true,
                        speaker: None,
                        timestamp: chrono::Utc::now(),
                    };

                    if transcript_tx.send(segment).await.is_err() {
                        break;
                    }

                    // Keep last second for overlap
                    let overlap = sample_rate;
                    if audio_buffer.len() > overlap {
                        audio_buffer = audio_buffer[audio_buffer.len() - overlap..].to_vec();
                    }

                    *status.lock() = WhisperStatus::Ready;
                }
            }
        });

        Ok((audio_tx, transcript_rx))
    }

    /// Transcribe a complete audio file
    pub async fn transcribe_file(&self, _audio_path: &str) -> Result<String> {
        if !matches!(*self.status.lock(), WhisperStatus::Ready) {
            return Err(anyhow!("Model not initialized. Call init() first."));
        }

        // In a real implementation:
        // let samples = load_audio_file(audio_path)?;
        // let mut state = ctx.create_state()?;
        // state.full(params, &samples)?;
        // let text = collect_all_segments(&state);

        Ok("[Local Whisper transcription - model integration pending]".to_string())
    }
}

/// Check if whisper models are available
pub fn check_whisper_status() -> WhisperModelStatus {
    let models_dir = LocalWhisperClient::models_dir();

    if !models_dir.exists() {
        return WhisperModelStatus::NoneDownloaded;
    }

    let mut available_models = Vec::new();

    for model in [WhisperModel::Tiny, WhisperModel::Base, WhisperModel::Small, WhisperModel::Medium, WhisperModel::Large] {
        let path = models_dir.join(model.filename());
        if path.exists() {
            available_models.push(model);
        }
    }

    if available_models.is_empty() {
        WhisperModelStatus::NoneDownloaded
    } else {
        WhisperModelStatus::Available(available_models)
    }
}

/// Status of whisper models
#[derive(Debug, Clone)]
pub enum WhisperModelStatus {
    /// No models downloaded
    NoneDownloaded,
    /// Models available
    Available(Vec<WhisperModel>),
}

impl WhisperModelStatus {
    pub fn is_available(&self) -> bool {
        matches!(self, WhisperModelStatus::Available(_))
    }
}

/// Get download URL for a model
pub fn get_model_download_url(model: WhisperModel) -> String {
    format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}",
        model.filename()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_path() {
        let client = LocalWhisperClient::new(LocalWhisperConfig::default());
        let path = client.model_path();
        assert!(path.to_string_lossy().contains("ggml-base.en.bin"));
    }

    #[test]
    fn test_model_from_str() {
        assert_eq!(WhisperModel::from_str("tiny"), WhisperModel::Tiny);
        assert_eq!(WhisperModel::from_str("BASE"), WhisperModel::Base);
        assert_eq!(WhisperModel::from_str("large"), WhisperModel::Large);
        assert_eq!(WhisperModel::from_str("unknown"), WhisperModel::Base);
    }
}
