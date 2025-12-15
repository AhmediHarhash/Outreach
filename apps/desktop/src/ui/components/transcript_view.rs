//! Transcript View Component
//!
//! Displays the real-time transcript of what the other person is saying.

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct TranscriptViewProps {
    pub text: String,
    #[props(default = false)]
    pub is_listening: bool,
}

#[component]
pub fn TranscriptView(props: TranscriptViewProps) -> Element {
    rsx! {
        div { class: "transcript-section",
            div { class: "transcript-label",
                span { "🎤" }
                span { "They said:" }
            }
            div { class: "transcript-text",
                {if props.text.is_empty() {
                    if props.is_listening {
                        "Listening..."
                    } else {
                        "Waiting for speech..."
                    }
                } else {
                    props.text.as_str()
                }}
            }
        }
    }
}
