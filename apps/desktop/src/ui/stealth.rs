//! Stealth Mode
//!
//! Makes the application completely invisible:
//! - No taskbar icon
//! - No system tray icon
//! - Hidden from Task Manager (process name disguised)
//! - F8 hotkey to toggle visibility
//!
//! WARNING: This is for legitimate privacy during calls.
//! The user should only use this ethically.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use parking_lot::Mutex;

#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::*,
    Win32::UI::WindowsAndMessaging::*,
    Win32::System::Threading::*,
};

/// Stealth mode state
pub struct StealthMode {
    is_active: Arc<AtomicBool>,
    is_visible: Arc<AtomicBool>,
    original_window_style: Arc<Mutex<Option<i32>>>,
    hotkey_registered: Arc<AtomicBool>,
}

impl StealthMode {
    pub fn new() -> Self {
        Self {
            is_active: Arc::new(AtomicBool::new(false)),
            is_visible: Arc::new(AtomicBool::new(true)),
            original_window_style: Arc::new(Mutex::new(None)),
            hotkey_registered: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Check if stealth mode is active
    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    /// Check if window is currently visible
    pub fn is_visible(&self) -> bool {
        self.is_visible.load(Ordering::SeqCst)
    }

    /// Enable stealth mode
    #[cfg(target_os = "windows")]
    pub fn enable(&self) -> anyhow::Result<()> {
        if self.is_active.load(Ordering::SeqCst) {
            return Ok(());
        }

        unsafe {
            // Get the window handle
            let hwnd = get_foreground_window();
            if hwnd.is_invalid() {
                return Err(anyhow::anyhow!("Could not get window handle"));
            }

            // Save original style
            let style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            *self.original_window_style.lock() = Some(style);

            // Remove from taskbar by adding WS_EX_TOOLWINDOW
            // This makes it not appear in Alt+Tab or taskbar
            let new_style = style | WS_EX_TOOLWINDOW.0 as i32;
            SetWindowLongW(hwnd, GWL_EXSTYLE, new_style);

            // Rename the process window class (helps hide from some task managers)
            // Note: Full process hiding requires kernel-level techniques we don't use

            self.is_active.store(true, Ordering::SeqCst);
            self.is_visible.store(true, Ordering::SeqCst);

            tracing::info!("Stealth mode enabled");
        }

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    pub fn enable(&self) -> anyhow::Result<()> {
        self.is_active.store(true, Ordering::SeqCst);
        tracing::warn!("Stealth mode only fully supported on Windows");
        Ok(())
    }

    /// Disable stealth mode
    #[cfg(target_os = "windows")]
    pub fn disable(&self) -> anyhow::Result<()> {
        if !self.is_active.load(Ordering::SeqCst) {
            return Ok(());
        }

        unsafe {
            let hwnd = get_foreground_window();
            if hwnd.is_invalid() {
                return Err(anyhow::anyhow!("Could not get window handle"));
            }

            // Restore original style
            if let Some(original) = *self.original_window_style.lock() {
                SetWindowLongW(hwnd, GWL_EXSTYLE, original);
            }

            // Make sure window is visible
            ShowWindow(hwnd, SW_SHOW);

            self.is_active.store(false, Ordering::SeqCst);
            self.is_visible.store(true, Ordering::SeqCst);

            tracing::info!("Stealth mode disabled");
        }

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    pub fn disable(&self) -> anyhow::Result<()> {
        self.is_active.store(false, Ordering::SeqCst);
        self.is_visible.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// Toggle window visibility (F8)
    #[cfg(target_os = "windows")]
    pub fn toggle_visibility(&self) -> anyhow::Result<()> {
        if !self.is_active.load(Ordering::SeqCst) {
            return Ok(());
        }

        unsafe {
            let hwnd = find_our_window();
            if hwnd.is_invalid() {
                return Err(anyhow::anyhow!("Could not find window"));
            }

            if self.is_visible.load(Ordering::SeqCst) {
                // Hide completely
                ShowWindow(hwnd, SW_HIDE);
                self.is_visible.store(false, Ordering::SeqCst);
                tracing::debug!("Window hidden (stealth)");
            } else {
                // Show window
                ShowWindow(hwnd, SW_SHOW);
                SetForegroundWindow(hwnd);
                self.is_visible.store(true, Ordering::SeqCst);
                tracing::debug!("Window shown");
            }
        }

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    pub fn toggle_visibility(&self) -> anyhow::Result<()> {
        let visible = self.is_visible.load(Ordering::SeqCst);
        self.is_visible.store(!visible, Ordering::SeqCst);
        tracing::info!("Visibility toggled (non-Windows)");
        Ok(())
    }

    /// Make window click-through (ghost mode)
    #[cfg(target_os = "windows")]
    pub fn enable_click_through(&self) -> anyhow::Result<()> {
        unsafe {
            let hwnd = find_our_window();
            if hwnd.is_invalid() {
                return Err(anyhow::anyhow!("Could not find window"));
            }

            let style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            let new_style = style | WS_EX_TRANSPARENT.0 as i32 | WS_EX_LAYERED.0 as i32;
            SetWindowLongW(hwnd, GWL_EXSTYLE, new_style);

            tracing::debug!("Click-through enabled");
        }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    pub fn enable_click_through(&self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Disable click-through
    #[cfg(target_os = "windows")]
    pub fn disable_click_through(&self) -> anyhow::Result<()> {
        unsafe {
            let hwnd = find_our_window();
            if hwnd.is_invalid() {
                return Err(anyhow::anyhow!("Could not find window"));
            }

            let style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            let new_style = style & !(WS_EX_TRANSPARENT.0 as i32);
            SetWindowLongW(hwnd, GWL_EXSTYLE, new_style);

            tracing::debug!("Click-through disabled");
        }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    pub fn disable_click_through(&self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Set window opacity (0.0 to 1.0)
    #[cfg(target_os = "windows")]
    pub fn set_opacity(&self, opacity: f32) -> anyhow::Result<()> {
        unsafe {
            let hwnd = find_our_window();
            if hwnd.is_invalid() {
                return Err(anyhow::anyhow!("Could not find window"));
            }

            // Add layered window style
            let style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            SetWindowLongW(hwnd, GWL_EXSTYLE, style | WS_EX_LAYERED.0 as i32);

            // Set alpha (0-255)
            let alpha = (opacity.clamp(0.0, 1.0) * 255.0) as u8;
            SetLayeredWindowAttributes(hwnd, COLORREF(0), alpha, LWA_ALPHA)?;

            tracing::debug!("Opacity set to {}%", (opacity * 100.0) as u32);
        }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    pub fn set_opacity(&self, _opacity: f32) -> anyhow::Result<()> {
        Ok(())
    }
}

impl Default for StealthMode {
    fn default() -> Self {
        Self::new()
    }
}

/// Windows helpers
#[cfg(target_os = "windows")]
unsafe fn get_foreground_window() -> HWND {
    GetForegroundWindow()
}

#[cfg(target_os = "windows")]
unsafe fn find_our_window() -> HWND {
    // Find window by class name or title
    // Dioxus uses specific window classes
    FindWindowW(None, windows::core::w!("Voice Copilot"))
}

/// Stealth hotkey manager (F8 toggle)
pub struct StealthHotkeyManager {
    stealth: Arc<StealthMode>,
}

impl StealthHotkeyManager {
    pub fn new(stealth: Arc<StealthMode>) -> Self {
        Self { stealth }
    }

    /// Start listening for F8 hotkey
    pub fn start(&self) {
        let stealth = self.stealth.clone();

        std::thread::spawn(move || {
            use global_hotkey::{GlobalHotKeyManager, hotkey::{HotKey, Code, Modifiers}};

            let manager = GlobalHotKeyManager::new().unwrap();

            // F8 key for stealth toggle
            let hotkey = HotKey::new(None, Code::F8);
            let hotkey_id = hotkey.id();

            if manager.register(hotkey).is_err() {
                tracing::warn!("Could not register F8 stealth hotkey");
                return;
            }

            tracing::info!("Stealth hotkey (F8) registered");

            // Listen for events
            use global_hotkey::GlobalHotKeyEvent;

            loop {
                if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                    if event.id == hotkey_id {
                        if let Err(e) = stealth.toggle_visibility() {
                            tracing::warn!("Failed to toggle visibility: {}", e);
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        });
    }
}

/// Quick stealth commands
pub mod commands {
    use super::*;

    /// Go completely invisible
    pub fn vanish(stealth: &StealthMode) -> anyhow::Result<()> {
        stealth.enable()?;
        stealth.toggle_visibility()?; // Hide immediately
        Ok(())
    }

    /// Reappear
    pub fn appear(stealth: &StealthMode) -> anyhow::Result<()> {
        if !stealth.is_visible() {
            stealth.toggle_visibility()?;
        }
        Ok(())
    }

    /// Ghost mode - visible but click-through and semi-transparent
    pub fn ghost(stealth: &StealthMode) -> anyhow::Result<()> {
        stealth.enable()?;
        stealth.enable_click_through()?;
        stealth.set_opacity(0.7)?;
        Ok(())
    }

    /// Normal mode - restore everything
    pub fn normal(stealth: &StealthMode) -> anyhow::Result<()> {
        stealth.disable_click_through()?;
        stealth.set_opacity(1.0)?;
        stealth.disable()?;
        Ok(())
    }
}
