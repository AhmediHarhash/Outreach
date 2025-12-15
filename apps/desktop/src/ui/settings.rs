//! Settings Panel
//!
//! Configuration UI for API keys, audio devices, and preferences.

use dioxus::prelude::*;
use crate::config::{Settings, ApiKeys};
use crate::updater::CURRENT_VERSION;

/// Settings panel state
#[derive(Debug, Clone, Default)]
pub struct SettingsState {
    pub openai_key: String,
    pub anthropic_key: String,
    pub google_key: String,
    pub deepgram_key: String,
    pub flash_model: String,
    pub deep_model: String,
    pub ollama_model: String,
    pub ollama_status: OllamaStatusUI,
    pub is_saving: bool,
    pub save_message: Option<String>,
}

/// Ollama status for UI display
#[derive(Debug, Clone, Default)]
pub struct OllamaStatusUI {
    pub available: bool,
    pub models: Vec<String>,
    pub message: String,
}

impl SettingsState {
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            openai_key: settings.api_keys.openai.clone().unwrap_or_default(),
            anthropic_key: settings.api_keys.anthropic.clone().unwrap_or_default(),
            google_key: settings.api_keys.google.clone().unwrap_or_default(),
            deepgram_key: settings.api_keys.deepgram.clone().unwrap_or_default(),
            flash_model: format!("{:?}", settings.models.flash_model),
            deep_model: format!("{:?}", settings.models.deep_model),
            ollama_model: "llama3.1:8b".to_string(),
            ollama_status: OllamaStatusUI::default(),
            is_saving: false,
            save_message: None,
        }
    }

    pub fn to_api_keys(&self) -> ApiKeys {
        ApiKeys {
            openai: if self.openai_key.is_empty() { None } else { Some(self.openai_key.clone()) },
            anthropic: if self.anthropic_key.is_empty() { None } else { Some(self.anthropic_key.clone()) },
            google: if self.google_key.is_empty() { None } else { Some(self.google_key.clone()) },
            deepgram: if self.deepgram_key.is_empty() { None } else { Some(self.deepgram_key.clone()) },
        }
    }
}

