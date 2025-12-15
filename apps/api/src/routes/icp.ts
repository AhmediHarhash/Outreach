/**
 * ICP (Ideal Customer Profile) Routes
 * Manage user's ICP profiles for lead scoring and discovery
 */

import { Router, Request, Response } from 'express';
import { authMiddleware, AuthRequest } from '../middleware/auth.js';
import { pool } from '../db.js';
import { isValidUUID, sanitizeString } from '../utils/validation.js';

const router = Router();

// All routes require authentication
router.use(authMiddleware);

/**
 * GET /icp
 * List all ICP profiles for the user
 */
router.get('/', async (req: AuthRequest, res: Response) => {
  try {
    const result = await pool.query(
      `SELECT id, name, description, is_default,
              industries, company_size_min, company_size_max,
              funding_stages, countries, target_titles,
              weight_intent, weight_fit, weight_accessibility,
              created_at, updated_at
       FROM icp_profiles
       WHERE user_id = $1
       ORDER BY is_default DESC, created_at DESC`,
      [req.user!.id]
    );

    res.json({
      profiles: result.rows,
      count: result.rows.length,
    });
  } catch (error) {
    console.error('Failed to fetch ICP profiles:', error);
    res.status(500).json({ error: 'Failed to fetch ICP profiles' });
  }
});

/**
 * GET /icp/:id
 * Get a specific ICP profile
 */
router.get('/:id', async (req: AuthRequest, res: Response) => {
  try {
    const { id } = req.params;

    if (!isValidUUID(id)) {
      return res.status(400).json({ error: 'Invalid ICP ID' });
    }

    const result = await pool.query(
      `SELECT * FROM icp_profiles WHERE id = $1 AND user_id = $2`,
      [id, req.user!.id]
    );

    if (result.rows.length === 0) {
      return res.status(404).json({ error: 'ICP profile not found' });
    }

    res.json(result.rows[0]);
  } catch (error) {
    console.error('Failed to fetch ICP profile:', error);
    res.status(500).json({ error: 'Failed to fetch ICP profile' });
  }
});

/**
 * POST /icp
 * Create a new ICP profile
 */
router.post('/', async (req: AuthRequest, res: Response) => {
  try {
    const {
      name,
      description,
      is_default = false,
      industries = [],
      excluded_industries = [],
      company_size_min,
      company_size_max,
      revenue_min,
      revenue_max,
      funding_stages = [],
      min_funding_amount,
      recently_funded_days,
      tech_must_have = [],
      tech_nice_to_have = [],
      tech_avoid = [],
      countries = [],
      excluded_countries = [],
      regions = [],
      target_titles = [],
      target_departments = [],
      seniority_levels = [],
      require_recent_funding = false,
      require_hiring_signals = false,
      require_tech_change = false,
      weight_intent = 40,
      weight_fit = 35,
      weight_accessibility = 25,
    } = req.body;

    if (!name || name.trim().length === 0) {
      return res.status(400).json({ error: 'Name is required' });
    }

    // Validate weights sum to 100
    if (weight_intent + weight_fit + weight_accessibility !== 100) {
      return res.status(400).json({
        error: 'Scoring weights must sum to 100',
      });
    }

    // If setting as default, unset other defaults first
    if (is_default) {
      await pool.query(
        `UPDATE icp_profiles SET is_default = false WHERE user_id = $1`,
        [req.user!.id]
      );
    }

    const result = await pool.query(
      `INSERT INTO icp_profiles (
        user_id, name, description, is_default,
        industries, excluded_industries,
        company_size_min, company_size_max,
        revenue_min, revenue_max,
        funding_stages, min_funding_amount, recently_funded_days,
        tech_must_have, tech_nice_to_have, tech_avoid,
        countries, excluded_countries, regions,
        target_titles, target_departments, seniority_levels,
        require_recent_funding, require_hiring_signals, require_tech_change,
        weight_intent, weight_fit, weight_accessibility
      )
      VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28)
      RETURNING *`,
      [
        req.user!.id,
        sanitizeString(name),
        description ? sanitizeString(description) : null,
        is_default,
        JSON.stringify(industries),
        JSON.stringify(excluded_industries),
        company_size_min || null,
        company_size_max || null,
        revenue_min || null,
        revenue_max || null,
        JSON.stringify(funding_stages),
        min_funding_amount || null,
        recently_funded_days || null,
        JSON.stringify(tech_must_have),
        JSON.stringify(tech_nice_to_have),
        JSON.stringify(tech_avoid),
        JSON.stringify(countries),
        JSON.stringify(excluded_countries),
        JSON.stringify(regions),
        JSON.stringify(target_titles),
        JSON.stringify(target_departments),
        JSON.stringify(seniority_levels),
        require_recent_funding,
        require_hiring_signals,
        require_tech_change,
        weight_intent,
        weight_fit,
        weight_accessibility,
      ]
    );

    res.status(201).json(result.rows[0]);
  } catch (error) {
    console.error('Failed to create ICP profile:', error);
    res.status(500).json({ error: 'Failed to create ICP profile' });
  }
});

