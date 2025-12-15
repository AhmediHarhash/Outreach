//! Embedding generation using OpenAI text-embedding-3-small
//!
//! The latest embedding model with 1536 dimensions.

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const EMBEDDING_MODEL: &str = "text-embedding-3-small";
const EMBEDDING_DIMENSION: usize = 1536;
const OPENAI_EMBEDDINGS_URL: &str = "https://api.openai.com/v1/embeddings";

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
    usage: EmbeddingUsage,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
    index: usize,
}

#[derive(Debug, Deserialize)]
struct EmbeddingUsage {
    prompt_tokens: i32,
    total_tokens: i32,
}

/// Result of embedding generation
#[derive(Debug)]
pub struct EmbeddingResult {
    pub embeddings: Vec<Vec<f32>>,
    pub model: String,
    pub total_tokens: i32,
}

/// Generate embeddings for a list of texts
pub async fn generate_embeddings(
    texts: &[String],
    api_key: &str,
) -> Result<EmbeddingResult> {
    if texts.is_empty() {
        return Ok(EmbeddingResult {
            embeddings: Vec::new(),
            model: EMBEDDING_MODEL.to_string(),
            total_tokens: 0,
        });
    }

    let client = Client::new();

    let request = EmbeddingRequest {
        model: EMBEDDING_MODEL.to_string(),
        input: texts.to_vec(),
    };

    let response = client
        .post(OPENAI_EMBEDDINGS_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("OpenAI embeddings API error: {}", error_text);
    }

    let result: EmbeddingResponse = response.json().await?;

    // Sort by index to maintain order
    let mut embeddings: Vec<(usize, Vec<f32>)> = result
        .data
        .into_iter()
        .map(|d| (d.index, d.embedding))
        .collect();
    embeddings.sort_by_key(|(idx, _)| *idx);

    Ok(EmbeddingResult {
        embeddings: embeddings.into_iter().map(|(_, e)| e).collect(),
        model: EMBEDDING_MODEL.to_string(),
        total_tokens: result.usage.total_tokens,
    })
}

/// Generate embedding for a single text
pub async fn generate_embedding(text: &str, api_key: &str) -> Result<Vec<f32>> {
    let result = generate_embeddings(&[text.to_string()], api_key).await?;
    result
        .embeddings
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No embedding returned"))
}

/// Batch process embeddings (max 2048 per request)
pub async fn batch_generate_embeddings(
    texts: &[String],
    api_key: &str,
    batch_size: usize,
) -> Result<Vec<Vec<f32>>> {
    let batch_size = batch_size.min(2048);
    let mut all_embeddings = Vec::with_capacity(texts.len());

    for chunk in texts.chunks(batch_size) {
        let result = generate_embeddings(chunk, api_key).await?;
        all_embeddings.extend(result.embeddings);
    }

    Ok(all_embeddings)
}

/// Get the embedding dimension
pub fn embedding_dimension() -> usize {
    EMBEDDING_DIMENSION
}

/// Get the model name
pub fn embedding_model() -> &'static str {
    EMBEDDING_MODEL
}
