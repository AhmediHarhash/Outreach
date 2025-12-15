-- Outreach Database Schema
-- Migration 005: Lead Intelligence System

-- =============================================================================
-- ICP (Ideal Customer Profile)
-- =============================================================================

CREATE TABLE icp_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    name VARCHAR(100) NOT NULL,
    description TEXT,
    is_default BOOLEAN DEFAULT false,

    -- Industry targeting
    industries JSONB DEFAULT '[]',           -- ["SaaS", "FinTech", "HealthTech"]
    excluded_industries JSONB DEFAULT '[]',

    -- Company size
    company_size_min INTEGER,                -- Minimum employees
    company_size_max INTEGER,                -- Maximum employees
    revenue_min BIGINT,                      -- Minimum ARR in USD
    revenue_max BIGINT,

    -- Funding stage preferences
    funding_stages JSONB DEFAULT '[]',       -- ["Seed", "Series A", "Series B"]
    min_funding_amount BIGINT,               -- Minimum total raised
    recently_funded_days INTEGER,            -- Funded within X days

    -- Technology requirements
    tech_must_have JSONB DEFAULT '[]',       -- Must use these technologies
    tech_nice_to_have JSONB DEFAULT '[]',    -- Bonus points for these
    tech_avoid JSONB DEFAULT '[]',           -- Red flags

    -- Geography
    countries JSONB DEFAULT '[]',            -- ["US", "UK", "CA"]
    excluded_countries JSONB DEFAULT '[]',
    regions JSONB DEFAULT '[]',              -- ["North America", "Europe"]

    -- Decision maker targeting
    target_titles JSONB DEFAULT '[]',        -- ["CTO", "VP Engineering"]
    target_departments JSONB DEFAULT '[]',   -- ["Engineering", "Product"]
    seniority_levels JSONB DEFAULT '[]',     -- ["C-Level", "VP", "Director"]

    -- Signal preferences
    require_recent_funding BOOLEAN DEFAULT false,
    require_hiring_signals BOOLEAN DEFAULT false,
    require_tech_change BOOLEAN DEFAULT false,

    -- Scoring weights (0-100, must sum to 100)
    weight_intent INTEGER DEFAULT 40,
    weight_fit INTEGER DEFAULT 35,
    weight_accessibility INTEGER DEFAULT 25,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_icp_profiles_user ON icp_profiles(user_id);
CREATE INDEX idx_icp_profiles_default ON icp_profiles(user_id, is_default) WHERE is_default = true;

-- Ensure only one default per user
CREATE UNIQUE INDEX idx_icp_profiles_one_default ON icp_profiles(user_id) WHERE is_default = true;

-- =============================================================================
-- ENRICHMENT CACHE (avoid redundant API calls)
-- =============================================================================

CREATE TABLE enrichment_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Entity identification
    entity_type VARCHAR(50) NOT NULL,        -- company, person, email
    entity_key VARCHAR(500) NOT NULL,        -- domain, email, linkedin_url

    -- Source tracking
    source VARCHAR(50) NOT NULL,             -- apollo, clearbit, hunter, crunchbase

    -- Cached data
    data JSONB NOT NULL,
    data_hash VARCHAR(64),                   -- SHA256 for change detection

    -- Timing
    fetched_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,

    -- Stats
    hit_count INTEGER DEFAULT 0,
    last_hit_at TIMESTAMPTZ,

    UNIQUE(entity_type, entity_key, source)
);

CREATE INDEX idx_enrichment_cache_lookup ON enrichment_cache(entity_type, entity_key);
CREATE INDEX idx_enrichment_cache_expiry ON enrichment_cache(expires_at) WHERE expires_at < NOW() + INTERVAL '1 day';

-- =============================================================================
-- LEAD SCORES (with history)
-- =============================================================================

