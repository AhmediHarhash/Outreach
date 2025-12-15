-- Outreach Database Schema
-- Migration 004: CV Processing System

-- =============================================================================
-- USER PROFILE (For CV Generation)
-- =============================================================================

CREATE TABLE user_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID UNIQUE NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Personal info
    full_name VARCHAR(255),
    email VARCHAR(255),
    phone VARCHAR(50),
    linkedin VARCHAR(255),
    github VARCHAR(255),
    portfolio_url VARCHAR(500),
    location VARCHAR(255),

    -- Professional summary
    headline VARCHAR(255),              -- "Senior Full-Stack Developer"
    summary TEXT,                       -- Professional summary paragraph
    years_experience INTEGER,

    -- Skills (structured)
    skills JSONB DEFAULT '[]',          -- [{name, level, category, yearsUsed}]
    languages JSONB DEFAULT '[]',       -- [{language, proficiency}]
    certifications JSONB DEFAULT '[]',  -- [{name, issuer, date, url}]

    -- Work experience
    experience JSONB DEFAULT '[]',      -- [{company, title, startDate, endDate, current, description, achievements}]

    -- Education
    education JSONB DEFAULT '[]',       -- [{institution, degree, field, startDate, endDate, gpa, achievements}]

    -- Projects (optional)
    projects JSONB DEFAULT '[]',        -- [{name, description, url, technologies, highlights}]

    -- Publications/Awards (optional)
    publications JSONB DEFAULT '[]',    -- [{title, publication, date, url}]
    awards JSONB DEFAULT '[]',          -- [{name, issuer, date, description}]

    -- Preferences
    desired_roles JSONB DEFAULT '[]',   -- ["Software Engineer", "Full-Stack Developer"]
    desired_industries JSONB DEFAULT '[]',
    salary_expectation_min INTEGER,
    salary_expectation_max INTEGER,
    salary_currency VARCHAR(3) DEFAULT 'USD',
    remote_preference VARCHAR(50),      -- remote, hybrid, onsite
    willing_to_relocate BOOLEAN DEFAULT false,

    -- Last uploaded CV reference
    master_cv_id UUID,                  -- Reference to uploaded CV document

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_user_profiles_user ON user_profiles(user_id);

-- =============================================================================
-- CV DOCUMENTS (Stored CVs - both uploaded and generated)
-- =============================================================================

CREATE TABLE cv_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Document info
    name VARCHAR(255) NOT NULL,         -- "Master CV", "CV for Acme Corp", etc.
    version INTEGER DEFAULT 1,

    -- File storage
    original_filename VARCHAR(255),
    file_type VARCHAR(10),              -- pdf, docx
    file_size_bytes BIGINT,
    r2_key VARCHAR(500),                -- Cloud storage key

    -- Extracted content
    raw_text TEXT,                      -- Full text extraction
    parsed_data JSONB,                  -- Structured extraction {sections, skills, experience, etc.}

    -- Document type
    doc_type VARCHAR(50) DEFAULT 'master',  -- master, tailored, generated

    -- For tailored CVs
    lead_id UUID REFERENCES leads(id) ON DELETE SET NULL,
    job_posting_url VARCHAR(500),
    job_posting_text TEXT,

    -- AI Analysis
    ats_score INTEGER,                  -- 0-100 ATS optimization score
    ats_issues JSONB DEFAULT '[]',      -- [{issue, severity, suggestion}]
    improvement_suggestions JSONB DEFAULT '[]', -- [{section, suggestion, priority}]

    -- Generation metadata
    template_id VARCHAR(50),
    generation_prompt TEXT,

    -- Status
    status VARCHAR(50) DEFAULT 'processing', -- processing, ready, failed
    processing_error TEXT,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_cv_documents_user ON cv_documents(user_id, created_at DESC);
CREATE INDEX idx_cv_documents_lead ON cv_documents(lead_id) WHERE lead_id IS NOT NULL;
CREATE INDEX idx_cv_documents_type ON cv_documents(user_id, doc_type);

-- =============================================================================
-- CV TEMPLATES
-- =============================================================================

