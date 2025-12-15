-- Hekax Database Schema
-- Migration 001: Initial schema

-- Enable extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "vector";

-- =============================================================================
-- USERS & AUTH
-- =============================================================================

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255),  -- Argon2id, NULL for OAuth-only users
    google_id VARCHAR(255) UNIQUE,
    full_name VARCHAR(255),
    avatar_url TEXT,
    subscription_tier VARCHAR(50) DEFAULT 'free',  -- free, pro, enterprise
    subscription_expires_at TIMESTAMPTZ,
    token_version INTEGER DEFAULT 0,  -- Increment to invalidate all refresh tokens
    email_verified BOOLEAN DEFAULT false,
    -- Internationalization
    locale VARCHAR(10) DEFAULT 'en-US',
    timezone VARCHAR(50) DEFAULT 'UTC',
    data_region VARCHAR(20) DEFAULT 'us-east',  -- For GDPR compliance
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_google_id ON users(google_id) WHERE google_id IS NOT NULL;

-- Refresh tokens (hashed, with rotation)
CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,  -- SHA256 of actual token
    device_id VARCHAR(255) NOT NULL,
    device_name VARCHAR(255),
    token_version INTEGER NOT NULL,  -- Must match user.token_version to be valid
    expires_at TIMESTAMPTZ NOT NULL,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, device_id)  -- One token per device
);

CREATE INDEX idx_refresh_tokens_user ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_expires ON refresh_tokens(expires_at);

-- User settings (with encrypted API keys)
CREATE TABLE user_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID UNIQUE NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    api_keys_encrypted BYTEA,  -- AES-256-GCM encrypted JSON blob
    default_mode VARCHAR(50) DEFAULT 'sales',
    auto_record BOOLEAN DEFAULT true,
    stealth_mode_default BOOLEAN DEFAULT false,
    theme VARCHAR(50) DEFAULT 'dark',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- LEADS
-- =============================================================================

CREATE TABLE leads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Company info
    company_name VARCHAR(255) NOT NULL,
    company_domain VARCHAR(255),
    company_linkedin VARCHAR(255),
    company_size VARCHAR(50),  -- 1-10, 11-50, 51-200, 201-500, 501-1000, 1000+
    industry VARCHAR(100),
    location VARCHAR(255),

    -- Contact info
    contact_name VARCHAR(255),
    contact_title VARCHAR(255),
    contact_email VARCHAR(255),
    contact_phone VARCHAR(50),
    contact_linkedin VARCHAR(255),

    -- Status tracking
    status VARCHAR(50) DEFAULT 'new',  -- new, researching, contacted, qualified, proposal, negotiation, won, lost
    priority INTEGER DEFAULT 0,  -- 0-5, higher = more important
    estimated_value DECIMAL(12,2),  -- Deal value in USD

    -- Enrichment data
    tech_stack JSONB DEFAULT '[]',
    funding_info JSONB,  -- {stage, amount, date, investors}
    recent_news JSONB DEFAULT '[]',
    employee_count INTEGER,

    -- Metadata
    source VARCHAR(100),  -- apollo, linkedin, manual, referral
    tags TEXT[],
    notes TEXT,
    custom_fields JSONB DEFAULT '{}',

    -- Timeline
    last_contacted_at TIMESTAMPTZ,
    next_followup_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_leads_user_status ON leads(user_id, status);
CREATE INDEX idx_leads_user_created ON leads(user_id, created_at DESC);
CREATE INDEX idx_leads_user_priority ON leads(user_id, priority DESC);
CREATE INDEX idx_leads_company ON leads(company_name);

-- =============================================================================
-- RECORDINGS
-- =============================================================================

