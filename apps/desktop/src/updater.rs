//! Auto-Update System
//!
//! Checks GitHub releases for new versions and handles updates.

use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::path::PathBuf;

/// GitHub repository for updates
const GITHUB_OWNER: &str = "AhmediHarhash";
const GITHUB_REPO: &str = "voice-copilot";

/// Current version from Cargo.toml
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// GitHub release information
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub html_url: String,
    pub published_at: String,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

/// Update status
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateStatus {
    /// Haven't checked yet
    Unknown,
    /// Currently checking
    Checking,
    /// Up to date
    UpToDate,
    /// New version available
    Available(UpdateInfo),
    /// Currently downloading
    Downloading(u8), // percentage
    /// Ready to install
    ReadyToInstall(PathBuf),
    /// Error occurred
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateInfo {
    pub version: String,
    pub release_notes: String,
    pub download_url: String,
    pub release_url: String,
    pub size_mb: f64,
}

/// Check for updates from GitHub
pub async fn check_for_updates() -> Result<UpdateStatus> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        GITHUB_OWNER, GITHUB_REPO
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "voice-copilot")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?;

    if response.status() == 404 {
        // No releases yet
        return Ok(UpdateStatus::UpToDate);
    }

    if !response.status().is_success() {
        return Err(anyhow!("GitHub API error: {}", response.status()));
    }

    let release: GitHubRelease = response.json().await?;

    // Parse version (remove 'v' prefix if present)
    let latest_version = release.tag_name.trim_start_matches('v');

    // Compare versions
    if is_newer_version(latest_version, CURRENT_VERSION) {
        // Find Windows executable asset
        let asset = release.assets.iter().find(|a| {
            a.name.ends_with(".exe") || a.name.ends_with(".msi") || a.name.contains("windows")
        });

        let (download_url, size) = match asset {
            Some(a) => (a.browser_download_url.clone(), a.size as f64 / 1_048_576.0),
            None => (release.html_url.clone(), 0.0),
        };

        Ok(UpdateStatus::Available(UpdateInfo {
            version: latest_version.to_string(),
            release_notes: release.body,
            download_url,
            release_url: release.html_url,
            size_mb: size,
        }))
    } else {
        Ok(UpdateStatus::UpToDate)
    }
}

/// Compare version strings (e.g., "0.2.0" > "0.1.0")
fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse_version = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };

    let latest_parts = parse_version(latest);
    let current_parts = parse_version(current);

    for i in 0..3 {
        let l = latest_parts.get(i).copied().unwrap_or(0);
        let c = current_parts.get(i).copied().unwrap_or(0);

        if l > c {
            return true;
        } else if l < c {
            return false;
        }
    }

    false
}

/// Download update to temp folder
pub async fn download_update(
    url: &str,
    progress_callback: impl Fn(u8) + Send + 'static,
) -> Result<PathBuf> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "voice-copilot")
        .send()
        .await?;

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    // Create temp file
    let temp_dir = std::env::temp_dir();
    let file_name = url.split('/').last().unwrap_or("voice-copilot-update.exe");
    let file_path = temp_dir.join(file_name);

    let mut file = tokio::fs::File::create(&file_path).await?;
    let mut stream = response.bytes_stream();

    use futures::StreamExt;
    use tokio::io::AsyncWriteExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;

        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let progress = ((downloaded as f64 / total_size as f64) * 100.0) as u8;
            progress_callback(progress);
        }
    }

    file.flush().await?;

    Ok(file_path)
}

/// Open the release page in browser
pub fn open_release_page(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}

/// Install update (Windows)
#[cfg(target_os = "windows")]
pub fn install_update(installer_path: &PathBuf) -> Result<()> {
    use std::process::Command;

    // If it's an MSI, use msiexec
    if installer_path.extension().map_or(false, |e| e == "msi") {
        Command::new("msiexec")
            .args(["/i", installer_path.to_str().unwrap(), "/passive"])
            .spawn()?;
    } else {
        // If it's an exe, just run it
        Command::new(installer_path).spawn()?;
    }

    // Exit current instance
    std::process::exit(0);
}

#[cfg(not(target_os = "windows"))]
pub fn install_update(_installer_path: &PathBuf) -> Result<()> {
    Err(anyhow!("Auto-install not supported on this platform. Please install manually."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("0.2.0", "0.1.0"));
        assert!(is_newer_version("1.0.0", "0.9.9"));
        assert!(is_newer_version("0.1.1", "0.1.0"));
        assert!(!is_newer_version("0.1.0", "0.1.0"));
        assert!(!is_newer_version("0.1.0", "0.2.0"));
    }
}
