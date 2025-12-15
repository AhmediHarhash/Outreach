//! Theme System
//!
//! Customizable color themes and styling for the UI.
//! Provides color-coded outputs based on content type.

use serde::{Deserialize, Serialize};

/// Color palette for the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Theme name
    pub name: String,

    // Base colors
    pub bg_primary: String,
    pub bg_secondary: String,
    pub bg_tertiary: String,
    pub bg_hover: String,

    // Text colors
    pub text_primary: String,
    pub text_secondary: String,
    pub text_muted: String,

    // Accent colors
    pub accent_blue: String,
    pub accent_green: String,
    pub accent_yellow: String,
    pub accent_orange: String,
    pub accent_red: String,
    pub accent_purple: String,
    pub accent_cyan: String,
    pub accent_pink: String,

    // Semantic colors for outputs
    pub color_transcript: String,      // What they said
    pub color_flash: String,           // Quick bullets
    pub color_deep: String,            // Detailed response
    pub color_question: String,        // Questions to ask
    pub color_objection: String,       // Objection detected
    pub color_buying_signal: String,   // Positive signal
    pub color_technical: String,       // Technical topic
    pub color_warning: String,         // Warning/caution
    pub color_success: String,         // Success indicator

    // Border and effects
    pub border_color: String,
    pub border_focus: String,
    pub shadow_color: String,
    pub glow_color: String,

    // Gradients
    pub gradient_start: String,
    pub gradient_end: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Dark theme (default) - Professional and easy on the eyes
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),

            // Base - Deep dark with subtle blue tint
            bg_primary: "#0d1117".to_string(),
            bg_secondary: "#161b22".to_string(),
            bg_tertiary: "#21262d".to_string(),
            bg_hover: "#30363d".to_string(),

            // Text
            text_primary: "#f0f6fc".to_string(),
            text_secondary: "#8b949e".to_string(),
            text_muted: "#484f58".to_string(),

            // Accents - Vibrant but not harsh
            accent_blue: "#58a6ff".to_string(),
            accent_green: "#3fb950".to_string(),
            accent_yellow: "#d29922".to_string(),
            accent_orange: "#db6d28".to_string(),
            accent_red: "#f85149".to_string(),
            accent_purple: "#a371f7".to_string(),
            accent_cyan: "#39c5cf".to_string(),
            accent_pink: "#db61a2".to_string(),

            // Semantic - Distinct colors for each type
            color_transcript: "#8b949e".to_string(),     // Gray - neutral
            color_flash: "#58a6ff".to_string(),          // Blue - quick info
            color_deep: "#a371f7".to_string(),           // Purple - detailed
            color_question: "#39c5cf".to_string(),       // Cyan - question to ask
            color_objection: "#f85149".to_string(),      // Red - caution
            color_buying_signal: "#3fb950".to_string(),  // Green - positive
            color_technical: "#db6d28".to_string(),      // Orange - technical
            color_warning: "#d29922".to_string(),        // Yellow - warning
            color_success: "#3fb950".to_string(),        // Green - success

            // Borders
            border_color: "#30363d".to_string(),
            border_focus: "#58a6ff".to_string(),
            shadow_color: "rgba(0, 0, 0, 0.4)".to_string(),
            glow_color: "rgba(88, 166, 255, 0.15)".to_string(),

            // Gradient for headers
            gradient_start: "#58a6ff".to_string(),
            gradient_end: "#a371f7".to_string(),
        }
    }

    /// Light theme - Clean and professional
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),

            bg_primary: "#ffffff".to_string(),
            bg_secondary: "#f6f8fa".to_string(),
            bg_tertiary: "#eaeef2".to_string(),
            bg_hover: "#d0d7de".to_string(),

            text_primary: "#1f2328".to_string(),
            text_secondary: "#656d76".to_string(),
            text_muted: "#8c959f".to_string(),

            accent_blue: "#0969da".to_string(),
            accent_green: "#1a7f37".to_string(),
            accent_yellow: "#9a6700".to_string(),
            accent_orange: "#bc4c00".to_string(),
            accent_red: "#cf222e".to_string(),
            accent_purple: "#8250df".to_string(),
            accent_cyan: "#0969da".to_string(),
            accent_pink: "#bf3989".to_string(),

            color_transcript: "#656d76".to_string(),
            color_flash: "#0969da".to_string(),
            color_deep: "#8250df".to_string(),
            color_question: "#0969da".to_string(),
            color_objection: "#cf222e".to_string(),
            color_buying_signal: "#1a7f37".to_string(),
            color_technical: "#bc4c00".to_string(),
            color_warning: "#9a6700".to_string(),
            color_success: "#1a7f37".to_string(),

            border_color: "#d0d7de".to_string(),
            border_focus: "#0969da".to_string(),
            shadow_color: "rgba(0, 0, 0, 0.1)".to_string(),
            glow_color: "rgba(9, 105, 218, 0.1)".to_string(),

            gradient_start: "#0969da".to_string(),
            gradient_end: "#8250df".to_string(),
        }
    }

    /// High contrast theme - Maximum readability
    pub fn high_contrast() -> Self {
        Self {
            name: "High Contrast".to_string(),

            bg_primary: "#000000".to_string(),
            bg_secondary: "#0a0a0a".to_string(),
            bg_tertiary: "#141414".to_string(),
            bg_hover: "#1e1e1e".to_string(),

            text_primary: "#ffffff".to_string(),
            text_secondary: "#cccccc".to_string(),
            text_muted: "#888888".to_string(),

            accent_blue: "#00d4ff".to_string(),
            accent_green: "#00ff7f".to_string(),
            accent_yellow: "#ffff00".to_string(),
            accent_orange: "#ff8c00".to_string(),
            accent_red: "#ff4444".to_string(),
            accent_purple: "#cc66ff".to_string(),
            accent_cyan: "#00ffff".to_string(),
            accent_pink: "#ff66cc".to_string(),

            color_transcript: "#aaaaaa".to_string(),
            color_flash: "#00d4ff".to_string(),
            color_deep: "#cc66ff".to_string(),
            color_question: "#00ffff".to_string(),
            color_objection: "#ff4444".to_string(),
            color_buying_signal: "#00ff7f".to_string(),
            color_technical: "#ff8c00".to_string(),
            color_warning: "#ffff00".to_string(),
            color_success: "#00ff7f".to_string(),

            border_color: "#333333".to_string(),
            border_focus: "#00d4ff".to_string(),
            shadow_color: "rgba(0, 0, 0, 0.8)".to_string(),
            glow_color: "rgba(0, 212, 255, 0.3)".to_string(),

            gradient_start: "#00d4ff".to_string(),
            gradient_end: "#cc66ff".to_string(),
        }
    }

    /// Cyberpunk theme - Neon and futuristic
    pub fn cyberpunk() -> Self {
        Self {
            name: "Cyberpunk".to_string(),

            bg_primary: "#0a0a12".to_string(),
            bg_secondary: "#12121f".to_string(),
            bg_tertiary: "#1a1a2e".to_string(),
            bg_hover: "#252540".to_string(),

            text_primary: "#eaeaea".to_string(),
            text_secondary: "#9090a0".to_string(),
            text_muted: "#606070".to_string(),

            accent_blue: "#00f0ff".to_string(),
            accent_green: "#00ff9f".to_string(),
            accent_yellow: "#ffe600".to_string(),
            accent_orange: "#ff6b00".to_string(),
            accent_red: "#ff0055".to_string(),
            accent_purple: "#bf00ff".to_string(),
            accent_cyan: "#00f0ff".to_string(),
            accent_pink: "#ff00aa".to_string(),

            color_transcript: "#9090a0".to_string(),
            color_flash: "#00f0ff".to_string(),
            color_deep: "#bf00ff".to_string(),
            color_question: "#00ff9f".to_string(),
            color_objection: "#ff0055".to_string(),
            color_buying_signal: "#00ff9f".to_string(),
            color_technical: "#ff6b00".to_string(),
            color_warning: "#ffe600".to_string(),
            color_success: "#00ff9f".to_string(),

            border_color: "#2a2a4a".to_string(),
            border_focus: "#00f0ff".to_string(),
            shadow_color: "rgba(0, 0, 0, 0.6)".to_string(),
            glow_color: "rgba(0, 240, 255, 0.2)".to_string(),

            gradient_start: "#00f0ff".to_string(),
            gradient_end: "#ff00aa".to_string(),
        }
    }

    /// Generate CSS variables from theme
    pub fn to_css_vars(&self) -> String {
        format!(
            r#"
            --bg-primary: {};
            --bg-secondary: {};
            --bg-tertiary: {};
            --bg-hover: {};
            --text-primary: {};
            --text-secondary: {};
            --text-muted: {};
            --accent-blue: {};
            --accent-green: {};
            --accent-yellow: {};
            --accent-orange: {};
            --accent-red: {};
            --accent-purple: {};
            --accent-cyan: {};
            --accent-pink: {};
            --color-transcript: {};
            --color-flash: {};
            --color-deep: {};
            --color-question: {};
            --color-objection: {};
            --color-buying-signal: {};
            --color-technical: {};
            --color-warning: {};
            --color-success: {};
            --border-color: {};
            --border-focus: {};
            --shadow-color: {};
            --glow-color: {};
            --gradient-start: {};
            --gradient-end: {};
            "#,
            self.bg_primary, self.bg_secondary, self.bg_tertiary, self.bg_hover,
            self.text_primary, self.text_secondary, self.text_muted,
            self.accent_blue, self.accent_green, self.accent_yellow, self.accent_orange,
            self.accent_red, self.accent_purple, self.accent_cyan, self.accent_pink,
            self.color_transcript, self.color_flash, self.color_deep, self.color_question,
            self.color_objection, self.color_buying_signal, self.color_technical,
            self.color_warning, self.color_success,
            self.border_color, self.border_focus, self.shadow_color, self.glow_color,
            self.gradient_start, self.gradient_end
        )
    }
}

/// Get color for statement type
pub fn get_statement_color(statement_type: &str) -> &'static str {
    match statement_type.to_lowercase().as_str() {
        "question" => "var(--color-flash)",
        "objection" => "var(--color-objection)",
        "buying_signal" => "var(--color-buying-signal)",
        "technical" => "var(--color-technical)",
        "statement" => "var(--text-secondary)",
        "small_talk" => "var(--text-muted)",
        _ => "var(--text-primary)",
    }
}

/// Get color for urgency level
pub fn get_urgency_color(urgency: &str) -> &'static str {
    match urgency.to_lowercase().as_str() {
        "answer_now" => "var(--accent-red)",
        "can_elaborate" => "var(--accent-yellow)",
        "just_listening" => "var(--accent-green)",
        _ => "var(--text-secondary)",
    }
}

/// Get color for sentiment
pub fn get_sentiment_color(sentiment: &str) -> &'static str {
    match sentiment.to_lowercase().as_str() {
        "very_positive" | "positive" => "var(--accent-green)",
        "very_negative" | "negative" => "var(--accent-red)",
        _ => "var(--text-secondary)",
    }
}
