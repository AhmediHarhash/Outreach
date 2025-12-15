/**
 * Lead Discovery Routes
 * Proxy to the Intelligence Worker for lead discovery and enrichment
 */

import { Router, Response } from 'express';
import { authMiddleware, AuthenticatedRequest } from '../middleware/auth.js';
import { config } from '../config.js';
import { query } from '../db.js';
import { validateUUID } from '../utils/validation.js';

const router = Router();

// All routes require authentication
router.use(authMiddleware);

// Intelligence worker URL
const INTELLIGENCE_URL = process.env.INTELLIGENCE_WORKER_URL || 'http://localhost:8000';

/**
 * POST /discovery/enrich-company
 * Enrich company data
 */
router.post('/enrich-company', async (req: AuthenticatedRequest, res: Response) => {
  try {
    const { domain, sources } = req.body;

    if (!domain) {
      return res.status(400).json({ error: 'Domain is required' });
    }

    // Get user's API credentials
    const credentials = await getUserCredentials(req.userId!);

    // Call intelligence worker
    const response = await fetch(`${INTELLIGENCE_URL}/api/v1/enrich/company`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        domain,
        sources,
        user_id: req.userId,
        ...credentials,
      }),
    });

    const result = await response.json();
    res.status(response.status).json(result);
  } catch (error) {
    console.error('Enrichment failed:', error);
    res.status(500).json({ error: 'Enrichment service unavailable' });
  }
});

/**
 * POST /discovery/find-contacts
 * Find contacts at a company
 */
router.post('/find-contacts', async (req: AuthenticatedRequest, res: Response) => {
  try {
    const { domain, titles, departments, limit = 5 } = req.body;

    if (!domain) {
      return res.status(400).json({ error: 'Domain is required' });
    }

    const credentials = await getUserCredentials(req.userId!);

    const response = await fetch(`${INTELLIGENCE_URL}/api/v1/enrich/contacts`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        domain,
        titles,
        departments,
        limit,
        user_id: req.userId,
        ...credentials,
      }),
    });

    const result = await response.json();
    res.status(response.status).json(result);
  } catch (error) {
    console.error('Contact finding failed:', error);
    res.status(500).json({ error: 'Contact finding service unavailable' });
  }
});

/**
 * POST /discovery/verify-email
 * Verify an email address
 */
router.post('/verify-email', async (req: AuthenticatedRequest, res: Response) => {
  try {
    const { email } = req.body;

    if (!email) {
      return res.status(400).json({ error: 'Email is required' });
    }

    const credentials = await getUserCredentials(req.userId!);

    const response = await fetch(`${INTELLIGENCE_URL}/api/v1/enrich/verify-email`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        email,
        user_id: req.userId,
        hunter_key: credentials.hunter_key,
      }),
    });

    const result = await response.json();
    res.status(response.status).json(result);
  } catch (error) {
    console.error('Email verification failed:', error);
    res.status(500).json({ error: 'Email verification service unavailable' });
  }
});

/**
 * POST /discovery/discover
 * Discover new leads matching ICP
 */
router.post('/discover', async (req: AuthenticatedRequest, res: Response) => {
  try {
    const { icp_id, limit = 25, min_score = 60, sources } = req.body;

    const credentials = await getUserCredentials(req.userId!);

    const response = await fetch(`${INTELLIGENCE_URL}/api/v1/discover`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        user_id: req.userId,
        icp_id,
        limit,
        min_score,
        sources,
        ...credentials,
      }),
    });

    const result = await response.json();
    res.status(response.status).json(result);
  } catch (error) {
    console.error('Discovery failed:', error);
    res.status(500).json({ error: 'Discovery service unavailable' });
  }
});

/**
 * GET /discovery/pending
 * Get discovered leads pending review
 */
router.get('/pending', async (req: AuthenticatedRequest, res: Response) => {
  try {
    const limit = parseInt(req.query.limit as string) || 50;
    const offset = parseInt(req.query.offset as string) || 0;

    const result = await query(
      `SELECT * FROM discovered_leads
       WHERE user_id = $1 AND status = 'new'
       ORDER BY preliminary_score DESC
       LIMIT $2 OFFSET $3`,
      [req.userId, limit, offset]
    );

    const countResult = await query(
      `SELECT COUNT(*) as count FROM discovered_leads
       WHERE user_id = $1 AND status = 'new'`,
      [req.userId]
    );

    res.json({
      leads: result.rows,
      count: result.rows.length,
      total: parseInt(countResult.rows[0].count),
    });
  } catch (error) {
    console.error('Failed to get pending discoveries:', error);
    res.status(500).json({ error: 'Failed to get pending discoveries' });
  }
});

