//! Copilot Pipeline
//!
//! The main orchestration engine that ties everything together:
//! Audio → STT → Flash → Deep → UI

use anyhow::Result;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::{mpsc, broadcast};

use crate::capture::{AudioCapture, AudioConfig, DeepgramClient, DeepgramConfig, TranscriptBuffer};
use crate::flash::{GeminiFlash, GPT4oMini, OllamaFlash, FlashAnalysis};
use crate::deep::{ModelRouter, ModelChoice};
use super::context::ConversationContext;
use super::intent::IntentAnalyzer;

/// Pipeline configuration
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Deepgram API key
    pub deepgram_key: Option<String>,
    /// OpenAI API key
    pub openai_key: Option<String>,
    /// Anthropic API key
    pub anthropic_key: Option<String>,
    /// Google AI API key
    pub google_key: Option<String>,
    /// Which flash model to use
    pub flash_model: FlashModelChoice,
    /// Which deep model to use
    pub deep_model: ModelChoice,
}

#[derive(Debug, Clone, Default)]
pub enum FlashModelChoice {
    #[default]
    GeminiFlash,
    GPT4oMini,
    /// Local Ollama (Llama 3.1 8B, Mistral, etc.)
    LocalOllama(String), // model name
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            deepgram_key: None,
            openai_key: None,
            anthropic_key: None,
            google_key: None,
            flash_model: FlashModelChoice::GeminiFlash,
            deep_model: ModelChoice::ClaudeSonnet,
        }
    }
}

/// Current state of the copilot
#[derive(Debug, Clone, Default)]
pub struct CopilotState {
    /// Is the pipeline running
    pub is_running: bool,
    /// Current transcript
    pub transcript: String,
    /// Flash analysis (quick bullets)
    pub flash: Option<FlashAnalysis>,
    /// Deep response (streaming in)
    pub deep_content: String,
    /// Is deep response still streaming
    pub deep_streaming: bool,
    /// Question to ask them
    pub question_to_ask: Option<String>,
    /// Current error (if any)
    pub error: Option<String>,
}

/// Events emitted by the pipeline
#[derive(Debug, Clone)]
pub enum PipelineEvent {
    /// New transcript segment
    Transcript(String),
    /// Flash analysis ready
    FlashReady(FlashAnalysis),
    /// Deep content chunk
    DeepChunk(String),
    /// Deep response complete
    DeepComplete,
    /// Question extracted
    QuestionReady(String),
    /// Error occurred
    Error(String),
    /// Pipeline started
    Started,
    /// Pipeline stopped
    Stopped,
}