CREATE TABLE recordings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    lead_id UUID REFERENCES leads(id) ON DELETE SET NULL,

    -- Session info
    mode VARCHAR(50) NOT NULL,  -- sales, interview, technical, etc.
    status VARCHAR(50) DEFAULT 'processing',  -- recording, processing, completed, failed

    -- Timing
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ,
    duration_seconds INTEGER,

    -- Content (normalized, small)
    transcript_turns JSONB,  -- [{speaker, text, timestamp_ms, duration_ms}]
    summary TEXT,
    key_points JSONB DEFAULT '[]',
    action_items JSONB DEFAULT '[]',

    -- Metrics
    talk_ratio FLOAT,  -- User talk time / total talk time
    user_word_count INTEGER,
    other_word_count INTEGER,
    user_wpm FLOAT,  -- Words per minute
    question_count INTEGER,
    objection_count INTEGER,
    sentiment_score FLOAT,  -- -1 to 1

    -- Performance analysis
    performance_score JSONB,  -- {overall, listening, response_quality, delivery, outcome}
    outcome VARCHAR(50),  -- achieved, partial, not_achieved, unknown

    -- Storage (large files in R2)
    audio_r2_key VARCHAR(500),
    transcript_r2_key VARCHAR(500),

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_recordings_user_time ON recordings(user_id, start_time DESC);
CREATE INDEX idx_recordings_lead ON recordings(lead_id) WHERE lead_id IS NOT NULL;
CREATE INDEX idx_recordings_status ON recordings(status);

-- =============================================================================
-- DOCUMENTS (Knowledge Base for RAG)
-- =============================================================================

CREATE TABLE documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    lead_id UUID REFERENCES leads(id) ON DELETE SET NULL,  -- Optional: link to specific lead

    title VARCHAR(255) NOT NULL,
    document_type VARCHAR(50),  -- pdf, doc, notes, research, website_scrape
    original_filename VARCHAR(255),
    r2_key VARCHAR(500),  -- File storage key
    file_size_bytes BIGINT,

    -- Content for search
    content_text TEXT,  -- Extracted text
    search_vector tsvector,  -- Full-text search

    -- RAG indexing status
    is_indexed BOOLEAN DEFAULT false,
    chunk_count INTEGER DEFAULT 0,
    embedding_model VARCHAR(50),
    indexed_at TIMESTAMPTZ,

    -- Metadata
    mode VARCHAR(50),  -- sales, interview, technical (for filtering)
    tags TEXT[],

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_documents_user ON documents(user_id);
CREATE INDEX idx_documents_lead ON documents(lead_id) WHERE lead_id IS NOT NULL;
CREATE INDEX idx_documents_fts ON documents USING gin(search_vector);

-- Document chunks for RAG
CREATE TABLE document_chunks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,  -- Denormalized for efficient filtering
    lead_id UUID,  -- Denormalized for efficient filtering

    chunk_index INTEGER NOT NULL,
    content TEXT NOT NULL,
    token_count INTEGER,

    -- Vector embedding
    embedding VECTOR(1536),  -- text-embedding-3-small dimension
    embedding_model VARCHAR(50) DEFAULT 'text-embedding-3-small',

    -- Full-text search
    search_vector tsvector,

    -- Metadata for filtering
    doc_type VARCHAR(50),
    mode VARCHAR(50),  -- sales, interview, technical

    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- HNSW index for fast vector similarity search
CREATE INDEX idx_chunks_embedding ON document_chunks
USING hnsw (embedding vector_cosine_ops)
WITH (m = 16, ef_construction = 64);

-- GIN index for full-text search
CREATE INDEX idx_chunks_fts ON document_chunks USING gin(search_vector);

-- Filter indexes
CREATE INDEX idx_chunks_user_lead ON document_chunks(user_id, lead_id);
CREATE INDEX idx_chunks_user_mode ON document_chunks(user_id, mode);

-- =============================================================================
-- SUGGESTION LOGS (Audit + Evaluation)
-- =============================================================================

CREATE TABLE suggestion_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    recording_id UUID REFERENCES recordings(id) ON DELETE SET NULL,
    lead_id UUID REFERENCES leads(id) ON DELETE SET NULL,

    -- What was generated
    suggestion_type VARCHAR(50) NOT NULL,  -- flash, deep, hint
    content TEXT NOT NULL,

    -- How it was generated
    model_used VARCHAR(100) NOT NULL,
    prompt_tokens INTEGER,
    completion_tokens INTEGER,
    latency_ms INTEGER,
    cost_usd DECIMAL(10,6),

    -- RAG context
    retrieved_chunk_ids UUID[],
    retrieval_scores FLOAT[],

    -- Evaluation
    evaluator_model VARCHAR(100),
    evaluator_score FLOAT,  -- 0-1
    evaluator_reasoning TEXT,

    -- User feedback
    user_feedback VARCHAR(20),  -- helpful, not_helpful, wrong, offensive
    feedback_text TEXT,
    feedback_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_suggestions_user_time ON suggestion_logs(user_id, created_at DESC);
