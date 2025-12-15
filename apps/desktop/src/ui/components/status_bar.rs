//! Status Bar Component
//!
//! Shows connection status and current mode at the top of the window.

use dioxus::prelude::*;
use crate::ui::app::ConnectionStatus;

#[derive(Props, Clone, PartialEq)]
pub struct StatusBarProps {
    pub status: ConnectionStatus,
    pub mode_label: String,
}

#[component]
pub fn StatusBar(props: StatusBarProps) -> Element {
    let (status_text, is_connected) = match &props.status {
        ConnectionStatus::Disconnected => ("Ready", false),
        ConnectionStatus::Connecting => ("Connecting...", false),
        ConnectionStatus::Connected => ("Listening", true),
        ConnectionStatus::Error(e) => ("Error", false),
    };

    rsx! {
        div { class: "status-bar",
            div { class: "status-indicator",
                div {
                    class: if is_connected { "status-dot connected" } else { "status-dot" }
                }
                span { "{status_text}" }
            }
            div { class: "mode-label", "{props.mode_label}" }
        }
    }
}
