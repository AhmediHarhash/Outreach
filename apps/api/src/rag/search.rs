//! Hybrid search combining vector similarity and full-text search
//!
//! Uses Reciprocal Rank Fusion (RRF) to merge results from both sources.

use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

/// Search result with combined score
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub chunk_id: Uuid,
    pub document_id: Uuid,
    pub content: String,
    pub vector_score: Option<f64>,
    pub fts_score: Option<f64>,
    pub rrf_score: f64,
}

/// Search configuration
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Number of results to return
    pub limit: i32,
    /// Filter by lead ID (optional)
    pub lead_id: Option<Uuid>,
    /// Filter by mode (optional)
    pub mode: Option<String>,
    /// RRF constant (typically 60)
    pub rrf_k: i32,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            limit: 5,
            lead_id: None,
            mode: None,
            rrf_k: 60,
        }
    }
}

/// Perform hybrid search combining vector and FTS
pub async fn hybrid_search(
    pool: &PgPool,
    user_id: Uuid,
    query_embedding: &[f32],
    query_text: &str,
    config: &SearchConfig,
) -> Result<Vec<SearchResult>> {
    // Convert embedding to PostgreSQL vector format
    let embedding_str = format!(
        "[{}]",
        query_embedding
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );

    // Hybrid search query using RRF
    let results = sqlx::query!(
        r#"
        WITH vector_results AS (
            SELECT
                id as chunk_id,
                document_id,
                content,
                1 - (embedding <=> $1::vector) as vector_score,
                ROW_NUMBER() OVER (ORDER BY embedding <=> $1::vector) as vector_rank
            FROM document_chunks
            WHERE user_id = $2
              AND ($4::uuid IS NULL OR lead_id = $4 OR lead_id IS NULL)
              AND ($5::text IS NULL OR mode = $5)
            ORDER BY embedding <=> $1::vector
            LIMIT 20
        ),
        fts_results AS (
            SELECT
                id as chunk_id,
                document_id,
                content,
                ts_rank(search_vector, plainto_tsquery('english', $3)) as fts_score,
                ROW_NUMBER() OVER (ORDER BY ts_rank(search_vector, plainto_tsquery('english', $3)) DESC) as fts_rank
            FROM document_chunks
            WHERE user_id = $2
              AND ($4::uuid IS NULL OR lead_id = $4 OR lead_id IS NULL)
              AND ($5::text IS NULL OR mode = $5)
              AND search_vector @@ plainto_tsquery('english', $3)
            ORDER BY fts_score DESC
            LIMIT 20
        )
        SELECT
            COALESCE(v.chunk_id, f.chunk_id) as "chunk_id!",
            COALESCE(v.document_id, f.document_id) as "document_id!",
            COALESCE(v.content, f.content) as "content!",
            v.vector_score as "vector_score?",
            f.fts_score as "fts_score?",
            v.vector_rank as "vector_rank?",
            f.fts_rank as "fts_rank?"
        FROM vector_results v
        FULL OUTER JOIN fts_results f ON v.chunk_id = f.chunk_id
        ORDER BY
            COALESCE(1.0 / ($6 + v.vector_rank), 0) +
            COALESCE(1.0 / ($6 + f.fts_rank), 0) DESC
        LIMIT $7
        "#,
        embedding_str,
        user_id,
        query_text,
        config.lead_id,
        config.mode,
        config.rrf_k as i64,
        config.limit as i64
    )
    .fetch_all(pool)
    .await?;

    Ok(results
        .into_iter()
        .enumerate()
        .map(|(idx, r)| {
            // Calculate RRF score
            let vector_rrf = r.vector_rank
                .map(|rank| 1.0 / (config.rrf_k as f64 + rank as f64))
                .unwrap_or(0.0);
            let fts_rrf = r.fts_rank
                .map(|rank| 1.0 / (config.rrf_k as f64 + rank as f64))
                .unwrap_or(0.0);

            SearchResult {
                chunk_id: r.chunk_id,
                document_id: r.document_id,
                content: r.content,
                vector_score: r.vector_score,
                fts_score: r.fts_score,
                rrf_score: vector_rrf + fts_rrf,
            }
        })
        .collect())
}

/// Vector-only search (for when FTS isn't needed)
pub async fn vector_search(
    pool: &PgPool,
    user_id: Uuid,
    query_embedding: &[f32],
    config: &SearchConfig,
) -> Result<Vec<SearchResult>> {
    let embedding_str = format!(
        "[{}]",
        query_embedding
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );

    let results = sqlx::query!(
        r#"
        SELECT
            id as chunk_id,
            document_id,
            content,
            1 - (embedding <=> $1::vector) as vector_score
        FROM document_chunks
        WHERE user_id = $2
          AND ($3::uuid IS NULL OR lead_id = $3 OR lead_id IS NULL)
          AND ($4::text IS NULL OR mode = $4)
        ORDER BY embedding <=> $1::vector
        LIMIT $5
        "#,
        embedding_str,
        user_id,
        config.lead_id,
        config.mode,
        config.limit as i64
    )
    .fetch_all(pool)
    .await?;

    Ok(results
        .into_iter()
        .map(|r| SearchResult {
            chunk_id: r.chunk_id,
            document_id: r.document_id,
            content: r.content,
            vector_score: r.vector_score,
            fts_score: None,
            rrf_score: r.vector_score.unwrap_or(0.0),
        })
        .collect())
}

/// Full-text search only
pub async fn fts_search(
    pool: &PgPool,
    user_id: Uuid,
    query_text: &str,
    config: &SearchConfig,
) -> Result<Vec<SearchResult>> {
    let results = sqlx::query!(
        r#"
        SELECT
            id as chunk_id,
            document_id,
            content,
            ts_rank(search_vector, plainto_tsquery('english', $2)) as fts_score
        FROM document_chunks
        WHERE user_id = $1
          AND ($3::uuid IS NULL OR lead_id = $3 OR lead_id IS NULL)
          AND ($4::text IS NULL OR mode = $4)
          AND search_vector @@ plainto_tsquery('english', $2)
        ORDER BY fts_score DESC
        LIMIT $5
        "#,
        user_id,
        query_text,
        config.lead_id,
        config.mode,
        config.limit as i64
    )
    .fetch_all(pool)
    .await?;

    Ok(results
        .into_iter()
        .map(|r| SearchResult {
            chunk_id: r.chunk_id,
            document_id: r.document_id,
            content: r.content,
            vector_score: None,
            fts_score: r.fts_score,
            rrf_score: r.fts_score.unwrap_or(0.0),
        })
        .collect())
}
