//! Mode Selector Component
//!
//! Allows switching between different copilot modes (Sales, Interview, Technical).

use dioxus::prelude::*;
use crate::ui::app::CopilotMode;

#[derive(Props, Clone, PartialEq)]
pub struct ModeSelectorProps {
    pub current_mode: CopilotMode,
    pub on_change: EventHandler<CopilotMode>,
}

#[component]
pub fn ModeSelector(props: ModeSelectorProps) -> Element {
    let modes = vec![
        CopilotMode::Sales,
        CopilotMode::Interview,
        CopilotMode::Technical,
        CopilotMode::General,
    ];

    rsx! {
        div { class: "mode-selector",
            for mode in modes {
                button {
                    class: if props.current_mode == mode { "mode-btn active" } else { "mode-btn" },
                    onclick: move |_| props.on_change.call(mode.clone()),
                    "{mode.label()}"
                }
            }
        }
    }
}
