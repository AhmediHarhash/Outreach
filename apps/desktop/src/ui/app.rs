//! Main Dioxus Application
//!
//! The root component that manages:
//! - Application state
//! - Window configuration
//! - Component routing
//! - Audio source selection
//! - Flexible UI modes
//! - Pipeline integration

use dioxus::prelude::*;
use std::sync::Arc;
use parking_lot::RwLock;

use crate::brain::{CopilotPipeline, PipelineConfig, CopilotState as PipelineCopilotState, PipelineEvent, FlashModelChoice};
use crate::deep::ModelChoice;
use crate::flash::{FlashAnalysis, Bullet as FlashBullet};
use crate::capture::{AudioCaptureState, AudioSource, CaptureApp, get_available_sources, detect_running_apps};
use crate::config::Settings;
use super::runtime::SharedState;

/// UI display mode
#[derive(Debug, Clone, Default, PartialEq)]
pub enum UIMode {
    #[default]
    FullWindow,
    Overlay,
    Minimized,
}

impl UIMode {
    pub fn label(&self) -> &'static str {
        match self {
            UIMode::FullWindow => "Full Window",
            UIMode::Overlay => "Overlay",
            UIMode::Minimized => "Minimized",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            UIMode::FullWindow => "🪟",
            UIMode::Overlay => "📌",
            UIMode::Minimized => "➖",
        }
    }
}

/// Global application state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Whether the copilot is actively listening
    pub is_listening: bool,
    /// Current mode (sales, interview, technical)
    pub mode: CopilotMode,
    /// Current transcript from the other person
    pub transcript: String,
    /// Flash response (quick bullets)
    pub flash_response: Option<FlashResponse>,
    /// Deep response (detailed answer, streams in)
    pub deep_response: Option<DeepResponse>,
    /// Connection status
    pub status: ConnectionStatus,
    /// Selected audio source
    pub audio_source: AudioSource,
    /// Available audio sources
    pub available_sources: Vec<AudioSource>,
    /// UI display mode
    pub ui_mode: UIMode,
    /// Whether settings panel is open
    pub settings_open: bool,
    /// Whether audio source picker is open
    pub source_picker_open: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum CopilotMode {
    #[default]
    Sales,
    Interview,
    Technical,
    General,
}

impl CopilotMode {
    pub fn label(&self) -> &'static str {
        match self {
            CopilotMode::Sales => "Sales Call",
            CopilotMode::Interview => "Interview",
            CopilotMode::Technical => "Technical",
            CopilotMode::General => "General",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FlashResponse {
    pub summary: String,
    pub bullets: Vec<Bullet>,
    pub response_type: String,
}

#[derive(Debug, Clone)]
pub struct Bullet {
    pub point: String,
    pub priority: u8,
}

#[derive(Debug, Clone, Default)]
pub struct DeepResponse {
    pub content: String,
    pub is_streaming: bool,
    pub question_to_ask: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            is_listening: false,
            mode: CopilotMode::default(),
            transcript: String::new(),
            flash_response: None,
            deep_response: None,
            status: ConnectionStatus::default(),
            audio_source: AudioSource::SystemDefault,
            available_sources: get_available_sources(),
            ui_mode: UIMode::default(),
            settings_open: false,
            source_picker_open: false,
        }
    }
}

