//! Runtime Service
//!
//! Bridges the Dioxus UI with the async pipeline.
//! Manages the tokio runtime and pipeline lifecycle.

use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use anyhow::Result;

use crate::brain::{CopilotPipeline, PipelineConfig, PipelineEvent, FlashModelChoice};
use crate::deep::ModelChoice;
use crate::capture::AudioSource;
use crate::config::Settings;
use crate::flash::{FlashAnalysis, Bullet};

/// Commands from UI to runtime
#[derive(Debug, Clone)]
pub enum RuntimeCommand {
    Start,
    Stop,
    SetMode(String),
    SetAudioSource(AudioSource),
}

/// State shared between UI and runtime
#[derive(Debug, Clone, Default)]
pub struct SharedState {
    pub is_running: bool,
    pub transcript: String,
    pub flash: Option<FlashAnalysis>,
    pub deep_content: String,
    pub deep_streaming: bool,
    pub question: Option<String>,
    pub error: Option<String>,
    pub status: String,
}

/// Runtime service that manages the pipeline
pub struct RuntimeService {
    pipeline: Option<CopilotPipeline>,
    state: Arc<RwLock<SharedState>>,
    settings: Settings,
    command_rx: mpsc::Receiver<RuntimeCommand>,
}

impl RuntimeService {
    /// Create a new runtime service
    pub fn new(
        settings: Settings,
        state: Arc<RwLock<SharedState>>,
        command_rx: mpsc::Receiver<RuntimeCommand>,
    ) -> Self {
        Self {
            pipeline: None,
            state,
            settings,
            command_rx,
        }
    }

    /// Run the service (call from tokio runtime)
    pub async fn run(mut self) {
        while let Some(cmd) = self.command_rx.recv().await {
            match cmd {
                RuntimeCommand::Start => {
                    if let Err(e) = self.start_pipeline().await {
                        self.state.write().error = Some(e.to_string());
                        self.state.write().status = "Error".to_string();
                    }
                }
                RuntimeCommand::Stop => {
                    self.stop_pipeline();
                }
                RuntimeCommand::SetMode(mode) => {
                    if let Some(ref pipeline) = self.pipeline {
                        pipeline.set_context(&mode);
                    }
                }
                RuntimeCommand::SetAudioSource(_source) => {
                    // TODO: Implement audio source switching
                }
            }
        }
    }

    async fn start_pipeline(&mut self) -> Result<()> {
        // Load API keys from .env or settings
        let config = self.build_config();

        let mut pipeline = CopilotPipeline::new(config);

        // Subscribe to events
        let mut event_rx = pipeline.subscribe();
        let state = self.state.clone();

        // Spawn event listener
        tokio::spawn(async move {
            while let Ok(event) = event_rx.recv().await {
                let mut state = state.write();
                match event {
                    PipelineEvent::Started => {
                        state.is_running = true;
                        state.status = "Listening".to_string();
                        state.error = None;
                    }
                    PipelineEvent::Stopped => {
                        state.is_running = false;
                        state.status = "Stopped".to_string();
                    }
                    PipelineEvent::Transcript(text) => {
                        state.transcript = text;
                    }
                    PipelineEvent::FlashReady(flash) => {
                        state.flash = Some(flash);
                    }
                    PipelineEvent::DeepChunk(chunk) => {
                        state.deep_content.push_str(&chunk);
                        state.deep_streaming = true;
                    }
                    PipelineEvent::DeepComplete => {
                        state.deep_streaming = false;
                    }
                    PipelineEvent::QuestionReady(q) => {
                        state.question = Some(q);
                    }
                    PipelineEvent::Error(e) => {
                        state.error = Some(e);
                        state.status = "Error".to_string();
                    }
                }
            }
        });

        // Start the pipeline
        pipeline.start().await?;
        self.pipeline = Some(pipeline);

        Ok(())
    }

    fn stop_pipeline(&mut self) {
        if let Some(ref mut pipeline) = self.pipeline {
            pipeline.stop();
        }
        self.pipeline = None;

        let mut state = self.state.write();
        state.is_running = false;
        state.transcript.clear();
        state.flash = None;
        state.deep_content.clear();
        state.deep_streaming = false;
        state.question = None;
        state.status = "Stopped".to_string();
    }

    fn build_config(&self) -> PipelineConfig {
        // Try to load from .env first
        let deepgram_key = std::env::var("DEEPGRAM_API_KEY").ok()
            .or_else(|| self.settings.api_keys.deepgram.clone());

        let openai_key = std::env::var("OPENAI_API_KEY").ok()
            .or_else(|| self.settings.api_keys.openai.clone());

        let anthropic_key = std::env::var("ANTHROPIC_API_KEY").ok()
            .or_else(|| self.settings.api_keys.anthropic.clone());

        let google_key = std::env::var("GOOGLE_AI_API_KEY").ok()
            .or_else(|| self.settings.api_keys.google.clone());

        // Determine which models to use based on available keys
        let flash_model = if google_key.is_some() {
            FlashModelChoice::GeminiFlash
        } else {
            FlashModelChoice::GPT4oMini
        };

        let deep_model = if anthropic_key.is_some() {
            ModelChoice::ClaudeSonnet
        } else {
            ModelChoice::GPT4o
        };

        PipelineConfig {
            deepgram_key,
            openai_key,
            anthropic_key,
            google_key,
            flash_model,
            deep_model,
        }
    }
}

/// Handle to control the runtime from UI
#[derive(Clone)]
pub struct RuntimeHandle {
    command_tx: mpsc::Sender<RuntimeCommand>,
    state: Arc<RwLock<SharedState>>,
}

impl RuntimeHandle {
    /// Create a new runtime handle and service
    pub fn new(settings: Settings) -> (Self, RuntimeService) {
        let (command_tx, command_rx) = mpsc::channel(32);
        let state = Arc::new(RwLock::new(SharedState::default()));

        let service = RuntimeService::new(settings, state.clone(), command_rx);
        let handle = RuntimeHandle { command_tx, state };

        (handle, service)
    }

    /// Start listening
    pub fn start(&self) {
        let _ = self.command_tx.try_send(RuntimeCommand::Start);
    }

    /// Stop listening
    pub fn stop(&self) {
        let _ = self.command_tx.try_send(RuntimeCommand::Stop);
    }

    /// Set the mode
    pub fn set_mode(&self, mode: &str) {
        let _ = self.command_tx.try_send(RuntimeCommand::SetMode(mode.to_string()));
    }

    /// Set audio source
    pub fn set_audio_source(&self, source: AudioSource) {
        let _ = self.command_tx.try_send(RuntimeCommand::SetAudioSource(source));
    }

    /// Get current state
    pub fn state(&self) -> SharedState {
        self.state.read().clone()
    }

    /// Get state reference for polling
    pub fn state_ref(&self) -> Arc<RwLock<SharedState>> {
        self.state.clone()
    }
}
