-- =============================================================================
-- NEON DATABASE SETUP FOR HEKAX
-- Run this in Neon's SQL Editor: https://console.neon.tech
-- =============================================================================

-- Step 1: Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "vector";

-- Verify extensions are enabled
SELECT extname, extversion FROM pg_extension WHERE extname IN ('uuid-ossp', 'pgcrypto', 'vector');