/**
 * POST /discovery/:id/review
 * Review a discovered lead (accept/reject)
 */
router.post('/:id/review', async (req: AuthenticatedRequest, res: Response) => {
  try {
    const { id } = req.params;
    const { action, rejection_reason } = req.body;

    if (!validateUUID(id)) {
      return res.status(400).json({ error: 'Invalid lead ID' });
    }

    if (!['accept', 'reject', 'skip'].includes(action)) {
      return res.status(400).json({ error: 'Invalid action' });
    }

    // Get the discovered lead
    const leadResult = await query(
      `SELECT * FROM discovered_leads WHERE id = $1 AND user_id = $2`,
      [id, req.userId]
    );

    if (leadResult.rows.length === 0) {
      return res.status(404).json({ error: 'Discovered lead not found' });
    }

    const discovered = leadResult.rows[0];

    if (action === 'accept') {
      // Create actual lead
      const newLeadResult = await query(
        `INSERT INTO leads (
          user_id, company_name, company_domain, name, title, email,
          linkedin_url, company_data, status, source
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'new', 'discovery')
        RETURNING id`,
        [
          req.userId,
          discovered.company_name,
          discovered.company_domain,
          discovered.contact_name,
          discovered.contact_title,
          discovered.contact_email,
          discovered.contact_linkedin,
          discovered.company_data,
        ]
      );

      // Update discovered lead status
      await query(
        `UPDATE discovered_leads
         SET status = 'accepted', reviewed_at = NOW(), accepted_at = NOW(),
             converted_lead_id = $2
         WHERE id = $1`,
        [id, newLeadResult.rows[0].id]
      );

      res.json({
        success: true,
        action: 'accepted',
        lead_id: newLeadResult.rows[0].id,
      });
    } else if (action === 'reject') {
      await query(
        `UPDATE discovered_leads
         SET status = 'rejected', reviewed_at = NOW(), rejection_reason = $2
         WHERE id = $1`,
        [id, rejection_reason]
      );

      res.json({ success: true, action: 'rejected' });
    } else {
      await query(
        `UPDATE discovered_leads
         SET status = 'reviewed', reviewed_at = NOW()
         WHERE id = $1`,
        [id]
      );

      res.json({ success: true, action: 'skipped' });
    }
  } catch (error) {
    console.error('Failed to review lead:', error);
    res.status(500).json({ error: 'Failed to review lead' });
  }
});

/**
 * GET /discovery/scores
 * Get lead score distribution
 */
router.get('/scores', async (req: AuthenticatedRequest, res: Response) => {
  try {
    const result = await query(
      `SELECT tier, COUNT(*) as count, AVG(total_score) as avg_score
       FROM lead_current_scores lcs
       JOIN leads l ON lcs.lead_id = l.id
       WHERE l.user_id = $1
       GROUP BY tier
       ORDER BY avg_score DESC`,
      [req.userId]
    );

    const distribution = {
      hot: { count: 0, avg_score: 0 },
      warm: { count: 0, avg_score: 0 },
      nurture: { count: 0, avg_score: 0 },
      cold: { count: 0, avg_score: 0 },
    };

    for (const row of result.rows) {
      if (row.tier in distribution) {
        distribution[row.tier as keyof typeof distribution] = {
          count: parseInt(row.count),
          avg_score: Math.round(parseFloat(row.avg_score) * 10) / 10,
        };
      }
    }

    const total = Object.values(distribution).reduce((sum, d) => sum + d.count, 0);

    res.json({
      total_leads: total,
      distribution,
    });
  } catch (error) {
    console.error('Failed to get score distribution:', error);
    res.status(500).json({ error: 'Failed to get score distribution' });
  }
});

/**
 * GET /discovery/scored-leads
 * Get scored leads
 */