CREATE TABLE cv_templates (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    description TEXT,

    -- Template content
    template_html TEXT NOT NULL,        -- HTML/Handlebars template
    template_css TEXT,                  -- Custom CSS

    -- Metadata
    category VARCHAR(50),               -- professional, creative, academic, technical
    industries JSONB DEFAULT '[]',      -- Recommended industries

    -- Preview
    preview_image_url VARCHAR(500),

    is_active BOOLEAN DEFAULT true,
    is_premium BOOLEAN DEFAULT false,
    sort_order INTEGER DEFAULT 0,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert default CV templates
INSERT INTO cv_templates (id, name, description, category, template_html, is_premium, sort_order) VALUES
('classic_professional', 'Classic Professional', 'Clean, traditional format suitable for most industries', 'professional',
'<!DOCTYPE html>
<html>
<head>
  <style>
    body { font-family: Georgia, serif; max-width: 800px; margin: 0 auto; padding: 40px; color: #333; }
    h1 { color: #1a365d; margin-bottom: 5px; }
    .contact { color: #666; font-size: 14px; margin-bottom: 20px; }
    .section { margin-bottom: 25px; }
    .section-title { color: #1a365d; border-bottom: 2px solid #1a365d; padding-bottom: 5px; margin-bottom: 15px; font-size: 16px; text-transform: uppercase; }
    .job { margin-bottom: 15px; }
    .job-header { display: flex; justify-content: space-between; margin-bottom: 5px; }
    .job-title { font-weight: bold; }
    .job-company { color: #666; }
    .job-date { color: #888; font-size: 14px; }
    ul { margin: 5px 0; padding-left: 20px; }
    li { margin-bottom: 3px; }
    .skills-grid { display: flex; flex-wrap: wrap; gap: 10px; }
    .skill-tag { background: #f3f4f6; padding: 4px 12px; border-radius: 4px; font-size: 14px; }
  </style>
</head>
<body>
  <h1>{{fullName}}</h1>
  <div class="contact">{{email}} | {{phone}} | {{location}}</div>

  {{#if summary}}
  <div class="section">
    <div class="section-title">Professional Summary</div>
    <p>{{summary}}</p>
  </div>
  {{/if}}

  {{#if experience.length}}
  <div class="section">
    <div class="section-title">Experience</div>
    {{#each experience}}
    <div class="job">
      <div class="job-header">
        <span class="job-title">{{title}}</span>
        <span class="job-date">{{startDate}} - {{#if current}}Present{{else}}{{endDate}}{{/if}}</span>
      </div>
      <div class="job-company">{{company}}</div>
      <ul>
        {{#each achievements}}
        <li>{{this}}</li>
        {{/each}}
      </ul>
    </div>
    {{/each}}
  </div>
  {{/if}}

  {{#if education.length}}
  <div class="section">
    <div class="section-title">Education</div>
    {{#each education}}
    <div class="job">
      <div class="job-header">
        <span class="job-title">{{degree}} in {{field}}</span>
        <span class="job-date">{{endDate}}</span>
      </div>
      <div class="job-company">{{institution}}</div>
    </div>
    {{/each}}
  </div>
  {{/if}}

  {{#if skills.length}}
  <div class="section">
    <div class="section-title">Skills</div>
    <div class="skills-grid">
      {{#each skills}}
      <span class="skill-tag">{{name}}</span>
      {{/each}}
    </div>
  </div>
  {{/if}}
</body>
</html>', false, 1),

('modern_minimal', 'Modern Minimal', 'Contemporary design with clean lines and modern typography', 'professional',
'<!DOCTYPE html>
<html>
<head>
  <style>
    body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; max-width: 800px; margin: 0 auto; padding: 40px; color: #1f2937; }
    h1 { font-weight: 600; margin-bottom: 8px; }
    .contact { color: #6b7280; font-size: 14px; margin-bottom: 30px; display: flex; gap: 15px; flex-wrap: wrap; }
    .contact a { color: #3b82f6; text-decoration: none; }
    .section { margin-bottom: 30px; }
    .section-title { font-weight: 600; font-size: 12px; text-transform: uppercase; letter-spacing: 1px; color: #9ca3af; margin-bottom: 15px; }
    .job { margin-bottom: 20px; }
    .job-header { display: flex; justify-content: space-between; align-items: baseline; }
    .job-title { font-weight: 600; }
    .job-company { color: #6b7280; }
    .job-date { color: #9ca3af; font-size: 14px; }
    .job-description { margin-top: 8px; color: #4b5563; }
    ul { margin: 8px 0; padding-left: 20px; color: #4b5563; }
    li { margin-bottom: 4px; }
    .skills-list { display: flex; flex-wrap: wrap; gap: 8px; }
    .skill { background: #f3f4f6; padding: 6px 14px; border-radius: 20px; font-size: 13px; }
  </style>
</head>
<body>
  <h1>{{fullName}}</h1>
  <div class="contact">
    <span>{{email}}</span>
    <span>{{phone}}</span>
    <span>{{location}}</span>
    {{#if linkedin}}<a href="{{linkedin}}">LinkedIn</a>{{/if}}
    {{#if github}}<a href="{{github}}">GitHub</a>{{/if}}
  </div>

  {{#if summary}}
  <div class="section">
    <div class="section-title">About</div>
    <p style="color: #4b5563;">{{summary}}</p>
  </div>
  {{/if}}

  {{#if experience.length}}
  <div class="section">
    <div class="section-title">Experience</div>
    {{#each experience}}
    <div class="job">
      <div class="job-header">
        <div>
          <span class="job-title">{{title}}</span>
          <span class="job-company"> at {{company}}</span>
        </div>
        <span class="job-date">{{startDate}} — {{#if current}}Present{{else}}{{endDate}}{{/if}}</span>
      </div>
      <ul>
        {{#each achievements}}
        <li>{{this}}</li>
        {{/each}}
      </ul>
    </div>
    {{/each}}
  </div>
  {{/if}}

  {{#if skills.length}}
  <div class="section">
    <div class="section-title">Skills</div>
    <div class="skills-list">
      {{#each skills}}
      <span class="skill">{{name}}</span>
      {{/each}}
    </div>
  </div>
  {{/if}}

  {{#if education.length}}
  <div class="section">
    <div class="section-title">Education</div>
    {{#each education}}
    <div class="job">
      <div class="job-header">
        <div>
          <span class="job-title">{{degree}}</span>
          <span class="job-company"> — {{institution}}</span>
        </div>
        <span class="job-date">{{endDate}}</span>
      </div>
    </div>
    {{/each}}
  </div>
  {{/if}}
</body>
</html>', false, 2),

('tech_focused', 'Tech Professional', 'Optimized for software developers and tech professionals', 'technical',
'<!DOCTYPE html>
<html>
<head>
  <style>
    body { font-family: "JetBrains Mono", "Fira Code", monospace; max-width: 800px; margin: 0 auto; padding: 40px; color: #e2e8f0; background: #1e293b; }
    h1 { color: #38bdf8; margin-bottom: 5px; font-weight: 700; }
    .headline { color: #94a3b8; font-size: 14px; margin-bottom: 15px; }
    .contact { color: #64748b; font-size: 13px; margin-bottom: 25px; }
    .contact a { color: #38bdf8; }
    .section { margin-bottom: 25px; }
    .section-title { color: #38bdf8; font-size: 14px; font-weight: 600; margin-bottom: 12px; padding-bottom: 5px; border-bottom: 1px solid #334155; }
    .job { margin-bottom: 18px; }
    .job-header { margin-bottom: 5px; }
    .job-title { color: #f1f5f9; font-weight: 600; }
    .job-company { color: #94a3b8; }
    .job-date { color: #64748b; font-size: 12px; }
    .job-tech { color: #38bdf8; font-size: 12px; margin-top: 5px; }
    ul { margin: 8px 0; padding-left: 20px; color: #cbd5e1; }
    li { margin-bottom: 4px; font-size: 14px; }
    .skills-section { display: grid; grid-template-columns: repeat(2, 1fr); gap: 15px; }
    .skill-category { background: #334155; padding: 12px; border-radius: 8px; }
    .skill-category-title { color: #38bdf8; font-size: 12px; margin-bottom: 8px; }
    .skill-list { display: flex; flex-wrap: wrap; gap: 6px; }
    .skill { background: #475569; padding: 3px 10px; border-radius: 4px; font-size: 12px; color: #e2e8f0; }
  </style>
</head>
<body>
  <h1>{{fullName}}</h1>
  {{#if headline}}<div class="headline">{{headline}}</div>{{/if}}
  <div class="contact">
    {{email}} • {{phone}} • {{location}}
    {{#if github}} • <a href="{{github}}">GitHub</a>{{/if}}
    {{#if linkedin}} • <a href="{{linkedin}}">LinkedIn</a>{{/if}}
  </div>

  {{#if summary}}
  <div class="section">
    <div class="section-title">// About</div>
    <p style="color: #cbd5e1; font-size: 14px;">{{summary}}</p>
  </div>
  {{/if}}

  {{#if experience.length}}
  <div class="section">
    <div class="section-title">// Experience</div>
    {{#each experience}}
    <div class="job">
      <div class="job-header">
        <span class="job-title">{{title}}</span>
        <span class="job-company"> @ {{company}}</span>
      </div>
      <div class="job-date">{{startDate}} → {{#if current}}Present{{else}}{{endDate}}{{/if}}</div>
      {{#if technologies}}<div class="job-tech">Stack: {{technologies}}</div>{{/if}}
      <ul>
        {{#each achievements}}
        <li>{{this}}</li>
        {{/each}}
      </ul>
    </div>
    {{/each}}
  </div>
  {{/if}}

  {{#if skills.length}}
  <div class="section">
    <div class="section-title">// Skills</div>
    <div class="skills-section">
      <div class="skill-category">
        <div class="skill-category-title">Languages & Frameworks</div>
        <div class="skill-list">
          {{#each skills}}
          {{#if (eq category "language")}}<span class="skill">{{name}}</span>{{/if}}
          {{/each}}
        </div>
      </div>
      <div class="skill-category">
        <div class="skill-category-title">Tools & Technologies</div>
        <div class="skill-list">
          {{#each skills}}
          {{#if (eq category "tool")}}<span class="skill">{{name}}</span>{{/if}}
          {{/each}}
        </div>
      </div>
    </div>
  </div>
  {{/if}}
</body>
</html>', false, 3);

-- =============================================================================
-- CV GENERATION HISTORY
-- =============================================================================

CREATE TABLE cv_generations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    cv_document_id UUID REFERENCES cv_documents(id) ON DELETE SET NULL,

    -- Target
    lead_id UUID REFERENCES leads(id) ON DELETE SET NULL,
    job_title VARCHAR(255),
    job_description TEXT,
    company_name VARCHAR(255),

    -- AI Generation details
    model_used VARCHAR(100),
    prompt_tokens INTEGER,
    completion_tokens INTEGER,
    cost_usd DECIMAL(10,6),

    -- What was customized
    customizations JSONB,              -- {summary: "tailored", skills: "reordered", experience: "highlighted"}

    -- Feedback
    user_rating INTEGER,               -- 1-5 stars
    feedback_text TEXT,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_cv_generations_user ON cv_generations(user_id, created_at DESC);
CREATE INDEX idx_cv_generations_lead ON cv_generations(lead_id) WHERE lead_id IS NOT NULL;

-- =============================================================================
-- CV PARSING CACHE (For faster re-parsing)
-- =============================================================================

CREATE TABLE cv_parsing_cache (
    file_hash VARCHAR(64) PRIMARY KEY,  -- SHA256 of file
    parsed_data JSONB NOT NULL,
    parser_version VARCHAR(20),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Auto-cleanup old cache entries (optional - run as scheduled job)
-- DELETE FROM cv_parsing_cache WHERE created_at < NOW() - INTERVAL '30 days';

-- =============================================================================
-- TRIGGERS
-- =============================================================================

-- Auto-update updated_at for user_profiles
CREATE TRIGGER user_profiles_updated_at BEFORE UPDATE ON user_profiles
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- Auto-update updated_at for cv_documents
CREATE TRIGGER cv_documents_updated_at BEFORE UPDATE ON cv_documents
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
