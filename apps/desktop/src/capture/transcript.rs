//! Transcript Types and Buffer
//!
//! Manages the stream of transcripts from STT services.

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;

/// A single transcript segment from STT
#[derive(Debug, Clone)]
pub struct TranscriptSegment {
    /// The transcribed text
    pub text: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Whether this is a final result or interim
    pub is_final: bool,
    /// Speaker ID if diarization is enabled
    pub speaker: Option<String>,
    /// When this segment was received
    pub timestamp: DateTime<Utc>,
}

/// Buffer for managing transcript segments
///
/// Handles merging interim and final results, maintaining conversation history.
#[derive(Debug)]
pub struct TranscriptBuffer {
    /// Final segments (confirmed)
    segments: Arc<RwLock<VecDeque<TranscriptSegment>>>,
    /// Current interim segment (may change)
    interim: Arc<RwLock<Option<TranscriptSegment>>>,
    /// Maximum number of segments to keep
    max_segments: usize,
}

impl Default for TranscriptBuffer {
    fn default() -> Self {
        Self::new(100)
    }
}

impl TranscriptBuffer {
    /// Create a new transcript buffer
    pub fn new(max_segments: usize) -> Self {
        Self {
            segments: Arc::new(RwLock::new(VecDeque::with_capacity(max_segments))),
            interim: Arc::new(RwLock::new(None)),
            max_segments,
        }
    }

    /// Add a new transcript segment
    pub fn add(&self, segment: TranscriptSegment) {
        if segment.is_final {
            // Clear interim and add to final segments
            *self.interim.write() = None;

            let mut segments = self.segments.write();
            segments.push_back(segment);

            // Trim if needed
            while segments.len() > self.max_segments {
                segments.pop_front();
            }
        } else {
            // Update interim
            *self.interim.write() = Some(segment);
        }
    }

    /// Get the current transcript text (including interim)
    pub fn get_current_text(&self) -> String {
        let segments = self.segments.read();
        let interim = self.interim.read();

        let mut parts: Vec<&str> = segments.iter().map(|s| s.text.as_str()).collect();

        if let Some(interim_seg) = interim.as_ref() {
            parts.push(&interim_seg.text);
        }

        parts.join(" ")
    }

    /// Get only the final transcript text
    pub fn get_final_text(&self) -> String {
        let segments = self.segments.read();
        segments
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Get the most recent segment (final or interim)
    pub fn get_latest(&self) -> Option<TranscriptSegment> {
        // Check interim first
        if let Some(interim) = self.interim.read().clone() {
            return Some(interim);
        }

        // Then check final segments
        self.segments.read().back().cloned()
    }

    /// Get the last N characters of the transcript
    pub fn get_tail(&self, char_limit: usize) -> String {
        let text = self.get_current_text();
        if text.len() <= char_limit {
            text
        } else {
            format!("...{}", &text[text.len() - char_limit..])
        }
    }

    /// Get all final segments
    pub fn get_segments(&self) -> Vec<TranscriptSegment> {
        self.segments.read().iter().cloned().collect()
    }

    /// Clear all segments
    pub fn clear(&self) {
        self.segments.write().clear();
        *self.interim.write() = None;
    }

    /// Check if there's any content
    pub fn is_empty(&self) -> bool {
        self.segments.read().is_empty() && self.interim.read().is_none()
    }

    /// Get the number of final segments
    pub fn len(&self) -> usize {
        self.segments.read().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcript_buffer() {
        let buffer = TranscriptBuffer::new(10);

        // Add interim
        buffer.add(TranscriptSegment {
            text: "Hello".to_string(),
            confidence: 0.8,
            is_final: false,
            speaker: None,
            timestamp: Utc::now(),
        });

        assert_eq!(buffer.get_current_text(), "Hello");
        assert_eq!(buffer.get_final_text(), "");

        // Update interim
        buffer.add(TranscriptSegment {
            text: "Hello world".to_string(),
            confidence: 0.9,
            is_final: false,
            speaker: None,
            timestamp: Utc::now(),
        });

        assert_eq!(buffer.get_current_text(), "Hello world");

        // Finalize
        buffer.add(TranscriptSegment {
            text: "Hello world!".to_string(),
            confidence: 0.95,
            is_final: true,
            speaker: None,
            timestamp: Utc::now(),
        });

        assert_eq!(buffer.get_final_text(), "Hello world!");
        assert_eq!(buffer.len(), 1);
    }
}