CREATE INDEX idx_suggestions_recording ON suggestion_logs(recording_id) WHERE recording_id IS NOT NULL;
CREATE INDEX idx_suggestions_feedback ON suggestion_logs(user_feedback) WHERE user_feedback IS NOT NULL;

-- =============================================================================
-- SYNC EVENTS (Event Sourcing for Offline Support)
-- =============================================================================

CREATE TABLE sync_events (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    entity_type VARCHAR(50) NOT NULL,  -- lead, recording, document
    entity_id UUID NOT NULL,
    event_type VARCHAR(50) NOT NULL,  -- created, updated, deleted
    payload JSONB NOT NULL,
    version BIGINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_sync_user_id ON sync_events(user_id, id);
CREATE INDEX idx_sync_entity ON sync_events(entity_type, entity_id);

-- =============================================================================
-- JOBS (Async Task Queue)
-- =============================================================================

CREATE TABLE jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    job_type VARCHAR(50) NOT NULL,  -- scrape_apollo, enrich_lead, index_document, generate_summary
    status VARCHAR(50) DEFAULT 'pending',  -- pending, running, completed, failed, cancelled
    priority INTEGER DEFAULT 0,

    -- Input/Output
    input_params JSONB NOT NULL,
    result JSONB,
    error_message TEXT,

    -- Timing
    scheduled_at TIMESTAMPTZ DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,

    -- Retry handling
    attempt_count INTEGER DEFAULT 0,
    max_attempts INTEGER DEFAULT 3,
    next_retry_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_jobs_status ON jobs(status, scheduled_at) WHERE status IN ('pending', 'running');
CREATE INDEX idx_jobs_user ON jobs(user_id, created_at DESC);

-- =============================================================================
-- ACTIVITY LOG
-- =============================================================================

CREATE TABLE activity_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    activity_type VARCHAR(50) NOT NULL,  -- login, lead_created, call_started, document_uploaded, etc.
    entity_type VARCHAR(50),  -- lead, recording, document
    entity_id UUID,
    metadata JSONB DEFAULT '{}',

    ip_address INET,
    user_agent TEXT,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_activity_user_time ON activity_log(user_id, created_at DESC);
CREATE INDEX idx_activity_type ON activity_log(activity_type, created_at DESC);

-- =============================================================================
-- HELPER FUNCTIONS
-- =============================================================================

-- Auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply to tables with updated_at
CREATE TRIGGER users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER user_settings_updated_at BEFORE UPDATE ON user_settings
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER leads_updated_at BEFORE UPDATE ON leads
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- Auto-generate search_vector for documents
CREATE OR REPLACE FUNCTION update_document_search_vector()
RETURNS TRIGGER AS $$
BEGIN
    NEW.search_vector = to_tsvector('english', COALESCE(NEW.title, '') || ' ' || COALESCE(NEW.content_text, ''));
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER documents_search_vector BEFORE INSERT OR UPDATE ON documents
    FOR EACH ROW EXECUTE FUNCTION update_document_search_vector();

-- Auto-generate search_vector for chunks
CREATE OR REPLACE FUNCTION update_chunk_search_vector()
RETURNS TRIGGER AS $$
BEGIN
    NEW.search_vector = to_tsvector('english', COALESCE(NEW.content, ''));
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER chunks_search_vector BEFORE INSERT OR UPDATE ON document_chunks
    FOR EACH ROW EXECUTE FUNCTION update_chunk_search_vector();

-- =============================================================================
-- INTERNATIONALIZATION (i18n)
-- =============================================================================

CREATE INDEX idx_users_locale ON users(locale);
CREATE INDEX idx_users_data_region ON users(data_region);

-- Translations table for multi-language support
CREATE TABLE translations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    locale VARCHAR(10) NOT NULL,        -- en-US, ar-SA, fr-FR, es-ES, de-DE, pt-BR, hi-IN, zh-CN
    namespace VARCHAR(50) NOT NULL,      -- common, dashboard, settings, leads, recordings, hints
    key VARCHAR(255) NOT NULL,
    value TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(locale, namespace, key)
);

CREATE INDEX idx_translations_lookup ON translations(locale, namespace);