/// Launch the Dioxus desktop application
pub fn launch_app() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("Voice Copilot")
                .with_inner_size(dioxus::desktop::LogicalSize::new(420.0, 600.0))
                .with_min_inner_size(dioxus::desktop::LogicalSize::new(380.0, 400.0))
                .with_always_on_top(true)
                .with_decorations(true)
                .with_transparent(false)
        )
        .with_custom_head(r#"
            <style>
                :root {
                    --bg-primary: #0f0f0f;
                    --bg-secondary: #1a1a1a;
                    --bg-tertiary: #252525;
                    --text-primary: #ffffff;
                    --text-secondary: #a0a0a0;
                    --accent-blue: #3b82f6;
                    --accent-green: #22c55e;
                    --accent-yellow: #eab308;
                    --accent-red: #ef4444;
                    --border-color: #333333;
                }

                * {
                    margin: 0;
                    padding: 0;
                    box-sizing: border-box;
                }

                body {
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                    background: var(--bg-primary);
                    color: var(--text-primary);
                    font-size: 14px;
                    line-height: 1.5;
                    overflow: hidden;
                }

                .app-container {
                    display: flex;
                    flex-direction: column;
                    height: 100vh;
                    padding: 12px;
                    gap: 12px;
                }

                .status-bar {
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    padding: 8px 12px;
                    background: var(--bg-secondary);
                    border-radius: 8px;
                    border: 1px solid var(--border-color);
                }

                .status-indicator {
                    display: flex;
                    align-items: center;
                    gap: 8px;
                }

                .status-dot {
                    width: 8px;
                    height: 8px;
                    border-radius: 50%;
                    background: var(--accent-red);
                }

                .status-dot.connected {
                    background: var(--accent-green);
                    animation: pulse 2s infinite;
                }

                @keyframes pulse {
                    0%, 100% { opacity: 1; }
                    50% { opacity: 0.5; }
                }

                .mode-selector {
                    display: flex;
                    gap: 4px;
                }

                .mode-btn {
                    padding: 4px 12px;
                    background: transparent;
                    border: 1px solid var(--border-color);
                    border-radius: 4px;
                    color: var(--text-secondary);
                    cursor: pointer;
                    font-size: 12px;
                    transition: all 0.2s;
                }

                .mode-btn:hover {
                    background: var(--bg-tertiary);
                }

                .mode-btn.active {
                    background: var(--accent-blue);
                    border-color: var(--accent-blue);
                    color: white;
                }

                .transcript-section {
                    background: var(--bg-secondary);
                    border-radius: 8px;
                    border: 1px solid var(--border-color);
                    padding: 12px;
                }

                .transcript-label {
                    display: flex;
                    align-items: center;
                    gap: 6px;
                    color: var(--text-secondary);
                    font-size: 12px;
                    margin-bottom: 8px;
                }

                .transcript-text {
                    color: var(--text-primary);
                    font-style: italic;
                    min-height: 40px;
                }

                .flash-section {
                    background: var(--bg-secondary);
                    border-radius: 8px;
                    border: 1px solid var(--accent-yellow);
                    border-left-width: 3px;
                    padding: 12px;
                }

                .flash-header {
                    display: flex;
                    align-items: center;
                    gap: 6px;
                    color: var(--accent-yellow);
                    font-size: 12px;
                    font-weight: 600;
                    margin-bottom: 8px;
                }

                .flash-summary {
                    color: var(--text-primary);
                    margin-bottom: 12px;
                }

                .bullet-list {
                    list-style: none;
                    display: flex;
                    flex-direction: column;
                    gap: 6px;
                }

                .bullet-item {
                    display: flex;
                    align-items: flex-start;
                    gap: 8px;
                    padding: 6px 8px;
                    background: var(--bg-tertiary);
                    border-radius: 4px;
                }

                .bullet-item.priority-1 {
                    background: rgba(34, 197, 94, 0.15);
                    border-left: 2px solid var(--accent-green);
                }

                .bullet-marker {
                    color: var(--accent-green);
                    font-weight: bold;
                }

                .deep-section {
                    flex: 1;
                    background: var(--bg-secondary);
                    border-radius: 8px;
                    border: 1px solid var(--accent-blue);
                    border-left-width: 3px;
                    padding: 12px;
                    overflow-y: auto;
                }

                .deep-header {
                    display: flex;
                    align-items: center;
                    gap: 6px;
                    color: var(--accent-blue);
                    font-size: 12px;
                    font-weight: 600;
                    margin-bottom: 8px;
                }

                .deep-content {
                    color: var(--text-primary);
                    white-space: pre-wrap;
                }

                .deep-content.streaming::after {
                    content: '|';
                    animation: blink 1s infinite;
                }

                @keyframes blink {
                    0%, 100% { opacity: 1; }
                    50% { opacity: 0; }
                }

                .question-back {
                    margin-top: 12px;
                    padding: 8px 12px;
                    background: rgba(59, 130, 246, 0.1);
                    border-radius: 4px;
                    border-left: 2px solid var(--accent-blue);
                }

                .question-label {
                    font-size: 11px;
                    color: var(--accent-blue);
                    margin-bottom: 4px;
                }

                .control-bar {
                    display: flex;
                    gap: 8px;
                }

                .listen-btn {
                    flex: 1;
                    padding: 12px;
                    background: var(--accent-green);
                    border: none;
                    border-radius: 8px;
                    color: white;
                    font-weight: 600;
                    cursor: pointer;
                    transition: all 0.2s;
                }

                .listen-btn:hover {
                    filter: brightness(1.1);
                }

                .listen-btn.listening {
                    background: var(--accent-red);
                }

                .settings-btn {
                    padding: 12px;
                    background: var(--bg-secondary);
                    border: 1px solid var(--border-color);
                    border-radius: 8px;
                    color: var(--text-secondary);
                    cursor: pointer;
                    transition: all 0.2s;
                }

                .settings-btn:hover {
                    background: var(--bg-tertiary);
                }

                .empty-state {
                    display: flex;
                    flex-direction: column;
                    align-items: center;
                    justify-content: center;
                    height: 100%;
                    color: var(--text-secondary);
                    text-align: center;
                    padding: 20px;
                }

                .empty-state-icon {
                    font-size: 48px;
                    margin-bottom: 12px;
                }

                /* Audio Source Picker */
                .source-picker {
                    background: var(--bg-secondary);
                    border-radius: 8px;
                    border: 1px solid var(--border-color);
                    padding: 12px;
                }

                .source-picker-header {
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    margin-bottom: 8px;
                }

                .source-picker-title {
                    font-size: 12px;
                    color: var(--text-secondary);
                    display: flex;
                    align-items: center;
                    gap: 6px;
                }

                .source-list {
                    display: flex;
                    flex-direction: column;
                    gap: 4px;
                    max-height: 200px;
                    overflow-y: auto;
                }

                .source-item {
                    display: flex;
                    align-items: center;
                    gap: 8px;
                    padding: 8px 12px;
                    background: var(--bg-tertiary);
                    border: 1px solid transparent;
                    border-radius: 6px;
                    cursor: pointer;
                    transition: all 0.2s;
                }

                .source-item:hover {
                    background: var(--bg-primary);
                    border-color: var(--border-color);
                }

                .source-item.selected {
                    background: rgba(59, 130, 246, 0.15);
                    border-color: var(--accent-blue);
                }

                .source-item.app {
                    border-left: 3px solid var(--accent-green);
                }

                .source-icon {
                    font-size: 18px;
                }

                .source-name {
                    flex: 1;
                    font-size: 13px;
                }

                .source-badge {
                    font-size: 10px;
                    padding: 2px 6px;
                    background: var(--accent-green);
                    color: white;
                    border-radius: 4px;
                }

                .refresh-btn {
                    padding: 4px 8px;
                    background: transparent;
                    border: 1px solid var(--border-color);
                    border-radius: 4px;
                    color: var(--text-secondary);
                    cursor: pointer;
                    font-size: 12px;
                }

                .refresh-btn:hover {
                    background: var(--bg-tertiary);
                }

                /* UI Mode Selector */
                .ui-mode-bar {
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    padding: 6px 12px;
                    background: var(--bg-secondary);
                    border-radius: 8px;
                    border: 1px solid var(--border-color);
                }

                .ui-mode-selector {
                    display: flex;
                    gap: 4px;
                }

                .ui-mode-btn {
                    padding: 4px 10px;
                    background: transparent;
                    border: 1px solid var(--border-color);
                    border-radius: 4px;
                    color: var(--text-secondary);
                    cursor: pointer;
                    font-size: 11px;
                    display: flex;
                    align-items: center;
                    gap: 4px;
                    transition: all 0.2s;
                }

                .ui-mode-btn:hover {
                    background: var(--bg-tertiary);
                }

                .ui-mode-btn.active {
                    background: var(--accent-blue);
                    border-color: var(--accent-blue);
                    color: white;
                }

                /* Selected source display */
                .selected-source {
                    display: flex;
                    align-items: center;
                    gap: 8px;
                    padding: 8px 12px;
                    background: var(--bg-secondary);
                    border-radius: 8px;
                    border: 1px solid var(--border-color);
                    cursor: pointer;
                    transition: all 0.2s;
                }

                .selected-source:hover {
                    border-color: var(--accent-blue);
                }

                .selected-source-icon {
                    font-size: 16px;
                }

                .selected-source-text {
                    flex: 1;
                }

                .selected-source-label {
                    font-size: 10px;
                    color: var(--text-secondary);
                }

                .selected-source-name {
                    font-size: 13px;
                    color: var(--text-primary);
                }

                .dropdown-arrow {
                    color: var(--text-secondary);
                    font-size: 10px;
                }

                /* Settings Panel Styles */
                .settings-overlay {
                    position: fixed;
                    top: 0;
                    left: 0;
                    right: 0;
                    bottom: 0;
                    background: rgba(0, 0, 0, 0.7);
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    z-index: 1000;
                }

                .settings-panel {
                    background: var(--bg-primary);
                    border-radius: 12px;
                    border: 1px solid var(--border-color);
                    width: 90%;
                    max-width: 450px;
                    max-height: 85vh;
                    overflow-y: auto;
                    padding: 20px;
                }

                .settings-header {
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    margin-bottom: 20px;
                    padding-bottom: 12px;
                    border-bottom: 1px solid var(--border-color);
                }

                .settings-header h2 {
                    font-size: 18px;
                    font-weight: 600;
                }

                .close-btn {
                    background: transparent;
                    border: none;
                    color: var(--text-secondary);
                    font-size: 20px;
                    cursor: pointer;
                    padding: 4px 8px;
                }

                .close-btn:hover {
                    color: var(--text-primary);
                }

                .settings-section {
                    margin-bottom: 24px;
                }

                .settings-section h3 {
                    font-size: 14px;
                    font-weight: 600;
                    color: var(--accent-blue);
                    margin-bottom: 12px;
                }

                .settings-hint {
                    font-size: 11px;
                    color: var(--text-secondary);
                    margin-bottom: 12px;
                }

                .setting-item {
                    display: flex;
                    flex-direction: column;
                    gap: 6px;
                    margin-bottom: 16px;
                }

                .setting-item label {
                    font-size: 12px;
                    color: var(--text-secondary);
                }

                .setting-item input,
                .setting-item select {
                    padding: 10px 12px;
                    background: var(--bg-secondary);
                    border: 1px solid var(--border-color);
                    border-radius: 6px;
                    color: var(--text-primary);
                    font-size: 13px;
                }

                .setting-item input:focus,
                .setting-item select:focus {
                    outline: none;
                    border-color: var(--accent-blue);
                }

                .key-status {
                    font-size: 10px;
                    padding: 2px 8px;
                    border-radius: 4px;
                    align-self: flex-start;
                }

                .key-status.ok {
                    background: rgba(34, 197, 94, 0.2);
                    color: var(--accent-green);
                }

                .key-status.optional {
                    background: rgba(234, 179, 8, 0.2);
                    color: var(--accent-yellow);
                }

                .key-status.error {
                    background: rgba(239, 68, 68, 0.2);
                    color: #ef4444;
                }

                .ollama-config {
                    margin-left: 16px;
                    padding-left: 12px;
                    border-left: 2px solid var(--accent-blue);
                }

                .ollama-info {
                    margin-left: 16px;
                    padding: 12px;
                    background: var(--bg-secondary);
                    border-radius: 8px;
                    margin-bottom: 16px;
                }

                .ollama-info p {
                    margin: 4px 0;
                }

                .ollama-info code {
                    font-family: monospace;
                    background: var(--bg-tertiary);
                    padding: 2px 6px;
                    border-radius: 4px;
                    font-size: 11px;
                }

                .settings-hint.warning {
                    color: var(--accent-yellow);
                }

                .shortcut-list {
                    display: flex;
                    flex-direction: column;
                    gap: 8px;
                }

                .shortcut-item {
                    display: flex;
                    align-items: center;
                    gap: 12px;
                    padding: 8px;
                    background: var(--bg-secondary);
                    border-radius: 6px;
                }

                .shortcut-key {
                    font-family: monospace;
                    font-size: 11px;
                    padding: 4px 8px;
                    background: var(--bg-tertiary);
                    border-radius: 4px;
                    color: var(--accent-blue);
                }

                .settings-footer {
                    display: flex;
                    justify-content: flex-end;
                    align-items: center;
                    gap: 12px;
                    margin-top: 20px;
                    padding-top: 16px;
                    border-top: 1px solid var(--border-color);
                }

                .save-message {
                    font-size: 12px;
                    color: var(--accent-green);
                }

                .save-btn {
                    padding: 10px 20px;
                    background: var(--accent-blue);
                    border: none;
                    border-radius: 6px;
                    color: white;
                    font-weight: 600;
                    cursor: pointer;
                    transition: all 0.2s;
                }

                .save-btn:hover {
                    filter: brightness(1.1);
                }

                .save-btn:disabled {
                    opacity: 0.6;
                    cursor: not-allowed;
                }

                /* Update Button Styles */
                .update-status {
                    display: flex;
                    align-items: center;
                    gap: 6px;
                    padding: 4px 10px;
                    border-radius: 4px;
                    font-size: 11px;
                    cursor: pointer;
                    transition: all 0.2s;
                }

                .update-status.checking {
                    background: var(--bg-tertiary);
                    color: var(--text-secondary);
                }

                .update-status.up-to-date {
                    background: rgba(34, 197, 94, 0.1);
                    color: var(--accent-green);
                }

                .update-status.up-to-date:hover {
                    background: rgba(34, 197, 94, 0.2);
                }

                .update-status.available {
                    background: rgba(59, 130, 246, 0.2);
                    color: var(--accent-blue);
                    border: 1px solid var(--accent-blue);
                }

                .update-status.available:hover {
                    background: rgba(59, 130, 246, 0.3);
                }

                .update-status.downloading {
                    background: var(--bg-tertiary);
                    color: var(--accent-yellow);
                }

                .update-status.error {
                    background: rgba(239, 68, 68, 0.1);
                    color: var(--text-secondary);
                }

                .update-icon {
                    font-size: 12px;
                }

                .update-icon.pulse {
                    animation: pulse-update 1.5s infinite;
                }

                @keyframes pulse-update {
                    0%, 100% { transform: scale(1); }
                    50% { transform: scale(1.2); }
                }

                /* Version bar at bottom of settings */
                .version-bar {
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    padding: 8px 12px;
                    background: var(--bg-tertiary);
                    border-radius: 6px;
                    margin-top: 16px;
                }

                .version-text {
                    font-size: 11px;
                    color: var(--text-secondary);
                }
            </style>
        "#.to_string());

    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(App);
}

