//! Flash Card Component
//!
//! Displays the quick response with bullet points from the Flash model (Gemini 2.0 Flash).
//! This appears within ~500ms and gives immediate talking points.

use dioxus::prelude::*;
use crate::ui::app::{FlashResponse, Bullet};

#[derive(Props, Clone, PartialEq)]
pub struct FlashCardProps {
    pub response: FlashResponse,
}

#[component]
pub fn FlashCard(props: FlashCardProps) -> Element {
    rsx! {
        div { class: "flash-section",
            div { class: "flash-header",
                span { "⚡" }
                span { "QUICK RESPONSE" }
                span { class: "flash-type", " • {props.response.response_type}" }
            }

            // Summary line
            div { class: "flash-summary", "{props.response.summary}" }

            // Bullet points
            ul { class: "bullet-list",
                for (idx, bullet) in props.response.bullets.iter().enumerate() {
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
}
