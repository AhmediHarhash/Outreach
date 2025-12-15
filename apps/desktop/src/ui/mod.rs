//! UI Module - Dioxus-based native desktop interface
//!
//! Provides:
//! - Main application window with overlay mode
//! - Floating suggestion panel
//! - System tray integration
//! - Settings management
//! - Runtime service for pipeline integration
//! - Global hotkey support
//! - Auto-update system
//! - Stealth mode (F8 toggle)
//! - Theme system with color-coded outputs

mod app;
mod overlay;
mod components;
mod tray;
mod settings;
mod hotkeys;
mod update_button;
mod theme;
mod stealth;
mod styles;
pub mod runtime;

pub use app::launch_app;
pub use runtime::{RuntimeHandle, RuntimeService, SharedState};
pub use hotkeys::{HotkeyHandler, HotkeyAction, spawn_hotkey_listener};
pub use tray::{TrayHandler, TrayAction, spawn_tray_listener};
pub use settings::SettingsPanel;
pub use update_button::UpdateButton;
pub use theme::{Theme, get_statement_color, get_urgency_color, get_sentiment_color};
pub use stealth::{StealthMode, StealthHotkeyManager, commands as stealth_commands};
pub use styles::{POLISHED_CSS, get_themed_css};