CREATE TABLE lead_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lead_id UUID NOT NULL REFERENCES leads(id) ON DELETE CASCADE,
    icp_id UUID REFERENCES icp_profiles(id) ON DELETE SET NULL,

    -- Individual scores (0-100)
    intent_score INTEGER NOT NULL DEFAULT 0,
    fit_score INTEGER NOT NULL DEFAULT 0,
    accessibility_score INTEGER NOT NULL DEFAULT 0,

    -- Weighted total (0-100)
    total_score INTEGER NOT NULL DEFAULT 0,

    -- Tier assignment
    tier VARCHAR(20) NOT NULL DEFAULT 'cold',  -- hot, warm, nurture, cold

    -- Detailed breakdown
    score_breakdown JSONB DEFAULT '{}',
    /*
    {
      "intent": {
        "recentFunding": { "points": 30, "reason": "Series B $20M (14 days ago)" },
        "hiringSignals": { "points": 25, "reason": "5 engineering roles open" }
      },
      "fit": {
        "industryMatch": { "points": 25, "reason": "SaaS - exact match" },
        "sizeMatch": { "points": 20, "reason": "150 employees - in range" }
      },
      "accessibility": {
        "emailFound": { "points": 30, "reason": "Verified work email" },
        "linkedinFound": { "points": 25, "reason": "Active LinkedIn profile" }
      }
    }
    */

    -- Signals that contributed
    active_signals JSONB DEFAULT '[]',

    -- Timing
    calculated_at TIMESTAMPTZ DEFAULT NOW(),
    previous_score INTEGER,                   -- For tracking changes
    score_change INTEGER                      -- Difference from previous
);

CREATE INDEX idx_lead_scores_lead ON lead_scores(lead_id, calculated_at DESC);
CREATE INDEX idx_lead_scores_tier ON lead_scores(tier, total_score DESC);
CREATE INDEX idx_lead_scores_recent ON lead_scores(calculated_at DESC);

-- Current score view (latest score per lead)
CREATE VIEW lead_current_scores AS
SELECT DISTINCT ON (lead_id) *
FROM lead_scores
ORDER BY lead_id, calculated_at DESC;

-- =============================================================================
-- SIGNAL EVENTS (timing intelligence)
-- =============================================================================

CREATE TABLE signal_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lead_id UUID REFERENCES leads(id) ON DELETE CASCADE,
    company_domain VARCHAR(255),              -- For company-level signals

    -- Signal classification
    signal_type VARCHAR(50) NOT NULL,
    /*
    Signal types:
    - funding_round      : New investment
    - executive_hire     : New C-level/VP
    - job_posting        : Relevant role posted
    - tech_adoption      : New technology detected
    - news_mention       : Press coverage
    - growth_indicator   : Revenue/employee growth
    - contract_ending    : Known renewal period
    - website_change     : Significant site updates
    */

    signal_category VARCHAR(20) NOT NULL,     -- intent, fit, engagement

    -- Signal details
    signal_data JSONB NOT NULL,
    /*
    Example for funding_round:
    {
      "round": "Series B",
      "amount": 20000000,
      "currency": "USD",
      "investors": ["Sequoia", "a16z"],
      "valuation": 100000000,
      "source_url": "https://techcrunch.com/..."
    }
    */

    -- Impact
    score_impact INTEGER DEFAULT 0,           -- Points added to score
    confidence FLOAT DEFAULT 1.0,             -- 0-1 confidence in signal

    -- Source
    source VARCHAR(100),                      -- Where we detected this
    source_url TEXT,

    -- Timing
    signal_date TIMESTAMPTZ,                  -- When the event happened
    detected_at TIMESTAMPTZ DEFAULT NOW(),    -- When we found it
    expires_at TIMESTAMPTZ,                   -- When signal becomes stale

    -- Processing
    is_processed BOOLEAN DEFAULT false,
    processed_at TIMESTAMPTZ
);

CREATE INDEX idx_signal_events_lead ON signal_events(lead_id, detected_at DESC);
CREATE INDEX idx_signal_events_domain ON signal_events(company_domain) WHERE company_domain IS NOT NULL;
CREATE INDEX idx_signal_events_type ON signal_events(signal_type, detected_at DESC);
CREATE INDEX idx_signal_events_unprocessed ON signal_events(is_processed) WHERE is_processed = false;

-- =============================================================================
-- ENRICHMENT JOBS (async processing queue)
-- =============================================================================

CREATE TABLE enrichment_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Job type
    job_type VARCHAR(50) NOT NULL,
    /*
    Job types:
    - enrich_lead        : Full lead enrichment
    - enrich_company     : Company-only enrichment
    - find_contacts      : Find decision makers
    - verify_email       : Email verification
    - detect_signals     : Check for new signals
    - score_lead         : Recalculate lead score
    - discover_leads     : Find new leads matching ICP
    */

    -- Target
    lead_id UUID REFERENCES leads(id) ON DELETE CASCADE,
    company_domain VARCHAR(255),
    icp_id UUID REFERENCES icp_profiles(id) ON DELETE SET NULL,

    -- Job configuration
    config JSONB DEFAULT '{}',
    /*
    Example for discover_leads:
    {
      "limit": 50,
      "sources": ["apollo", "clearbit"],
      "minScore": 60
    }
    */

    -- Status
    status VARCHAR(20) DEFAULT 'pending',     -- pending, running, completed, failed, cancelled
    priority INTEGER DEFAULT 0,               -- Higher = more urgent

    -- Results
    result JSONB,
    error_message TEXT,
    credits_used INTEGER DEFAULT 0,           -- API credits consumed

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

