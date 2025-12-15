//! Document chunking
//!
//! Splits documents into ~500 token chunks with overlap.

/// Chunk configuration
pub struct ChunkConfig {
    /// Target tokens per chunk
    pub target_tokens: usize,
    /// Overlap between chunks (tokens)
    pub overlap_tokens: usize,
    /// Minimum chunk size (characters)
    pub min_chars: usize,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            target_tokens: 500,
            overlap_tokens: 50,
            min_chars: 100,
        }
    }
}

/// A chunk of text from a document
#[derive(Debug, Clone)]
pub struct Chunk {
    pub index: usize,
    pub content: String,
    pub char_start: usize,
    pub char_end: usize,
    pub estimated_tokens: usize,
}

/// Estimate token count (rough: ~4 chars per token for English)
fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

/// Split text into chunks
pub fn chunk_document(text: &str, config: &ChunkConfig) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let target_chars = config.target_tokens * 4;
    let overlap_chars = config.overlap_tokens * 4;

    // Split into paragraphs first
    let paragraphs: Vec<&str> = text
        .split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .collect();

    let mut current_chunk = String::new();
    let mut chunk_start = 0;
    let mut char_offset = 0;

    for para in paragraphs {
        let para_with_break = format!("{}\n\n", para.trim());

        // If adding this paragraph exceeds target, finalize current chunk
        if !current_chunk.is_empty()
            && current_chunk.len() + para_with_break.len() > target_chars
        {
            // Save current chunk
            if current_chunk.len() >= config.min_chars {
                chunks.push(Chunk {
                    index: chunks.len(),
                    content: current_chunk.trim().to_string(),
                    char_start: chunk_start,
                    char_end: char_offset,
                    estimated_tokens: estimate_tokens(&current_chunk),
                });
            }

            // Start new chunk with overlap
            let overlap_start = current_chunk
                .len()
                .saturating_sub(overlap_chars);
            current_chunk = current_chunk[overlap_start..].to_string();
            chunk_start = char_offset.saturating_sub(overlap_chars);
        }

        current_chunk.push_str(&para_with_break);
        char_offset += para_with_break.len();
    }

    // Don't forget the last chunk
    if current_chunk.len() >= config.min_chars {
        chunks.push(Chunk {
            index: chunks.len(),
            content: current_chunk.trim().to_string(),
            char_start: chunk_start,
            char_end: char_offset,
            estimated_tokens: estimate_tokens(&current_chunk),
        });
    }

    chunks
}

/// Chunk text by sentences for more precise splitting
pub fn chunk_by_sentences(text: &str, config: &ChunkConfig) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let target_chars = config.target_tokens * 4;
    let overlap_chars = config.overlap_tokens * 4;

    // Simple sentence splitting (could use more sophisticated NLP)
    let sentences: Vec<&str> = text
        .split(|c| c == '.' || c == '!' || c == '?')
        .filter(|s| !s.trim().is_empty())
        .collect();

    let mut current_chunk = String::new();
    let mut chunk_start = 0;
    let mut char_offset = 0;

    for sentence in sentences {
        let sentence_with_period = format!("{}. ", sentence.trim());

        if !current_chunk.is_empty()
            && current_chunk.len() + sentence_with_period.len() > target_chars
        {
            if current_chunk.len() >= config.min_chars {
                chunks.push(Chunk {
                    index: chunks.len(),
                    content: current_chunk.trim().to_string(),
                    char_start: chunk_start,
                    char_end: char_offset,
                    estimated_tokens: estimate_tokens(&current_chunk),
                });
            }

            let overlap_start = current_chunk.len().saturating_sub(overlap_chars);
            current_chunk = current_chunk[overlap_start..].to_string();
            chunk_start = char_offset.saturating_sub(overlap_chars);
        }

        current_chunk.push_str(&sentence_with_period);
        char_offset += sentence_with_period.len();
    }

    if current_chunk.len() >= config.min_chars {
        chunks.push(Chunk {
            index: chunks.len(),
            content: current_chunk.trim().to_string(),
            char_start: chunk_start,
            char_end: char_offset,
            estimated_tokens: estimate_tokens(&current_chunk),
        });
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_document() {
        let text = "First paragraph with some content.\n\nSecond paragraph.\n\nThird one.";
        let config = ChunkConfig {
            target_tokens: 10,
            overlap_tokens: 2,
            min_chars: 10,
        };

        let chunks = chunk_document(text, &config);
        assert!(!chunks.is_empty());
    }
}
