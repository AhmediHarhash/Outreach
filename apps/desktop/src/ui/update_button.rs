//! Update Button Component
//!
//! Shows update status and allows downloading/installing updates.

use dioxus::prelude::*;
use crate::updater::{UpdateStatus, UpdateInfo, check_for_updates, open_release_page, CURRENT_VERSION};

/// Update button component
#[component]
pub fn UpdateButton() -> Element {
    let mut status = use_signal(|| UpdateStatus::Unknown);
    let mut checking = use_signal(|| false);

    // Check for updates on first render
    use_effect(move || {
        if *status.read() == UpdateStatus::Unknown && !*checking.read() {
            checking.set(true);
            spawn(async move {
                status.set(UpdateStatus::Checking);
                match check_for_updates().await {
                    Ok(s) => status.set(s),
                    Err(e) => {
                        tracing::warn!("Update check failed: {}", e);
                        status.set(UpdateStatus::Error(e.to_string()));
                    }
                }
                checking.set(false);
            });
        }
    });

    let check_now = move |_| {
        if !*checking.read() {
            checking.set(true);
            spawn(async move {
                status.set(UpdateStatus::Checking);
                match check_for_updates().await {
                    Ok(s) => status.set(s),
                    Err(e) => status.set(UpdateStatus::Error(e.to_string())),
                }
                checking.set(false);
            });
        }
    };

    let current_status = status.read().clone();

    match current_status {
        UpdateStatus::Unknown | UpdateStatus::Checking => {
            rsx! {
                div { class: "update-status checking",
                    span { class: "update-icon", "🔄" }
                    span { "Checking..." }
                }
            }
        }
        UpdateStatus::UpToDate => {
            rsx! {
                div { class: "update-status up-to-date",
                    onclick: check_now,
                    span { class: "update-icon", "✓" }
                    span { "v{CURRENT_VERSION}" }
                }
            }
        }
        UpdateStatus::Available(info) => {
            let url = info.release_url.clone();
            rsx! {
                div {
                    class: "update-status available",
                    onclick: move |_| open_release_page(&url),
                    span { class: "update-icon pulse", "⬆" }
                    span { "Update v{info.version}" }
                }
            }
        }
        UpdateStatus::Downloading(progress) => {
            rsx! {
                div { class: "update-status downloading",
                    span { class: "update-icon", "⏳" }
                    span { "Downloading {progress}%" }
                }
            }
        }
        UpdateStatus::ReadyToInstall(_) => {
            rsx! {
                div { class: "update-status ready",
                    span { class: "update-icon", "📦" }
                    span { "Ready to install" }
                }
            }
        }
        UpdateStatus::Error(_) => {
            rsx! {
                div {
                    class: "update-status error",
                    onclick: check_now,
                    span { class: "update-icon", "⚠" }
                    span { "v{CURRENT_VERSION}" }
                }
            }
        }
    }
}