/// Runtime handle stored in context
static RUNTIME: std::sync::OnceLock<super::runtime::RuntimeHandle> = std::sync::OnceLock::new();

/// Initialize runtime (call once at startup)
fn init_runtime() -> super::runtime::RuntimeHandle {
    let settings = Settings::load().unwrap_or_default();
    let (handle, service) = super::runtime::RuntimeHandle::new(settings);

    // Spawn the runtime service in a background thread with its own tokio runtime
    let service_handle = handle.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(service.run());
    });

    handle
}

/// Get or create the runtime handle
fn get_runtime() -> &'static super::runtime::RuntimeHandle {
    RUNTIME.get_or_init(init_runtime)
}

/// Root application component
#[component]
fn App() -> Element {
    // Global state
    let mut app_state = use_signal(AppState::default);

    // Get runtime handle
    let runtime = get_runtime();

    // Poll runtime state periodically
    let runtime_state = runtime.state_ref();
    use_future(move || {
        let runtime_state = runtime_state.clone();
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                let state = runtime_state.read().clone();

                // Update UI state from runtime state
                let mut ui_state = app_state.write();
                ui_state.is_listening = state.is_running;
                ui_state.transcript = state.transcript.clone();

                // Update flash response
                if let Some(flash) = &state.flash {
                    ui_state.flash_response = Some(FlashResponse {
                        summary: flash.summary.clone(),
                        bullets: flash.bullets.iter().map(|b| Bullet {
                            point: b.point.clone(),
                            priority: b.priority,
                        }).collect(),
                        response_type: flash.statement_type.label().to_string(),
                    });
                } else {
                    ui_state.flash_response = None;
                }

                // Update deep response
                if !state.deep_content.is_empty() || state.deep_streaming {
                    ui_state.deep_response = Some(DeepResponse {
                        content: state.deep_content.clone(),
                        is_streaming: state.deep_streaming,
                        question_to_ask: state.question.clone(),
                    });
                } else {
                    ui_state.deep_response = None;
                }

                // Update status
                ui_state.status = if state.is_running {
                    ConnectionStatus::Connected
                } else if state.error.is_some() {
                    ConnectionStatus::Error(state.error.clone().unwrap_or_default())
                } else {
                    ConnectionStatus::Disconnected
                };
            }
        }
    });

    // Toggle listening
    let toggle_listening = move |_| {
        let runtime = get_runtime();
        let is_listening = app_state.read().is_listening;

        if is_listening {
            runtime.stop();
        } else {
            app_state.write().status = ConnectionStatus::Connecting;
            app_state.write().source_picker_open = false;
            runtime.start();
        }
    };

    // Change mode
    let change_mode = move |mode: CopilotMode| {
        let runtime = get_runtime();
        app_state.write().mode = mode.clone();
        runtime.set_mode(mode.label());
    };

    // Change UI mode
    let change_ui_mode = move |ui_mode: UIMode| {
        app_state.write().ui_mode = ui_mode;
    };

    // Toggle source picker
    let toggle_source_picker = move |_| {
        let mut state = app_state.write();
        state.source_picker_open = !state.source_picker_open;
        if state.source_picker_open {
            // Refresh available sources when opening
            state.available_sources = get_available_sources();
        }
    };

    // Select audio source
    let select_source = move |source: AudioSource| {
        let runtime = get_runtime();
        let mut state = app_state.write();
        state.audio_source = source.clone();
        state.source_picker_open = false;
        runtime.set_audio_source(source);
    };

    // Refresh sources
    let refresh_sources = move |_| {
        app_state.write().available_sources = get_available_sources();
    };

    let state = app_state.read();

    // Get source icon
    let source_icon = match &state.audio_source {
        AudioSource::SystemDefault => "🔊",
        AudioSource::SpecificApp(app) => app.icon,
        AudioSource::Device(_) => "🎧",
    };

    rsx! {
        div { class: "app-container",
            // UI Mode Bar
            div { class: "ui-mode-bar",
                div { class: "status-indicator",
                    div {
                        class: if state.status == ConnectionStatus::Connected { "status-dot connected" } else { "status-dot" }
                    }
                    span {
                        {match &state.status {
                            ConnectionStatus::Disconnected => "Ready",
                            ConnectionStatus::Connecting => "Connecting...",
                            ConnectionStatus::Connected => "Listening",
                            ConnectionStatus::Error(_) => "Error",
                        }}
                    }
                    // Update button
                    super::update_button::UpdateButton {}
                }
                div { class: "ui-mode-selector",
                    button {
                        class: if state.ui_mode == UIMode::FullWindow { "ui-mode-btn active" } else { "ui-mode-btn" },
                        onclick: move |_| change_ui_mode(UIMode::FullWindow),
                        span { "🪟" }
                        span { "Full" }
                    }
                    button {
                        class: if state.ui_mode == UIMode::Overlay { "ui-mode-btn active" } else { "ui-mode-btn" },
                        onclick: move |_| change_ui_mode(UIMode::Overlay),
                        span { "📌" }
                        span { "Overlay" }
                    }
                    button {
                        class: if state.ui_mode == UIMode::Minimized { "ui-mode-btn active" } else { "ui-mode-btn" },
                        onclick: move |_| change_ui_mode(UIMode::Minimized),
                        span { "➖" }
                        span { "Mini" }
                    }
                }
            }

            // Audio Source Selector (click to expand)
            div { class: "selected-source", onclick: toggle_source_picker,
                span { class: "selected-source-icon", "{source_icon}" }
                div { class: "selected-source-text",
                    div { class: "selected-source-label", "Audio Source" }
                    div { class: "selected-source-name", "{state.audio_source.display_name()}" }
                }
                span { class: "dropdown-arrow", {if state.source_picker_open { "▲" } else { "▼" }} }
            }

            // Source Picker (expanded)
            if state.source_picker_open {
                div { class: "source-picker",
                    div { class: "source-picker-header",
                        div { class: "source-picker-title",
                            span { "🎯" }
                            span { "Select Audio Source" }
                        }
                        button { class: "refresh-btn", onclick: refresh_sources, "🔄 Refresh" }
                    }
                    div { class: "source-list",
                        for source in state.available_sources.iter() {
                            {
                                let source_clone = source.clone();
                                let is_selected = state.audio_source == *source;
                                let is_app = matches!(source, AudioSource::SpecificApp(_));
                                let display_name = source.display_name();
                                let source_for_click = source.clone();
                                rsx! {
                                    div {
                                        class: format!(
                                            "source-item {} {}",
                                            if is_selected { "selected" } else { "" },
                                            if is_app { "app" } else { "" }
                                        ),
                                        onclick: move |_| select_source(source_for_click.clone()),
                                        span { class: "source-name", "{display_name}" }
                                        if is_app {
                                            span { class: "source-badge", "RUNNING" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Mode Selector
            div { class: "status-bar",
                span { style: "font-size: 12px; color: var(--text-secondary);", "Mode:" }
                div { class: "mode-selector",
                    button {
                        class: if state.mode == CopilotMode::Sales { "mode-btn active" } else { "mode-btn" },
                        onclick: move |_| change_mode(CopilotMode::Sales),
                        "Sales"
                    }
                    button {
                        class: if state.mode == CopilotMode::Interview { "mode-btn active" } else { "mode-btn" },
                        onclick: move |_| change_mode(CopilotMode::Interview),
                        "Interview"
                    }
                    button {
                        class: if state.mode == CopilotMode::Technical { "mode-btn active" } else { "mode-btn" },
                        onclick: move |_| change_mode(CopilotMode::Technical),
                        "Technical"
                    }
                }
            }

            // Transcript Section
            div { class: "transcript-section",
                div { class: "transcript-label",
                    span { "🎤" }
                    span { "They said:" }
                }
                div { class: "transcript-text",
                    {if state.transcript.is_empty() {
                        "Waiting for speech..."
                    } else {
                        state.transcript.as_str()
                    }}
                }
            }

            // Flash Response (Quick Bullets)
            if let Some(flash) = &state.flash_response {
                div { class: "flash-section",
                    div { class: "flash-header",
                        span { "⚡" }
                        span { "QUICK RESPONSE" }
                    }
                    div { class: "flash-summary", "{flash.summary}" }
                    ul { class: "bullet-list",
                        for (idx, bullet) in flash.bullets.iter().enumerate() {
                            li {
                                class: if bullet.priority == 1 { "bullet-item priority-1" } else { "bullet-item" },
                                key: "{idx}",
                                span { class: "bullet-marker",
                                    {if bullet.priority == 1 { "★" } else { "•" }}
                                }
                                span { "{bullet.point}" }
                            }
                        }
                    }
                }
            }

            // Deep Response (Detailed Answer)
            if let Some(deep) = &state.deep_response {
                div { class: "deep-section",
                    div { class: "deep-header",
                        span { "🧠" }
                        span { "DETAILED ANSWER" }
                    }
                    div {
                        class: if deep.is_streaming { "deep-content streaming" } else { "deep-content" },
                        "{deep.content}"
                    }
                    if let Some(question) = &deep.question_to_ask {
                        div { class: "question-back",
                            div { class: "question-label", "🔄 ASK THEM:" }
                            div { "{question}" }
                        }
                    }
                }
            }

            // Empty State
            if state.flash_response.is_none() && state.deep_response.is_none() && !state.is_listening {
                div { class: "empty-state",
                    div { class: "empty-state-icon", "🎯" }
                    div { "Select an audio source and click Start" }
                    div { style: "font-size: 12px; margin-top: 8px;",
                        "Pick Zoom, Discord, Teams, or any app to capture only that audio"
                    }
                }
            }

            // Control Bar
            div { class: "control-bar",
                button {
                    class: if state.is_listening { "listen-btn listening" } else { "listen-btn" },
                    onclick: toggle_listening,
                    {if state.is_listening { "⏹ Stop Listening" } else { "▶ Start Listening" }}
                }
                button {
                    class: "settings-btn",
                    onclick: move |_| app_state.write().settings_open = true,
                    "⚙️"
                }
            }

            // Settings Panel
            super::settings::SettingsPanel {
                is_open: state.settings_open,
                on_close: move |_| app_state.write().settings_open = false,
            }
        }
    }
}
