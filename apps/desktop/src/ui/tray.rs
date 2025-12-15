//! System Tray Integration
//!
//! Provides system tray icon and menu for quick access.

use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu},
    TrayIcon, TrayIconBuilder,
};
use std::sync::mpsc;

/// Tray menu actions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrayAction {
    Show,
    Hide,
    StartListening,
    StopListening,
    ModeSales,
    ModeInterview,
    ModeTechnical,
    Settings,
    Quit,
}

/// System tray handler
pub struct TrayHandler {
    tray_icon: TrayIcon,
    menu_channel: mpsc::Receiver<MenuEvent>,
    // Menu item IDs
    show_id: String,
    hide_id: String,
    start_id: String,
    stop_id: String,
    sales_id: String,
    interview_id: String,
    technical_id: String,
    settings_id: String,
    quit_id: String,
}

impl TrayHandler {
    /// Create the system tray icon and menu
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create menu items
        let show_item = MenuItem::new("Show Window", true, None);
        let hide_item = MenuItem::new("Hide Window", true, None);
        let separator1 = PredefinedMenuItem::separator();

        let start_item = MenuItem::new("Start Listening", true, None);
        let stop_item = MenuItem::new("Stop Listening", true, None);
        let separator2 = PredefinedMenuItem::separator();

        // Mode submenu
        let sales_item = MenuItem::new("Sales", true, None);
        let interview_item = MenuItem::new("Interview", true, None);
        let technical_item = MenuItem::new("Technical", true, None);

        let mode_menu = Submenu::new("Mode", true);
        mode_menu.append(&sales_item)?;
        mode_menu.append(&interview_item)?;
        mode_menu.append(&technical_item)?;

        let separator3 = PredefinedMenuItem::separator();
        let settings_item = MenuItem::new("Settings...", true, None);
        let separator4 = PredefinedMenuItem::separator();
        let quit_item = MenuItem::new("Quit", true, None);

        // Build menu
        let menu = Menu::new();
        menu.append(&show_item)?;
        menu.append(&hide_item)?;
        menu.append(&separator1)?;
        menu.append(&start_item)?;
        menu.append(&stop_item)?;
        menu.append(&separator2)?;
        menu.append(&mode_menu)?;
        menu.append(&separator3)?;
        menu.append(&settings_item)?;
        menu.append(&separator4)?;
        menu.append(&quit_item)?;

        // Get menu event channel
        let menu_channel = MenuEvent::receiver().clone();

        // Create tray icon
        // Note: In production, embed an actual icon
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Voice Copilot")
            .build()?;

        Ok(Self {
            tray_icon,
            menu_channel,
            show_id: show_item.id().0.to_string(),
            hide_id: hide_item.id().0.to_string(),
            start_id: start_item.id().0.to_string(),
            stop_id: stop_item.id().0.to_string(),
            sales_id: sales_item.id().0.to_string(),
            interview_id: interview_item.id().0.to_string(),
            technical_id: technical_item.id().0.to_string(),
            settings_id: settings_item.id().0.to_string(),
            quit_id: quit_item.id().0.to_string(),
        })
    }

    /// Check for menu events (non-blocking)
    pub fn poll_event(&self) -> Option<TrayAction> {
        match self.menu_channel.try_recv() {
            Ok(event) => {
                let id = event.id().0.to_string();

                if id == self.show_id {
                    Some(TrayAction::Show)
                } else if id == self.hide_id {
                    Some(TrayAction::Hide)
                } else if id == self.start_id {
                    Some(TrayAction::StartListening)
                } else if id == self.stop_id {
                    Some(TrayAction::StopListening)
                } else if id == self.sales_id {
                    Some(TrayAction::ModeSales)
                } else if id == self.interview_id {
                    Some(TrayAction::ModeInterview)
                } else if id == self.technical_id {
                    Some(TrayAction::ModeTechnical)
                } else if id == self.settings_id {
                    Some(TrayAction::Settings)
                } else if id == self.quit_id {
                    Some(TrayAction::Quit)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    /// Update tooltip text
    pub fn set_tooltip(&self, text: &str) {
        let _ = self.tray_icon.set_tooltip(Some(text));
    }
}

/// Spawn tray event listener
pub fn spawn_tray_listener(
    action_tx: tokio::sync::mpsc::Sender<TrayAction>,
) -> Option<std::thread::JoinHandle<()>> {
    // Tray must be created on main thread for Windows
    // This function should be called from the UI thread

    match TrayHandler::new() {
        Ok(handler) => {
            let handle = std::thread::spawn(move || {
                tracing::info!("System tray initialized");

                loop {
                    if let Some(action) = handler.poll_event() {
                        tracing::debug!("Tray action: {:?}", action);
                        if action == TrayAction::Quit {
                            let _ = action_tx.blocking_send(action);
                            break;
                        }
                        let _ = action_tx.blocking_send(action);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            });
            Some(handle)
        }
        Err(e) => {
            tracing::error!("Failed to create system tray: {}", e);
            None
        }
    }
}