/// The main copilot pipeline
pub struct CopilotPipeline {
    config: PipelineConfig,
    state: Arc<RwLock<CopilotState>>,
    context: Arc<RwLock<ConversationContext>>,
    transcript_buffer: Arc<TranscriptBuffer>,
    intent_analyzer: IntentAnalyzer,
    event_tx: broadcast::Sender<PipelineEvent>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl CopilotPipeline {
    /// Create a new pipeline
    pub fn new(config: PipelineConfig) -> Self {
        let (event_tx, _) = broadcast::channel(100);

        Self {
            config,
            state: Arc::new(RwLock::new(CopilotState::default())),
            context: Arc::new(RwLock::new(ConversationContext::default())),
            transcript_buffer: Arc::new(TranscriptBuffer::default()),
            intent_analyzer: IntentAnalyzer::new(),
            event_tx,
            shutdown_tx: None,
        }
    }

    /// Subscribe to pipeline events
    pub fn subscribe(&self) -> broadcast::Receiver<PipelineEvent> {
        self.event_tx.subscribe()
    }

    /// Get current state
    pub fn state(&self) -> CopilotState {
        self.state.read().clone()
    }

    /// Set conversation context/mode
    pub fn set_context(&self, context: impl Into<String>) {
        self.context.write().set_mode_context(context);
    }

    /// Start the pipeline
    pub async fn start(&mut self) -> Result<()> {
        if self.state.read().is_running {
            return Ok(());
        }

        // Validate configuration
        if self.config.deepgram_key.is_none() && self.config.openai_key.is_none() {
            return Err(anyhow::anyhow!("No STT API key configured"));
        }

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Start audio capture
        let mut audio_capture = AudioCapture::new(AudioConfig::default());
        let audio_rx = audio_capture.start()?;

        // Start STT
        let deepgram = DeepgramClient::new(
            self.config.deepgram_key.clone().unwrap_or_default()
        );
        let (audio_tx, mut transcript_rx) = deepgram
            .start_streaming(DeepgramConfig::default())
            .await?;

        // Update state
        self.state.write().is_running = true;
        let _ = self.event_tx.send(PipelineEvent::Started);

        // Spawn audio forwarding task
        let audio_tx_clone = audio_tx.clone();
        tokio::spawn(async move {
            let mut audio_rx = audio_rx;
            while let Some(samples) = audio_rx.recv().await {
                // Convert to PCM bytes
                let bytes = crate::capture::audio::f32_to_pcm_bytes(&samples);
                if audio_tx_clone.send(bytes).await.is_err() {
                    break;
                }
            }
        });

        // Spawn transcript processing task
        let state = self.state.clone();
        let context = self.context.clone();
        let transcript_buffer = self.transcript_buffer.clone();
        let event_tx = self.event_tx.clone();
        let config = self.config.clone();
        let intent_analyzer = IntentAnalyzer::new();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(segment) = transcript_rx.recv() => {
                        // Add to buffer
                        transcript_buffer.add(segment.clone());

                        // Update state
                        let current_text = transcript_buffer.get_current_text();
                        state.write().transcript = current_text.clone();

                        // Emit event
                        let _ = event_tx.send(PipelineEvent::Transcript(segment.text.clone()));

                        // If final segment, trigger AI analysis
                        if segment.is_final && !segment.text.is_empty() {
                            // Add to conversation context
                            let intent = intent_analyzer.analyze(&segment.text);
                            context.write().add_their_turn(&segment.text, Some(format!("{:?}", intent.category)));

                            // Trigger Flash analysis
                            let flash_result = run_flash_analysis(
                                &config,
                                &segment.text,
                                &context.read().get_full_context(),
                            ).await;

                            if let Ok(flash) = flash_result {
                                state.write().flash = Some(flash.clone());
                                let _ = event_tx.send(PipelineEvent::FlashReady(flash.clone()));

                                // Trigger Deep analysis
                                let bullets: Vec<String> = flash.bullets.iter().map(|b| b.point.clone()).collect();
                                let deep_result = run_deep_analysis(
                                    &config,
                                    &segment.text,
                                    &context.read().get_full_context(),
                                    &bullets,
                                    &context.read().get_history_string(),
                                    event_tx.clone(),
                                    state.clone(),
                                ).await;

                                if let Err(e) = deep_result {
                                    let _ = event_tx.send(PipelineEvent::Error(e.to_string()));
                                }
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the pipeline
    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.try_send(());
        }
        self.state.write().is_running = false;
        self.transcript_buffer.clear();
        let _ = self.event_tx.send(PipelineEvent::Stopped);
    }
}

/// Run flash analysis using configured model
async fn run_flash_analysis(
    config: &PipelineConfig,
    transcript: &str,
    context: &str,
) -> Result<FlashAnalysis> {
    match &config.flash_model {
        FlashModelChoice::GeminiFlash => {
            let client = GeminiFlash::new(config.google_key.clone().unwrap_or_default());
            client.analyze(transcript, context).await
        }
        FlashModelChoice::GPT4oMini => {
            let client = GPT4oMini::new(config.openai_key.clone().unwrap_or_default());
            client.analyze(transcript, context).await
        }
        FlashModelChoice::LocalOllama(model) => {
            let client = OllamaFlash::new().with_model(model.clone());
            client.analyze(transcript, context).await
        }
    }
}

/// Run deep analysis using configured model
async fn run_deep_analysis(
    config: &PipelineConfig,
    transcript: &str,
    context: &str,
    bullets: &[String],
    history: &str,
    event_tx: broadcast::Sender<PipelineEvent>,
    state: Arc<RwLock<CopilotState>>,
) -> Result<()> {
    let mut router = ModelRouter::new();

    if let Some(key) = &config.anthropic_key {
        router = router.with_claude(key.clone());
    }
    if let Some(key) = &config.openai_key {
        router = router.with_gpt4o(key.clone()).with_o1(key.clone());
    }

    router = router.with_default(config.deep_model.clone());

    state.write().deep_streaming = true;
    state.write().deep_content.clear();

    let mut stream = router
        .analyze_streaming(transcript, context, bullets, history, config.deep_model.clone())
        .await?;

    while let Some(chunk) = stream.receiver.recv().await {
        match chunk {
            crate::deep::streaming::StreamChunk::Content(text) => {
                state.write().deep_content.push_str(&text);
                let _ = event_tx.send(PipelineEvent::DeepChunk(text));
            }
            crate::deep::streaming::StreamChunk::Question(q) => {
                state.write().question_to_ask = Some(q.clone());
                let _ = event_tx.send(PipelineEvent::QuestionReady(q));
            }
            crate::deep::streaming::StreamChunk::Done => {
                state.write().deep_streaming = false;
                let _ = event_tx.send(PipelineEvent::DeepComplete);
                break;
            }
            crate::deep::streaming::StreamChunk::Error(e) => {
                state.write().deep_streaming = false;
                state.write().error = Some(e.clone());
                let _ = event_tx.send(PipelineEvent::Error(e));
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