-- Supported locales reference table
CREATE TABLE supported_locales (
    code VARCHAR(10) PRIMARY KEY,        -- en-US, ar-SA, etc.
    name VARCHAR(100) NOT NULL,          -- English (US), Arabic (Saudi Arabia)
    native_name VARCHAR(100) NOT NULL,   -- English, العربية
    direction VARCHAR(3) DEFAULT 'ltr',  -- ltr or rtl
    is_active BOOLEAN DEFAULT true,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert default supported locales
INSERT INTO supported_locales (code, name, native_name, direction, sort_order) VALUES
    ('en-US', 'English (US)', 'English', 'ltr', 1),
    ('es-ES', 'Spanish', 'Español', 'ltr', 2),
    ('ar-SA', 'Arabic', 'العربية', 'rtl', 3),
    ('fr-FR', 'French', 'Français', 'ltr', 4),
    ('de-DE', 'German', 'Deutsch', 'ltr', 5),
    ('pt-BR', 'Portuguese (Brazil)', 'Português', 'ltr', 6),
    ('hi-IN', 'Hindi', 'हिन्दी', 'ltr', 7),
    ('zh-CN', 'Chinese (Simplified)', '简体中文', 'ltr', 8);

-- =============================================================================
-- API KEYS & INTEGRATIONS
-- =============================================================================

-- Store user's third-party API keys securely
CREATE TABLE user_api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    service VARCHAR(50) NOT NULL,        -- openai, anthropic, apollo, etc.
    key_encrypted BYTEA NOT NULL,        -- AES-256-GCM encrypted
    key_hint VARCHAR(20),                -- Last 4 chars for display
    is_valid BOOLEAN DEFAULT true,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, service)
);

CREATE INDEX idx_user_api_keys_user ON user_api_keys(user_id);

-- =============================================================================
-- SUBSCRIPTION & BILLING
-- =============================================================================

CREATE TABLE subscription_plans (
    id VARCHAR(50) PRIMARY KEY,          -- free, pro, enterprise
    name VARCHAR(100) NOT NULL,
    price_monthly_usd DECIMAL(10,2),
    price_yearly_usd DECIMAL(10,2),

    -- Limits
    max_recordings_per_month INTEGER,
    max_leads INTEGER,
    max_documents INTEGER,
    max_storage_gb INTEGER,
    max_api_calls_per_day INTEGER,

    -- Features
    features JSONB DEFAULT '[]',          -- ["real_time_hints", "priority_support", etc.]

    is_active BOOLEAN DEFAULT true,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert default plans
INSERT INTO subscription_plans (id, name, price_monthly_usd, price_yearly_usd, max_recordings_per_month, max_leads, max_documents, max_storage_gb, max_api_calls_per_day, features, sort_order) VALUES
    ('free', 'Free', 0, 0, 10, 50, 20, 1, 100, '["basic_hints"]', 1),
    ('pro', 'Professional', 49, 490, 100, 500, 200, 10, 1000, '["basic_hints", "real_time_hints", "deep_analysis", "priority_support"]', 2),
    ('enterprise', 'Enterprise', 199, 1990, -1, -1, -1, 100, -1, '["basic_hints", "real_time_hints", "deep_analysis", "priority_support", "custom_modes", "api_access", "team_features"]', 3);

-- User subscription history
CREATE TABLE subscription_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_id VARCHAR(50) NOT NULL REFERENCES subscription_plans(id),
    status VARCHAR(50) NOT NULL,         -- active, cancelled, expired, trial

    started_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,

    -- Payment info
    payment_provider VARCHAR(50),         -- stripe, paddle, etc.
    payment_id VARCHAR(255),

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_subscription_history_user ON subscription_history(user_id, created_at DESC);

-- =============================================================================
-- USAGE TRACKING
-- =============================================================================

CREATE TABLE usage_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    period_start DATE NOT NULL,          -- Monthly period start

    -- Counts
    recordings_count INTEGER DEFAULT 0,
    leads_count INTEGER DEFAULT 0,
    documents_count INTEGER DEFAULT 0,
    api_calls_count INTEGER DEFAULT 0,

    -- Storage
    storage_used_bytes BIGINT DEFAULT 0,

    -- AI usage
    ai_tokens_used INTEGER DEFAULT 0,
    ai_cost_usd DECIMAL(10,4) DEFAULT 0,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, period_start)
);

CREATE INDEX idx_usage_user_period ON usage_metrics(user_id, period_start DESC);
