//! Global Hotkey Integration
//!
//! Registers and handles global keyboard shortcuts:
//! - Ctrl+Shift+S: Start/Stop listening
//! - Ctrl+Shift+H: Hide/Show window
//! - Ctrl+Shift+M: Switch mode
//! - Ctrl+Shift+C: Copy last suggestion

use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use std::sync::mpsc;

/// Hotkey actions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HotkeyAction {
    ToggleListen,
    ToggleVisibility,
    SwitchMode,
    CopySuggestion,
}

/// Hotkey manager that registers and handles global shortcuts
pub struct HotkeyHandler {
    manager: GlobalHotKeyManager,
    toggle_listen_id: u32,
    toggle_visibility_id: u32,
    switch_mode_id: u32,
    copy_suggestion_id: u32,
}

impl HotkeyHandler {
    /// Create and register all hotkeys
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let manager = GlobalHotKeyManager::new()?;

        // Ctrl+Shift+S - Toggle listening
        let toggle_listen = HotKey::new(
            Some(Modifiers::CONTROL | Modifiers::SHIFT),
            Code::KeyS,
        );

        // Ctrl+Shift+H - Toggle visibility
        let toggle_visibility = HotKey::new(
            Some(Modifiers::CONTROL | Modifiers::SHIFT),
            Code::KeyH,
        );

        // Ctrl+Shift+M - Switch mode
        let switch_mode = HotKey::new(
            Some(Modifiers::CONTROL | Modifiers::SHIFT),
            Code::KeyM,
        );

        // Ctrl+Shift+C - Copy suggestion
        let copy_suggestion = HotKey::new(
            Some(Modifiers::CONTROL | Modifiers::SHIFT),
            Code::KeyC,
        );

        // Register all hotkeys
        manager.register(toggle_listen)?;
        manager.register(toggle_visibility)?;
        manager.register(switch_mode)?;
        manager.register(copy_suggestion)?;

        Ok(Self {
            manager,
            toggle_listen_id: toggle_listen.id(),
            toggle_visibility_id: toggle_visibility.id(),
            switch_mode_id: switch_mode.id(),
            copy_suggestion_id: copy_suggestion.id(),
        })
    }

    /// Get the action for a hotkey event
    pub fn get_action(&self, event: &GlobalHotKeyEvent) -> Option<HotkeyAction> {
        let id = event.id();

        if id == self.toggle_listen_id {
            Some(HotkeyAction::ToggleListen)
        } else if id == self.toggle_visibility_id {
            Some(HotkeyAction::ToggleVisibility)
        } else if id == self.switch_mode_id {
            Some(HotkeyAction::SwitchMode)
        } else if id == self.copy_suggestion_id {
            Some(HotkeyAction::CopySuggestion)
        } else {
            None
        }
    }

    /// Get the global hotkey event receiver
    pub fn receiver() -> std::sync::mpsc::Receiver<GlobalHotKeyEvent> {
        GlobalHotKeyEvent::receiver().clone()
    }
}

impl Drop for HotkeyHandler {
    fn drop(&mut self) {
        // Hotkeys are automatically unregistered when manager is dropped
    }
}

/// Spawn hotkey listener thread
pub fn spawn_hotkey_listener(
    action_tx: tokio::sync::mpsc::Sender<HotkeyAction>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let handler = match HotkeyHandler::new() {
            Ok(h) => h,
            Err(e) => {
                tracing::error!("Failed to register hotkeys: {}", e);
                return;
            }
        };

        tracing::info!("Hotkeys registered:");
        tracing::info!("  Ctrl+Shift+S: Start/Stop listening");
        tracing::info!("  Ctrl+Shift+H: Hide/Show window");
        tracing::info!("  Ctrl+Shift+M: Switch mode");
        tracing::info!("  Ctrl+Shift+C: Copy suggestion");

        let receiver = HotkeyHandler::receiver();

        loop {
            if let Ok(event) = receiver.recv() {
                if let Some(action) = handler.get_action(&event) {
                    tracing::debug!("Hotkey action: {:?}", action);
                    let _ = action_tx.blocking_send(action);
                }
            }
        }
    })
}
