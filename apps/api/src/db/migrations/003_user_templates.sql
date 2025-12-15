-- Hekax Database Schema
-- Migration 003: User Email Templates & Auto-Cleanup

-- =============================================================================
-- USER EMAIL TEMPLATES (Custom user-created templates)
-- =============================================================================

CREATE TABLE user_email_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    name VARCHAR(100) NOT NULL,
    subject VARCHAR(255) NOT NULL,
    body_html TEXT NOT NULL,
    body_text TEXT,
    category VARCHAR(50),              -- cold_outreach, follow_up, cv_submission, etc.
    variables JSONB DEFAULT '[]',      -- Auto-extracted from template: ["firstName", "companyName"]

    is_active BOOLEAN DEFAULT true,
    use_count INTEGER DEFAULT 0,       -- Track template usage

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_user_templates_user ON user_email_templates(user_id);
CREATE INDEX idx_user_templates_category ON user_email_templates(user_id, category);

-- Trigger for updated_at
CREATE TRIGGER user_email_templates_updated_at BEFORE UPDATE ON user_email_templates
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- =============================================================================
-- USER SKILLS & LEARNING PROGRESS
-- =============================================================================

CREATE TABLE user_skills (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    skill_name VARCHAR(100) NOT NULL,
    skill_category VARCHAR(50),        -- technical, soft_skills, domain_knowledge
    proficiency_level VARCHAR(20),     -- beginner, intermediate, advanced, expert

    -- Learning tracking
    learning_started_at TIMESTAMPTZ,
    last_practiced_at TIMESTAMPTZ,
    hours_invested FLOAT DEFAULT 0,

    -- Source tracking
    source VARCHAR(50),                -- manual, cv_analysis, ai_detected
    confidence_score FLOAT,            -- 0-1 for AI-detected skills

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, skill_name)
);

CREATE INDEX idx_user_skills_user ON user_skills(user_id);
CREATE INDEX idx_user_skills_category ON user_skills(user_id, skill_category);

-- Trigger for updated_at
CREATE TRIGGER user_skills_updated_at BEFORE UPDATE ON user_skills
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- =============================================================================
-- LEARNING RESOURCES (Free courses, videos, articles)
-- =============================================================================

CREATE TABLE learning_resources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    skill_name VARCHAR(100) NOT NULL,

    title VARCHAR(255) NOT NULL,
    resource_type VARCHAR(20) NOT NULL,  -- video, course, article, book
    platform VARCHAR(50) NOT NULL,        -- youtube, edx, coursera, freecodecamp, khan
    url TEXT NOT NULL,

    is_free BOOLEAN DEFAULT true,
    duration VARCHAR(50),                  -- "2 hours", "4 weeks", etc.

    -- Progress tracking
    status VARCHAR(20) DEFAULT 'not_started',  -- not_started, in_progress, completed
    progress_percent INTEGER DEFAULT 0,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,

    -- AI recommendation metadata
    ai_recommended BOOLEAN DEFAULT false,
    recommendation_reason TEXT,
    priority VARCHAR(10),                  -- high, medium, low

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_learning_resources_user ON learning_resources(user_id);
CREATE INDEX idx_learning_resources_skill ON learning_resources(user_id, skill_name);
CREATE INDEX idx_learning_resources_status ON learning_resources(user_id, status);

-- Trigger for updated_at
CREATE TRIGGER learning_resources_updated_at BEFORE UPDATE ON learning_resources
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- =============================================================================
-- USER CVS (Stored and analyzed CVs)
-- =============================================================================

CREATE TABLE user_cvs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    name VARCHAR(100) NOT NULL,           -- "Master CV", "Tech CV", etc.
    original_filename VARCHAR(255),
    file_type VARCHAR(20),                -- pdf, docx
    r2_key VARCHAR(500),                  -- Storage key

    -- Extracted content
    content_text TEXT,                    -- Plain text extraction
    parsed_data JSONB,                    -- Structured: {name, email, experience[], education[], skills[]}

    -- AI Analysis
    ai_analysis JSONB,                    -- {strengths[], weaknesses[], suggestions[], overallScore}
    analyzed_at TIMESTAMPTZ,

    is_default BOOLEAN DEFAULT false,     -- Default CV for quick sends

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_user_cvs_user ON user_cvs(user_id);
CREATE UNIQUE INDEX idx_user_cvs_default ON user_cvs(user_id) WHERE is_default = true;

