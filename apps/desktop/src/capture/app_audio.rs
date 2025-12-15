//! App-Specific Audio Capture
//!
//! Captures audio from specific applications (Zoom, Discord, Teams, etc.)
//! Uses Windows Audio Session API (WASAPI) to capture per-application audio.

use anyhow::{anyhow, Result};
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Known applications that can be captured
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CaptureApp {
    pub name: String,
    pub process_name: String,
    pub icon: &'static str,
}

impl CaptureApp {
    /// Get list of known voice/meeting applications
    pub fn known_apps() -> Vec<CaptureApp> {
        vec![
            CaptureApp {
                name: "Zoom".to_string(),
                process_name: "Zoom.exe".to_string(),
                icon: "📹",
            },
            CaptureApp {
                name: "Discord".to_string(),
                process_name: "Discord.exe".to_string(),
                icon: "🎮",
            },
            CaptureApp {
                name: "Microsoft Teams".to_string(),
                process_name: "Teams.exe".to_string(),
                icon: "👥",
            },
            CaptureApp {
                name: "Google Meet (Chrome)".to_string(),
                process_name: "chrome.exe".to_string(),
                icon: "🌐",
            },
            CaptureApp {
                name: "Google Meet (Edge)".to_string(),
                process_name: "msedge.exe".to_string(),
                icon: "🌐",
            },
            CaptureApp {
                name: "Slack".to_string(),
                process_name: "slack.exe".to_string(),
                icon: "💬",
            },
            CaptureApp {
                name: "Skype".to_string(),
                process_name: "Skype.exe".to_string(),
                icon: "📞",
            },
            CaptureApp {
                name: "WebEx".to_string(),
                process_name: "webexmta.exe".to_string(),
                icon: "🎥",
            },
        ]
    }
}

/// Audio source selection
#[derive(Debug, Clone, PartialEq)]
pub enum AudioSource {
    /// Capture from system default (all audio)
    SystemDefault,
    /// Capture from a specific application
    SpecificApp(CaptureApp),
    /// Capture from a specific device by name
    Device(String),
}

impl Default for AudioSource {
    fn default() -> Self {
        AudioSource::SystemDefault
    }
}

impl AudioSource {
    pub fn display_name(&self) -> String {
        match self {
            AudioSource::SystemDefault => "System Audio (All)".to_string(),
            AudioSource::SpecificApp(app) => format!("{} {}", app.icon, app.name),
            AudioSource::Device(name) => format!("🔊 {}", name),
        }
    }
}

/// Detects running applications that can be captured
#[cfg(target_os = "windows")]
pub fn detect_running_apps() -> Vec<CaptureApp> {
    use sysinfo::System;

    let mut sys = System::new_all();
    sys.refresh_all();

    let known = CaptureApp::known_apps();
    let mut running = Vec::new();

    for process in sys.processes().values() {
        let proc_name = process.name().to_string_lossy().to_lowercase();
        for app in &known {
            if proc_name == app.process_name.to_lowercase() && !running.iter().any(|a: &CaptureApp| a.name == app.name) {
                running.push(app.clone());
            }
        }
    }

    running
}

#[cfg(not(target_os = "windows"))]
pub fn detect_running_apps() -> Vec<CaptureApp> {
    // On non-Windows, just return empty - per-app capture not supported
    Vec::new()
}

/// Audio device information
#[derive(Debug, Clone)]
pub struct AudioDevice {
    pub name: String,
    pub is_input: bool,
    pub is_output: bool,
    pub is_default: bool,
}

/// List all available audio devices
pub fn list_audio_devices() -> Result<Vec<AudioDevice>> {
    use cpal::traits::{DeviceTrait, HostTrait};

    let host = cpal::default_host();
    let mut devices = Vec::new();

    let default_input = host.default_input_device().and_then(|d| d.name().ok());
    let default_output = host.default_output_device().and_then(|d| d.name().ok());

    // List output devices (for loopback capture)
    if let Ok(output_devices) = host.output_devices() {
        for device in output_devices {
            if let Ok(name) = device.name() {
                let is_default = default_output.as_ref().map_or(false, |d| d == &name);
                devices.push(AudioDevice {
                    name: name.clone(),
                    is_input: false,
                    is_output: true,
                    is_default,
                });
            }
        }
    }

    // List input devices (microphones)
    if let Ok(input_devices) = host.input_devices() {
        for device in input_devices {
            if let Ok(name) = device.name() {
                let is_default = default_input.as_ref().map_or(false, |d| d == &name);
                // Check if we already have this device from output
                if let Some(existing) = devices.iter_mut().find(|d| d.name == name) {
                    existing.is_input = true;
                } else {
                    devices.push(AudioDevice {
                        name,
                        is_input: true,
                        is_output: false,
                        is_default,
                    });
                }
            }
        }
    }

    Ok(devices)
}

/// Get available audio sources (apps + devices)
pub fn get_available_sources() -> Vec<AudioSource> {
    let mut sources = vec![AudioSource::SystemDefault];

    // Add running apps
    for app in detect_running_apps() {
        sources.push(AudioSource::SpecificApp(app));
    }

    // Add devices
    if let Ok(devices) = list_audio_devices() {
        for device in devices {
            if device.is_output {
                sources.push(AudioSource::Device(device.name));
            }
        }
    }

    sources
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_apps() {
        let apps = CaptureApp::known_apps();
        assert!(!apps.is_empty());
        assert!(apps.iter().any(|a| a.name == "Zoom"));
    }

    #[test]
    fn test_list_devices() {
        let devices = list_audio_devices();
        assert!(devices.is_ok());
        println!("Devices: {:?}", devices.unwrap());
    }
}
