//! Overlay Component
//!
//! A floating, always-on-top panel that displays suggestions.
//! Can be minimized to a compact mode while on calls.

use dioxus::prelude::*;

/// Overlay display mode
#[derive(Debug, Clone, Default, PartialEq)]
pub enum OverlayMode {
    #[default]
    Full,       // Full panel with all sections
    Compact,    // Just quick bullets
    Minimal,    // Just status indicator
}

#[component]
pub fn Overlay() -> Element {
    // This component will be used for the detachable floating window
    // For now, the main app.rs contains the overlay UI inline
    rsx! {
        div { "Overlay placeholder" }
    }
}
