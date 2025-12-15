//! RAG (Retrieval-Augmented Generation) Module
//!
//! Implements hybrid search combining:
//! - Vector similarity (pgvector with HNSW)
//! - Full-text search (PostgreSQL tsvector)
//! - Reciprocal Rank Fusion for merging results

pub mod chunker;
pub mod embeddings;
pub mod search;
pub mod synthesis;

pub use chunker::chunk_document;
pub use embeddings::generate_embeddings;
pub use search::hybrid_search;
pub use synthesis::generate_hints;