/**
 * PUT /icp/:id
 * Update an ICP profile
 */
router.put('/:id', async (req: AuthRequest, res: Response) => {
  try {
    const { id } = req.params;

    if (!isValidUUID(id)) {
      return res.status(400).json({ error: 'Invalid ICP ID' });
    }

    // Check ownership
    const existing = await pool.query(
      `SELECT id FROM icp_profiles WHERE id = $1 AND user_id = $2`,
      [id, req.user!.id]
    );

    if (existing.rows.length === 0) {
      return res.status(404).json({ error: 'ICP profile not found' });
    }

    const {
      name,
      description,
      is_default,
      industries,
      excluded_industries,
      company_size_min,
      company_size_max,
      revenue_min,
      revenue_max,
      funding_stages,
      min_funding_amount,
      recently_funded_days,
      tech_must_have,
      tech_nice_to_have,
      tech_avoid,
      countries,
      excluded_countries,
      regions,
      target_titles,
      target_departments,
      seniority_levels,
      require_recent_funding,
      require_hiring_signals,
      require_tech_change,
      weight_intent,
      weight_fit,
      weight_accessibility,
    } = req.body;

    // Validate weights if provided
    if (weight_intent !== undefined && weight_fit !== undefined && weight_accessibility !== undefined) {
      if (weight_intent + weight_fit + weight_accessibility !== 100) {
        return res.status(400).json({
          error: 'Scoring weights must sum to 100',
        });
      }
    }

    // If setting as default, unset other defaults first
    if (is_default) {
      await pool.query(
        `UPDATE icp_profiles SET is_default = false WHERE user_id = $1 AND id != $2`,
        [req.user!.id, id]
      );
    }

    const result = await pool.query(
      `UPDATE icp_profiles SET
        name = COALESCE($1, name),
        description = COALESCE($2, description),
        is_default = COALESCE($3, is_default),
        industries = COALESCE($4, industries),
        excluded_industries = COALESCE($5, excluded_industries),
        company_size_min = COALESCE($6, company_size_min),
        company_size_max = COALESCE($7, company_size_max),
        revenue_min = COALESCE($8, revenue_min),
        revenue_max = COALESCE($9, revenue_max),
        funding_stages = COALESCE($10, funding_stages),
        min_funding_amount = COALESCE($11, min_funding_amount),
        recently_funded_days = COALESCE($12, recently_funded_days),
        tech_must_have = COALESCE($13, tech_must_have),
        tech_nice_to_have = COALESCE($14, tech_nice_to_have),
        tech_avoid = COALESCE($15, tech_avoid),
        countries = COALESCE($16, countries),
        excluded_countries = COALESCE($17, excluded_countries),
        regions = COALESCE($18, regions),
        target_titles = COALESCE($19, target_titles),
        target_departments = COALESCE($20, target_departments),
        seniority_levels = COALESCE($21, seniority_levels),
        require_recent_funding = COALESCE($22, require_recent_funding),
        require_hiring_signals = COALESCE($23, require_hiring_signals),
        require_tech_change = COALESCE($24, require_tech_change),
        weight_intent = COALESCE($25, weight_intent),
        weight_fit = COALESCE($26, weight_fit),
        weight_accessibility = COALESCE($27, weight_accessibility),
        updated_at = NOW()
      WHERE id = $28 AND user_id = $29
      RETURNING *`,
      [
        name ? sanitizeString(name) : null,
        description !== undefined ? (description ? sanitizeString(description) : null) : null,
        is_default,
        industries ? JSON.stringify(industries) : null,
        excluded_industries ? JSON.stringify(excluded_industries) : null,
        company_size_min,
        company_size_max,
        revenue_min,
        revenue_max,
        funding_stages ? JSON.stringify(funding_stages) : null,
        min_funding_amount,
        recently_funded_days,
        tech_must_have ? JSON.stringify(tech_must_have) : null,
        tech_nice_to_have ? JSON.stringify(tech_nice_to_have) : null,
        tech_avoid ? JSON.stringify(tech_avoid) : null,
        countries ? JSON.stringify(countries) : null,
        excluded_countries ? JSON.stringify(excluded_countries) : null,
        regions ? JSON.stringify(regions) : null,
        target_titles ? JSON.stringify(target_titles) : null,
        target_departments ? JSON.stringify(target_departments) : null,
        seniority_levels ? JSON.stringify(seniority_levels) : null,
        require_recent_funding,
        require_hiring_signals,
        require_tech_change,
        weight_intent,
        weight_fit,
        weight_accessibility,
        id,
        req.user!.id,
      ]
    );

    res.json(result.rows[0]);
  } catch (error) {
    console.error('Failed to update ICP profile:', error);
    res.status(500).json({ error: 'Failed to update ICP profile' });
  }
});

