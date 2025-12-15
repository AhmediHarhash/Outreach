-- Hekax Database Schema
-- Migration 002: Email System (AWS SES)

-- =============================================================================
-- EMAIL TEMPLATES
-- =============================================================================

CREATE TABLE email_templates (
    id VARCHAR(50) PRIMARY KEY,           -- welcome, password_reset, trial_ending, etc.
    name VARCHAR(100) NOT NULL,
    subject VARCHAR(255) NOT NULL,
    body_html TEXT NOT NULL,
    body_text TEXT,                        -- Plain text fallback
    variables JSONB DEFAULT '[]',          -- ["user_name", "reset_link", etc.]
    locale VARCHAR(10) DEFAULT 'en-US',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Default email templates
INSERT INTO email_templates (id, name, subject, body_html, body_text, variables) VALUES
('welcome', 'Welcome Email', 'Welcome to Hekax - Let''s Close Some Deals!',
'<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: -apple-system, BlinkMacSystemFont, ''Segoe UI'', Roboto, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
  <div style="text-align: center; margin-bottom: 30px;">
    <div style="width: 50px; height: 50px; background: #6366f1; border-radius: 12px; display: inline-flex; align-items: center; justify-content: center;">
      <span style="color: white; font-weight: bold; font-size: 24px;">H</span>
    </div>
  </div>
  <h1 style="color: #1f2937; margin-bottom: 20px;">Welcome to Hekax, {{user_name}}!</h1>
  <p style="color: #4b5563; line-height: 1.6;">You''re now equipped with AI superpowers for your sales calls. Here''s what you can do:</p>
  <ul style="color: #4b5563; line-height: 1.8;">
    <li><strong>Voice Copilot</strong> - Real-time AI hints during calls</li>
    <li><strong>Lead Hunter</strong> - Find and research B2B prospects</li>
    <li><strong>Call Analytics</strong> - AI-powered post-call insights</li>
  </ul>
  <div style="text-align: center; margin: 30px 0;">
    <a href="{{dashboard_url}}" style="background: #6366f1; color: white; padding: 12px 30px; border-radius: 8px; text-decoration: none; font-weight: 600;">Go to Dashboard</a>
  </div>
  <p style="color: #9ca3af; font-size: 14px;">Ready to close more deals?<br>The Hekax Team</p>
</body>
</html>',
'Welcome to Hekax, {{user_name}}!

You''re now equipped with AI superpowers for your sales calls.

Go to your dashboard: {{dashboard_url}}

The Hekax Team',
'["user_name", "dashboard_url"]'),

('password_reset', 'Password Reset', 'Reset Your Hekax Password',
'<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: -apple-system, BlinkMacSystemFont, ''Segoe UI'', Roboto, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
  <div style="text-align: center; margin-bottom: 30px;">
    <div style="width: 50px; height: 50px; background: #6366f1; border-radius: 12px; display: inline-flex; align-items: center; justify-content: center;">
      <span style="color: white; font-weight: bold; font-size: 24px;">H</span>
    </div>
  </div>
  <h1 style="color: #1f2937; margin-bottom: 20px;">Reset Your Password</h1>
  <p style="color: #4b5563; line-height: 1.6;">We received a request to reset your password. Click the button below to create a new one:</p>
  <div style="text-align: center; margin: 30px 0;">
    <a href="{{reset_url}}" style="background: #6366f1; color: white; padding: 12px 30px; border-radius: 8px; text-decoration: none; font-weight: 600;">Reset Password</a>
  </div>
  <p style="color: #9ca3af; font-size: 14px;">This link expires in 1 hour. If you didn''t request this, you can safely ignore this email.</p>
</body>
</html>',
'Reset Your Hekax Password

Click here to reset: {{reset_url}}

This link expires in 1 hour.',
'["reset_url"]'),

('email_verification', 'Email Verification', 'Verify Your Hekax Email',
'<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: -apple-system, BlinkMacSystemFont, ''Segoe UI'', Roboto, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
  <div style="text-align: center; margin-bottom: 30px;">
    <div style="width: 50px; height: 50px; background: #6366f1; border-radius: 12px; display: inline-flex; align-items: center; justify-content: center;">
      <span style="color: white; font-weight: bold; font-size: 24px;">H</span>
    </div>
  </div>
  <h1 style="color: #1f2937; margin-bottom: 20px;">Verify Your Email</h1>
  <p style="color: #4b5563; line-height: 1.6;">Please verify your email address to complete your Hekax setup:</p>
  <div style="text-align: center; margin: 30px 0;">
    <a href="{{verify_url}}" style="background: #6366f1; color: white; padding: 12px 30px; border-radius: 8px; text-decoration: none; font-weight: 600;">Verify Email</a>
  </div>
</body>
</html>',
'Verify your Hekax email: {{verify_url}}',
'["verify_url"]'),

('trial_ending', 'Trial Ending', 'Your Hekax Trial Ends in {{days_left}} Days',
'<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: -apple-system, BlinkMacSystemFont, ''Segoe UI'', Roboto, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
  <div style="text-align: center; margin-bottom: 30px;">
    <div style="width: 50px; height: 50px; background: #6366f1; border-radius: 12px; display: inline-flex; align-items: center; justify-content: center;">
      <span style="color: white; font-weight: bold; font-size: 24px;">H</span>
    </div>
  </div>
  <h1 style="color: #1f2937; margin-bottom: 20px;">Your Trial Ends Soon</h1>
  <p style="color: #4b5563; line-height: 1.6;">Hi {{user_name}}, your Hekax trial ends in <strong>{{days_left}} days</strong>.</p>
  <p style="color: #4b5563; line-height: 1.6;">Upgrade now to keep your AI sales superpowers:</p>
  <div style="text-align: center; margin: 30px 0;">
    <a href="{{upgrade_url}}" style="background: #6366f1; color: white; padding: 12px 30px; border-radius: 8px; text-decoration: none; font-weight: 600;">Upgrade to Pro</a>
  </div>
</body>
</html>',
'Your Hekax trial ends in {{days_left}} days. Upgrade: {{upgrade_url}}',
'["user_name", "days_left", "upgrade_url"]');

-- =============================================================================
-- EMAIL LOG (Track all sent emails)
-- =============================================================================

CREATE TABLE email_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,

    -- Email details
    to_email VARCHAR(255) NOT NULL,
    from_email VARCHAR(255) NOT NULL,
    subject VARCHAR(255) NOT NULL,
    template_id VARCHAR(50) REFERENCES email_templates(id),

    -- SES tracking
    ses_message_id VARCHAR(255),
    status VARCHAR(50) DEFAULT 'sent',    -- sent, delivered, bounced, complained, failed

    -- Bounce/complaint handling
    bounce_type VARCHAR(50),               -- Permanent, Transient
    bounce_subtype VARCHAR(50),            -- General, NoEmail, Suppressed, etc.
    complaint_type VARCHAR(50),            -- abuse, auth-failure, fraud, etc.

    -- Metadata
    metadata JSONB DEFAULT '{}',           -- Template variables used
    error_message TEXT,

    sent_at TIMESTAMPTZ DEFAULT NOW(),
    delivered_at TIMESTAMPTZ,
    opened_at TIMESTAMPTZ,
    clicked_at TIMESTAMPTZ
);

CREATE INDEX idx_email_log_user ON email_log(user_id, sent_at DESC);
CREATE INDEX idx_email_log_status ON email_log(status) WHERE status != 'delivered';
CREATE INDEX idx_email_log_ses ON email_log(ses_message_id) WHERE ses_message_id IS NOT NULL;

-- =============================================================================
-- EMAIL SUPPRESSION LIST (Bounced/Complained emails)
-- =============================================================================

CREATE TABLE email_suppression (
    email VARCHAR(255) PRIMARY KEY,
    reason VARCHAR(50) NOT NULL,          -- bounce, complaint, unsubscribe
    details JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- PASSWORD RESET TOKENS
-- =============================================================================

CREATE TABLE password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,      -- SHA256 hash
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_password_reset_user ON password_reset_tokens(user_id);
CREATE INDEX idx_password_reset_expires ON password_reset_tokens(expires_at) WHERE used_at IS NULL;

-- =============================================================================
-- EMAIL VERIFICATION TOKENS
-- =============================================================================

CREATE TABLE email_verification_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_email_verify_user ON email_verification_tokens(user_id);