CREATE INDEX idx_enrichment_jobs_status ON enrichment_jobs(status, priority DESC, scheduled_at)
    WHERE status IN ('pending', 'running');
CREATE INDEX idx_enrichment_jobs_user ON enrichment_jobs(user_id, created_at DESC);
CREATE INDEX idx_enrichment_jobs_lead ON enrichment_jobs(lead_id) WHERE lead_id IS NOT NULL;

-- =============================================================================
-- API CREDENTIALS (user's external service keys)
-- =============================================================================

CREATE TABLE enrichment_credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    service VARCHAR(50) NOT NULL,             -- apollo, clearbit, hunter, crunchbase
    api_key_encrypted BYTEA NOT NULL,         -- AES-256 encrypted
    api_key_hint VARCHAR(10),                 -- Last 4 chars for display

    -- Status
    is_valid BOOLEAN DEFAULT true,
    last_validated_at TIMESTAMPTZ,
    error_message TEXT,

    -- Usage tracking
    credits_remaining INTEGER,
    credits_limit INTEGER,
    reset_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(user_id, service)
);

CREATE INDEX idx_enrichment_credentials_user ON enrichment_credentials(user_id);

-- =============================================================================
-- DISCOVERED LEADS (staging before adding to main leads)
-- =============================================================================

CREATE TABLE discovered_leads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    icp_id UUID REFERENCES icp_profiles(id) ON DELETE SET NULL,

    -- Company info
    company_name VARCHAR(255) NOT NULL,
    company_domain VARCHAR(255),
    company_linkedin VARCHAR(255),

    -- Contact info (decision maker)
    contact_name VARCHAR(255),
    contact_title VARCHAR(255),
    contact_email VARCHAR(255),
    contact_linkedin VARCHAR(255),

    -- Enrichment data
    company_data JSONB DEFAULT '{}',
    contact_data JSONB DEFAULT '{}',

    -- Scoring
    preliminary_score INTEGER DEFAULT 0,
    score_breakdown JSONB DEFAULT '{}',

    -- Signals that triggered discovery
    discovery_signals JSONB DEFAULT '[]',

    -- Status
    status VARCHAR(20) DEFAULT 'new',         -- new, reviewed, accepted, rejected, duplicate
    rejection_reason TEXT,

    -- Source
    source VARCHAR(100),                      -- Which service found this
    source_id VARCHAR(255),                   -- ID in source system

    -- Timing
    discovered_at TIMESTAMPTZ DEFAULT NOW(),
    reviewed_at TIMESTAMPTZ,
    accepted_at TIMESTAMPTZ,

    -- Link to actual lead if accepted
    converted_lead_id UUID REFERENCES leads(id) ON DELETE SET NULL
);

CREATE INDEX idx_discovered_leads_user_status ON discovered_leads(user_id, status, preliminary_score DESC);
CREATE INDEX idx_discovered_leads_domain ON discovered_leads(company_domain);
CREATE INDEX idx_discovered_leads_new ON discovered_leads(user_id, discovered_at DESC) WHERE status = 'new';

-- =============================================================================
-- TRIGGERS
-- =============================================================================

-- Auto-update updated_at for icp_profiles
CREATE TRIGGER icp_profiles_updated_at BEFORE UPDATE ON icp_profiles
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- Auto-update updated_at for enrichment_credentials
CREATE TRIGGER enrichment_credentials_updated_at BEFORE UPDATE ON enrichment_credentials
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- =============================================================================
-- FUNCTIONS
-- =============================================================================

-- Calculate lead tier based on score
CREATE OR REPLACE FUNCTION calculate_lead_tier(score INTEGER)
RETURNS VARCHAR(20) AS $$
BEGIN
    IF score >= 80 THEN
        RETURN 'hot';
    ELSIF score >= 60 THEN
        RETURN 'warm';
    ELSIF score >= 40 THEN
        RETURN 'nurture';
    ELSE
        RETURN 'cold';
    END IF;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Clean up expired cache entries (run as scheduled job)
-- SELECT cleanup_enrichment_cache();
CREATE OR REPLACE FUNCTION cleanup_enrichment_cache()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM enrichment_cache WHERE expires_at < NOW();
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;