/// Settings panel component
#[component]
pub fn SettingsPanel(
    is_open: bool,
    on_close: EventHandler<()>,
) -> Element {
    let mut state = use_signal(|| {
        let settings = Settings::load().unwrap_or_default();
        SettingsState::from_settings(&settings)
    });

    // Load from env if settings are empty
    use_effect(move || {
        let mut s = state.write();
        if s.openai_key.is_empty() {
            if let Ok(key) = std::env::var("OPENAI_API_KEY") {
                s.openai_key = key;
            }
        }
        if s.deepgram_key.is_empty() {
            if let Ok(key) = std::env::var("DEEPGRAM_API_KEY") {
                s.deepgram_key = key;
            }
        }
        if s.anthropic_key.is_empty() {
            if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
                s.anthropic_key = key;
            }
        }
        if s.google_key.is_empty() {
            if let Ok(key) = std::env::var("GOOGLE_AI_API_KEY") {
                s.google_key = key;
            }
        }
    });

    let save_settings = move |_| {
        let mut s = state.write();
        s.is_saving = true;
        s.save_message = None;

        // Save API keys
        let api_keys = s.to_api_keys();
        match api_keys.save_secure() {
            Ok(_) => {
                s.save_message = Some("Settings saved!".to_string());

                // Also update environment variables for current session
                if let Some(ref key) = api_keys.openai {
                    std::env::set_var("OPENAI_API_KEY", key);
                }
                if let Some(ref key) = api_keys.deepgram {
                    std::env::set_var("DEEPGRAM_API_KEY", key);
                }
                if let Some(ref key) = api_keys.anthropic {
                    std::env::set_var("ANTHROPIC_API_KEY", key);
                }
                if let Some(ref key) = api_keys.google {
                    std::env::set_var("GOOGLE_AI_API_KEY", key);
                }
            }
            Err(e) => {
                s.save_message = Some(format!("Error: {}", e));
            }
        }
        s.is_saving = false;
    };

    let current = state.read();

    if !is_open {
        return rsx! {};
    }

    rsx! {
        div { class: "settings-overlay",
            onclick: move |_| on_close.call(()),

            div {
                class: "settings-panel",
                onclick: move |e| e.stop_propagation(),

                // Header
                div { class: "settings-header",
                    h2 { "Settings" }
                    button {
                        class: "close-btn",
                        onclick: move |_| on_close.call(()),
                        "x"
                    }
                }

                // API Keys Section
                div { class: "settings-section",
                    h3 { "API Keys" }
                    p { class: "settings-hint",
                        "Keys are stored securely in your system keychain"
                    }

                    div { class: "setting-item",
                        label { "Deepgram (STT)" }
                        input {
                            r#type: "password",
                            placeholder: "Enter Deepgram API key...",
                            value: "{current.deepgram_key}",
                            oninput: move |e| state.write().deepgram_key = e.value().clone(),
                        }
                        if !current.deepgram_key.is_empty() {
                            span { class: "key-status ok", "OK" }
                        }
                    }

                    div { class: "setting-item",
                        label { "OpenAI (GPT-4o)" }
                        input {
                            r#type: "password",
                            placeholder: "sk-...",
                            value: "{current.openai_key}",
                            oninput: move |e| state.write().openai_key = e.value().clone(),
                        }
                        if !current.openai_key.is_empty() {
                            span { class: "key-status ok", "OK" }
                        }
                    }

                    div { class: "setting-item",
                        label { "Anthropic (Claude)" }
                        input {
                            r#type: "password",
                            placeholder: "sk-ant-...",
                            value: "{current.anthropic_key}",
                            oninput: move |e| state.write().anthropic_key = e.value().clone(),
                        }
                        if !current.anthropic_key.is_empty() {
                            span { class: "key-status ok", "OK" }
                        } else {
                            span { class: "key-status optional", "Optional" }
                        }
                    }

                    div { class: "setting-item",
                        label { "Google AI (Gemini)" }
                        input {
                            r#type: "password",
                            placeholder: "AI...",
                            value: "{current.google_key}",
                            oninput: move |e| state.write().google_key = e.value().clone(),
                        }
                        if !current.google_key.is_empty() {
                            span { class: "key-status ok", "OK" }
                        } else {
                            span { class: "key-status optional", "Optional" }
                        }
                    }
                }

                // Model Settings
                div { class: "settings-section",
                    h3 { "Model Selection" }

                    div { class: "setting-item",
                        label { "Flash Model (Quick)" }
                        select {
                            value: "{current.flash_model}",
                            onchange: move |e| state.write().flash_model = e.value().clone(),
                            option { value: "GeminiFlash", "Gemini 2.0 Flash (Recommended)" }
                            option { value: "GPT4oMini", "GPT-4o-mini" }
                            option { value: "LocalOllama", "Local Ollama (Free, Offline)" }
                        }
                    }

                    // Show Ollama config if selected
                    if current.flash_model == "LocalOllama" {
                        div { class: "setting-item ollama-config",
                            label { "Ollama Model" }
                            input {
                                r#type: "text",
                                placeholder: "llama3.1:8b",
                                value: "{current.ollama_model}",
                                oninput: move |e| state.write().ollama_model = e.value().clone(),
                            }
                            if current.ollama_status.available {
                                span { class: "key-status ok", "Connected" }
                            } else {
                                span { class: "key-status error", "Not Running" }
                            }
                        }

                        div { class: "ollama-info",
                            p { class: "settings-hint",
                                "Ollama runs locally - no API costs!"
                            }
                            if !current.ollama_status.available {
                                p { class: "settings-hint warning",
                                    "Start Ollama: "
                                    code { "ollama serve" }
                                }
                                p { class: "settings-hint warning",
                                    "Install model: "
                                    code { "ollama pull llama3.1:8b" }
                                }
                            }
                            if !current.ollama_status.models.is_empty() {
                                p { class: "settings-hint",
                                    "Available: {current.ollama_status.models.join(\", \")}"
                                }
                            }
                        }
                    }

                    div { class: "setting-item",
                        label { "Deep Model (Detailed)" }
                        select {
                            value: "{current.deep_model}",
                            onchange: move |e| state.write().deep_model = e.value().clone(),
                            option { value: "ClaudeSonnet", "Claude 3.5 Sonnet (Recommended)" }
                            option { value: "GPT4o", "GPT-4o" }
                            option { value: "O1Preview", "o1-preview (Complex)" }
                        }
                    }
                }

                // Keyboard Shortcuts (read-only info)
                div { class: "settings-section",
                    h3 { "Keyboard Shortcuts" }

                    div { class: "shortcut-list",
                        div { class: "shortcut-item",
                            span { class: "shortcut-key", "Ctrl+Shift+S" }
                            span { "Start/Stop listening" }
                        }
                        div { class: "shortcut-item",
                            span { class: "shortcut-key", "Ctrl+Shift+H" }
                            span { "Hide/Show window" }
                        }
                        div { class: "shortcut-item",
                            span { class: "shortcut-key", "Ctrl+Shift+M" }
                            span { "Switch mode" }
                        }
                        div { class: "shortcut-item",
                            span { class: "shortcut-key", "Ctrl+Shift+C" }
                            span { "Copy suggestion" }
                        }
                    }
                }

                // Save Button
                div { class: "settings-footer",
                    if let Some(msg) = &current.save_message {
                        span { class: "save-message", "{msg}" }
                    }
                    button {
                        class: "save-btn",
                        disabled: current.is_saving,
                        onclick: save_settings,
                        {if current.is_saving { "Saving..." } else { "Save Settings" }}
                    }
                }

                // Version bar
                div { class: "version-bar",
                    span { class: "version-text", "Voice Copilot v{CURRENT_VERSION}" }
                    super::update_button::UpdateButton {}
                }
            }
        }
    }
}
