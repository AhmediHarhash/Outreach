//! Recording Storage
//!
//! Save and load recordings to/from disk.

use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use tokio::fs;

use super::session::RecordingSession;

/// Get the recordings directory
pub fn recordings_dir() -> PathBuf {
    let base = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("VoiceCopilot").join("recordings")
}

/// Save a recording to disk
pub async fn save_recording(session: &RecordingSession) -> Result<PathBuf> {
    let dir = recordings_dir();
    fs::create_dir_all(&dir).await
        .context("Failed to create recordings directory")?;

    let filename = format!(
        "{}_{}.json",
        session.start_time.format("%Y%m%d_%H%M%S"),
        &session.id[..8]
    );
    let path = dir.join(&filename);

    let json = serde_json::to_string_pretty(session)
        .context("Failed to serialize recording")?;

    fs::write(&path, json).await
        .context("Failed to write recording file")?;

    tracing::info!("Recording saved to: {:?}", path);
    Ok(path)
}

/// Load a recording from disk
pub async fn load_recording(id: &str) -> Result<RecordingSession> {
    let dir = recordings_dir();

    // Find the file by ID
    let mut entries = fs::read_dir(&dir).await
        .context("Failed to read recordings directory")?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.contains(id) {
                let content = fs::read_to_string(&path).await
                    .context("Failed to read recording file")?;
                let session: RecordingSession = serde_json::from_str(&content)
                    .context("Failed to parse recording")?;
                return Ok(session);
            }
        }
    }

    anyhow::bail!("Recording not found: {}", id)
}

/// List all recordings
pub async fn list_recordings() -> Result<Vec<RecordingInfo>> {
    let dir = recordings_dir();

    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = fs::read_dir(&dir).await
        .context("Failed to read recordings directory")?;

    let mut recordings = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            if let Ok(content) = fs::read_to_string(&path).await {
                if let Ok(session) = serde_json::from_str::<RecordingSession>(&content) {
                    recordings.push(RecordingInfo {
                        id: session.id,
                        mode: session.mode,
                        start_time: session.start_time,
                        duration_mins: session.duration().num_minutes() as u32,
                        turn_count: session.turns.len(),
                        path,
                    });
                }
            }
        }
    }

    // Sort by date, newest first
    recordings.sort_by(|a, b| b.start_time.cmp(&a.start_time));

    Ok(recordings)
}

/// Delete a recording
pub async fn delete_recording(id: &str) -> Result<()> {
    let dir = recordings_dir();

    let mut entries = fs::read_dir(&dir).await
        .context("Failed to read recordings directory")?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.contains(id) {
                fs::remove_file(&path).await
                    .context("Failed to delete recording")?;
                tracing::info!("Recording deleted: {:?}", path);
                return Ok(());
            }
        }
    }

    anyhow::bail!("Recording not found: {}", id)
}

/// Recording info for list display
#[derive(Debug, Clone)]
pub struct RecordingInfo {
    pub id: String,
    pub mode: String,
    pub start_time: DateTime<Utc>,
    pub duration_mins: u32,
    pub turn_count: usize,
    pub path: PathBuf,
}

impl RecordingInfo {
    pub fn display_name(&self) -> String {
        format!(
            "{} - {} ({} min, {} turns)",
            self.start_time.format("%Y-%m-%d %H:%M"),
            self.mode,
            self.duration_mins,
            self.turn_count
        )
    }
}

/// Export recording to different formats
pub async fn export_recording(session: &RecordingSession, format: ExportFormat) -> Result<String> {
    match format {
        ExportFormat::Json => {
            serde_json::to_string_pretty(session)
                .context("Failed to serialize to JSON")
        }
        ExportFormat::Markdown => {
            Ok(export_markdown(session))
        }
        ExportFormat::PlainText => {
            Ok(export_plain_text(session))
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Json,
    Markdown,
    PlainText,
}

fn export_markdown(session: &RecordingSession) -> String {
    let mut md = String::new();

    md.push_str(&format!("# Call Recording: {}\n\n", session.mode));
    md.push_str(&format!("**Date:** {}\n", session.start_time.format("%Y-%m-%d %H:%M")));
    md.push_str(&format!("**Duration:** {} minutes\n", session.duration().num_minutes()));
    md.push_str(&format!("**Mode:** {}\n\n", session.mode));

    md.push_str("## Transcript\n\n");
    for turn in &session.turns {
        md.push_str(&format!("**{}:** {}\n\n", turn.speaker.label(), turn.text));
    }

    if !session.suggestions.is_empty() {
        md.push_str("## AI Suggestions\n\n");
        for suggestion in &session.suggestions {
            let used = if suggestion.was_used { " (used)" } else { "" };
            md.push_str(&format!("- **{:?}**{}: {}\n", suggestion.suggestion_type, used, suggestion.content));
        }
    }

    md.push_str("\n---\n");
    md.push_str("*Recorded with Voice Copilot*\n");

    md
}

fn export_plain_text(session: &RecordingSession) -> String {
    let mut txt = String::new();

    txt.push_str(&format!("CALL RECORDING: {}\n", session.mode.to_uppercase()));
    txt.push_str(&format!("Date: {}\n", session.start_time.format("%Y-%m-%d %H:%M")));
    txt.push_str(&format!("Duration: {} minutes\n", session.duration().num_minutes()));
    txt.push_str("\n");
    txt.push_str(&"=".repeat(50));
    txt.push_str("\n\nTRANSCRIPT:\n\n");

    for turn in &session.turns {
        txt.push_str(&format!("{}: {}\n\n", turn.speaker.label().to_uppercase(), turn.text));
    }

    txt.push_str(&"=".repeat(50));
    txt.push_str("\n");

    txt
}