-- Trigger for updated_at
CREATE TRIGGER user_cvs_updated_at BEFORE UPDATE ON user_cvs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- =============================================================================
-- AUTO-CLEANUP FOR OLD UNREPLIED EMAILS (2 years)
-- =============================================================================

-- Function to clean up old unreplied emails
CREATE OR REPLACE FUNCTION cleanup_old_unreplied_emails()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM email_log
    WHERE sent_at < NOW() - INTERVAL '2 years'
    AND status NOT IN ('replied', 'clicked');

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Add replied_at column to email_log if not exists
ALTER TABLE email_log ADD COLUMN IF NOT EXISTS replied_at TIMESTAMPTZ;

-- Index for cleanup queries
CREATE INDEX IF NOT EXISTS idx_email_log_cleanup ON email_log(sent_at, status)
WHERE status NOT IN ('replied', 'clicked');

-- =============================================================================
-- EMAIL SCHEDULING
-- =============================================================================

CREATE TABLE scheduled_emails (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    lead_id UUID REFERENCES leads(id) ON DELETE CASCADE,

    -- Email content
    to_email VARCHAR(255) NOT NULL,
    subject VARCHAR(255) NOT NULL,
    body_html TEXT NOT NULL,
    body_text TEXT,

    -- Scheduling
    scheduled_for TIMESTAMPTZ NOT NULL,
    timezone VARCHAR(50) DEFAULT 'UTC',

    -- Status
    status VARCHAR(20) DEFAULT 'pending',  -- pending, sent, cancelled, failed
    sent_at TIMESTAMPTZ,
    email_log_id UUID REFERENCES email_log(id),
    error_message TEXT,

    -- Metadata
    purpose VARCHAR(50),
    template_id UUID,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_scheduled_emails_user ON scheduled_emails(user_id);
CREATE INDEX idx_scheduled_emails_pending ON scheduled_emails(scheduled_for)
WHERE status = 'pending';

-- =============================================================================
-- EMAIL SEQUENCES (Automated follow-up chains)
-- =============================================================================

CREATE TABLE email_sequences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    name VARCHAR(100) NOT NULL,
    description TEXT,

    is_active BOOLEAN DEFAULT true,

    -- Sequence settings
    stop_on_reply BOOLEAN DEFAULT true,
    stop_on_click BOOLEAN DEFAULT false,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_email_sequences_user ON email_sequences(user_id);

-- Sequence steps
CREATE TABLE email_sequence_steps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sequence_id UUID NOT NULL REFERENCES email_sequences(id) ON DELETE CASCADE,

    step_number INTEGER NOT NULL,
    delay_days INTEGER NOT NULL DEFAULT 3,  -- Days after previous step

    subject VARCHAR(255) NOT NULL,
    body_html TEXT NOT NULL,
    body_text TEXT,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(sequence_id, step_number)
);

CREATE INDEX idx_sequence_steps_sequence ON email_sequence_steps(sequence_id);

-- Sequence enrollments (leads in a sequence)
CREATE TABLE email_sequence_enrollments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sequence_id UUID NOT NULL REFERENCES email_sequences(id) ON DELETE CASCADE,
    lead_id UUID NOT NULL REFERENCES leads(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    current_step INTEGER DEFAULT 1,
    status VARCHAR(20) DEFAULT 'active',   -- active, completed, stopped, paused

    next_email_at TIMESTAMPTZ,

    enrolled_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    stopped_at TIMESTAMPTZ,
    stop_reason VARCHAR(50),               -- replied, clicked, manual, bounced

    UNIQUE(sequence_id, lead_id)
);

CREATE INDEX idx_enrollments_sequence ON email_sequence_enrollments(sequence_id);
CREATE INDEX idx_enrollments_lead ON email_sequence_enrollments(lead_id);
CREATE INDEX idx_enrollments_next ON email_sequence_enrollments(next_email_at)
WHERE status = 'active';
