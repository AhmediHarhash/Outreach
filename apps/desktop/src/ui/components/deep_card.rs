//! Deep Card Component
//!
//! Displays the detailed response from the Deep model (Claude 3.5 Sonnet / GPT-4o).
//! This streams in while you're talking, providing comprehensive answers.

use dioxus::prelude::*;
use crate::ui::app::DeepResponse;

#[derive(Props, Clone, PartialEq)]
pub struct DeepCardProps {
    pub response: DeepResponse,
}

#[component]
pub fn DeepCard(props: DeepCardProps) -> Element {
    rsx! {
        div { class: "deep-section",
            div { class: "deep-header",
                span { "🧠" }
                span { "DETAILED ANSWER" }
                if props.response.is_streaming {
                    span { class: "streaming-indicator", " • streaming..." }
                }
            }

            // Main content (streams in)
            div {
                class: if props.response.is_streaming { "deep-content streaming" } else { "deep-content" },
                "{props.response.content}"
            }

            // Question to ask them (appears at end)
            if let Some(question) = &props.response.question_to_ask {
                div { class: "question-back",
                    div { class: "question-label", "🔄 ASK THEM:" }
                    div { "{question}" }
                }
            }
        }
    }
}