/**
 * DELETE /icp/:id
 * Delete an ICP profile
 */
router.delete('/:id', async (req: AuthRequest, res: Response) => {
  try {
    const { id } = req.params;

    if (!isValidUUID(id)) {
      return res.status(400).json({ error: 'Invalid ICP ID' });
    }

    const result = await pool.query(
      `DELETE FROM icp_profiles WHERE id = $1 AND user_id = $2 RETURNING id`,
      [id, req.user!.id]
    );

    if (result.rows.length === 0) {
      return res.status(404).json({ error: 'ICP profile not found' });
    }

    res.json({ success: true, deleted: id });
  } catch (error) {
    console.error('Failed to delete ICP profile:', error);
    res.status(500).json({ error: 'Failed to delete ICP profile' });
  }
});

/**
 * POST /icp/:id/set-default
 * Set an ICP profile as the default
 */
router.post('/:id/set-default', async (req: AuthRequest, res: Response) => {
  try {
    const { id } = req.params;

    if (!isValidUUID(id)) {
      return res.status(400).json({ error: 'Invalid ICP ID' });
    }

    // Unset all defaults first
    await pool.query(
      `UPDATE icp_profiles SET is_default = false WHERE user_id = $1`,
      [req.user!.id]
    );

    // Set the specified profile as default
    const result = await pool.query(
      `UPDATE icp_profiles SET is_default = true, updated_at = NOW()
       WHERE id = $1 AND user_id = $2
       RETURNING *`,
      [id, req.user!.id]
    );

    if (result.rows.length === 0) {
      return res.status(404).json({ error: 'ICP profile not found' });
    }

    res.json(result.rows[0]);
  } catch (error) {
    console.error('Failed to set default ICP:', error);
    res.status(500).json({ error: 'Failed to set default ICP' });
  }
});

export default router;