router.get('/scored-leads', async (req: AuthenticatedRequest, res: Response) => {
  try {
    const tier = req.query.tier as string;
    const minScore = parseInt(req.query.min_score as string) || 0;
    const limit = parseInt(req.query.limit as string) || 50;
    const offset = parseInt(req.query.offset as string) || 0;

    let whereClause = 'WHERE l.user_id = $1';
    const params: any[] = [req.userId];

    if (tier) {
      params.push(tier);
      whereClause += ` AND lcs.tier = $${params.length}`;
    }

    if (minScore > 0) {
      params.push(minScore);
      whereClause += ` AND lcs.total_score >= $${params.length}`;
    }

    params.push(limit, offset);

    const result = await query(
      `SELECT l.*, lcs.total_score, lcs.tier, lcs.intent_score,
              lcs.fit_score, lcs.accessibility_score, lcs.score_breakdown,
              lcs.calculated_at as scored_at
       FROM leads l
       JOIN lead_current_scores lcs ON l.id = lcs.lead_id
       ${whereClause}
       ORDER BY lcs.total_score DESC
       LIMIT $${params.length - 1} OFFSET $${params.length}`,
      params
    );

    res.json({
      leads: result.rows,
      count: result.rows.length,
    });
  } catch (error) {
    console.error('Failed to get scored leads:', error);
    res.status(500).json({ error: 'Failed to get scored leads' });
  }
});

/**
 * POST /discovery/credentials
 * Save user's enrichment API credentials
 */
router.post('/credentials', async (req: AuthenticatedRequest, res: Response) => {
  try {
    const { service, api_key } = req.body;

    const validServices = ['apollo', 'clearbit', 'hunter', 'crunchbase'];
    if (!validServices.includes(service)) {
      return res.status(400).json({ error: 'Invalid service' });
    }

    if (!api_key || api_key.trim().length === 0) {
      return res.status(400).json({ error: 'API key is required' });
    }

    // Store the key (in production, this should be encrypted)
    // For now, store as-is with a hint
    const keyHint = api_key.slice(-4);

    await query(
      `INSERT INTO enrichment_credentials (user_id, service, api_key_encrypted, api_key_hint, is_valid)
       VALUES ($1, $2, $3, $4, true)
       ON CONFLICT (user_id, service)
       DO UPDATE SET api_key_encrypted = $3, api_key_hint = $4, is_valid = true, updated_at = NOW()`,
      [req.userId, service, Buffer.from(api_key), keyHint]
    );

    res.json({
      success: true,
      service,
      hint: `****${keyHint}`,
    });
  } catch (error) {
    console.error('Failed to save credentials:', error);
    res.status(500).json({ error: 'Failed to save credentials' });
  }
});

/**
 * GET /discovery/credentials
 * Get user's configured credentials (hints only)
 */
router.get('/credentials', async (req: AuthenticatedRequest, res: Response) => {
  try {
    const result = await query(
      `SELECT service, api_key_hint, is_valid, last_validated_at, error_message,
              credits_remaining, credits_limit
       FROM enrichment_credentials
       WHERE user_id = $1`,
      [req.userId]
    );

    res.json({
      credentials: result.rows.map((row) => ({
        service: row.service,
        configured: true,
        hint: row.api_key_hint ? `****${row.api_key_hint}` : null,
        is_valid: row.is_valid,
        error: row.error_message,
        credits_remaining: row.credits_remaining,
        credits_limit: row.credits_limit,
      })),
    });
  } catch (error) {
    console.error('Failed to get credentials:', error);
    res.status(500).json({ error: 'Failed to get credentials' });
  }
});

/**
 * DELETE /discovery/credentials/:service
 * Remove a credential
 */
router.delete('/credentials/:service', async (req: AuthenticatedRequest, res: Response) => {
  try {
    const { service } = req.params;

    await query(
      `DELETE FROM enrichment_credentials WHERE user_id = $1 AND service = $2`,
      [req.userId, service]
    );

    res.json({ success: true });
  } catch (error) {
    console.error('Failed to delete credential:', error);
    res.status(500).json({ error: 'Failed to delete credential' });
  }
});

// Helper function to get user's API credentials
async function getUserCredentials(userId: string): Promise<Record<string, string>> {
  const result = await query(
    `SELECT service, api_key_encrypted FROM enrichment_credentials
     WHERE user_id = $1 AND is_valid = true`,
    [userId]
  );

  const credentials: Record<string, string> = {};

  for (const row of result.rows) {
    // In production, decrypt the key here
    const key = row.api_key_encrypted.toString();
    credentials[`${row.service}_key`] = key;
  }

  return credentials;
}

export default router;
