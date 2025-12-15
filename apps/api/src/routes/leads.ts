import { Router, Response } from 'express';
import { z } from 'zod';
import { pool } from '../db.js';
import { authMiddleware, AuthRequest } from '../middleware/auth.js';

const router = Router();

// Validation schemas
const createLeadSchema = z.object({
  companyName: z.string().min(1),
  companyDomain: z.string().optional(),
  companyLinkedin: z.string().optional(),
  companySize: z.string().optional(),
  industry: z.string().optional(),
  location: z.string().optional(),
  contactName: z.string().optional(),
  contactTitle: z.string().optional(),
  contactEmail: z.string().email().optional(),
  contactPhone: z.string().optional(),
  contactLinkedin: z.string().optional(),
  status: z.string().default('new'),
  priority: z.number().min(0).max(5).default(0),
  estimatedValue: z.number().optional(),
  source: z.string().optional(),
  tags: z.array(z.string()).optional(),
  notes: z.string().optional(),
  nextFollowupAt: z.string().datetime().optional(),
});

const updateLeadSchema = createLeadSchema.partial();

const listQuerySchema = z.object({
  page: z.coerce.number().min(1).default(1),
  perPage: z.coerce.number().min(1).max(100).default(20),
  status: z.string().optional(),
  priority: z.coerce.number().optional(),
  search: z.string().optional(),
  sortBy: z.string().default('created_at'),
  sortOrder: z.enum(['asc', 'desc']).default('desc'),
});

// GET /leads
router.get('/', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const query = listQuerySchema.parse(req.query);
    const offset = (query.page - 1) * query.perPage;

    // Get leads with filtering
    const leadsResult = await pool.query(
      `SELECT * FROM leads
       WHERE user_id = $1
         AND ($2::text IS NULL OR status = $2)
         AND ($3::int IS NULL OR priority >= $3)
         AND ($4::text IS NULL OR
              company_name ILIKE '%' || $4 || '%' OR
              contact_name ILIKE '%' || $4 || '%' OR
              contact_email ILIKE '%' || $4 || '%')
       ORDER BY
         CASE WHEN $5 = 'created_at' AND $6 = 'desc' THEN created_at END DESC,
         CASE WHEN $5 = 'created_at' AND $6 = 'asc' THEN created_at END ASC,
         CASE WHEN $5 = 'priority' AND $6 = 'desc' THEN priority END DESC,
         CASE WHEN $5 = 'priority' AND $6 = 'asc' THEN priority END ASC,
         CASE WHEN $5 = 'company_name' AND $6 = 'desc' THEN company_name END DESC,
         CASE WHEN $5 = 'company_name' AND $6 = 'asc' THEN company_name END ASC,
         created_at DESC
       LIMIT $7 OFFSET $8`,
      [req.user!.id, query.status || null, query.priority || null, query.search || null,
       query.sortBy, query.sortOrder, query.perPage, offset]
    );

    // Get total count
    const countResult = await pool.query(
      `SELECT COUNT(*) FROM leads
       WHERE user_id = $1
         AND ($2::text IS NULL OR status = $2)
         AND ($3::int IS NULL OR priority >= $3)
         AND ($4::text IS NULL OR
              company_name ILIKE '%' || $4 || '%' OR
              contact_name ILIKE '%' || $4 || '%' OR
              contact_email ILIKE '%' || $4 || '%')`,
      [req.user!.id, query.status || null, query.priority || null, query.search || null]
    );

    const total = parseInt(countResult.rows[0].count, 10);
    const totalPages = Math.ceil(total / query.perPage);

    res.json({
      leads: leadsResult.rows.map(formatLead),
      total,
      page: query.page,
      perPage: query.perPage,
      totalPages,
    });
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({ error: 'Validation error', details: err.errors });
    }
    console.error('List leads error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// POST /leads
router.post('/', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const body = createLeadSchema.parse(req.body);

    const result = await pool.query(
      `INSERT INTO leads (
        user_id, company_name, company_domain, company_linkedin,
        company_size, industry, location,
        contact_name, contact_title, contact_email, contact_phone, contact_linkedin,
        status, priority, estimated_value, source, tags, notes, next_followup_at
      )
      VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
      RETURNING *`,
      [
        req.user!.id, body.companyName, body.companyDomain, body.companyLinkedin,
        body.companySize, body.industry, body.location,
        body.contactName, body.contactTitle, body.contactEmail, body.contactPhone, body.contactLinkedin,
        body.status, body.priority, body.estimatedValue, body.source, body.tags, body.notes,
        body.nextFollowupAt ? new Date(body.nextFollowupAt) : null
      ]
    );

    const lead = result.rows[0];

    // Create sync event
    await pool.query(
      `INSERT INTO sync_events (user_id, entity_type, entity_id, event_type, payload, version)
       VALUES ($1, 'lead', $2, 'created', $3, 1)`,
      [req.user!.id, lead.id, JSON.stringify(lead)]
    );

    // Log activity
    await pool.query(
      `INSERT INTO activity_log (user_id, activity_type, entity_type, entity_id)
       VALUES ($1, 'lead_created', 'lead', $2)`,
      [req.user!.id, lead.id]
    );

    res.status(201).json(formatLead(lead));
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({ error: 'Validation error', details: err.errors });
    }
    console.error('Create lead error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// GET /leads/:id
router.get('/:id', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const result = await pool.query(
      'SELECT * FROM leads WHERE id = $1 AND user_id = $2',
      [req.params.id, req.user!.id]
    );

    if (result.rows.length === 0) {
      return res.status(404).json({ error: 'Lead not found' });
    }

    res.json(formatLead(result.rows[0]));
  } catch (err) {
    console.error('Get lead error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// PUT /leads/:id
router.put('/:id', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const body = updateLeadSchema.parse(req.body);

    // Check ownership
    const existing = await pool.query(
      'SELECT id FROM leads WHERE id = $1 AND user_id = $2',
      [req.params.id, req.user!.id]
    );

    if (existing.rows.length === 0) {
      return res.status(404).json({ error: 'Lead not found' });
    }

    const result = await pool.query(
      `UPDATE leads SET
        company_name = COALESCE($3, company_name),
        company_domain = COALESCE($4, company_domain),
        company_linkedin = COALESCE($5, company_linkedin),
        company_size = COALESCE($6, company_size),
        industry = COALESCE($7, industry),
        location = COALESCE($8, location),
        contact_name = COALESCE($9, contact_name),
        contact_title = COALESCE($10, contact_title),
        contact_email = COALESCE($11, contact_email),
        contact_phone = COALESCE($12, contact_phone),
        contact_linkedin = COALESCE($13, contact_linkedin),
        status = COALESCE($14, status),
        priority = COALESCE($15, priority),
        estimated_value = COALESCE($16, estimated_value),
        tags = COALESCE($17, tags),
        notes = COALESCE($18, notes),
        next_followup_at = COALESCE($19, next_followup_at),
        updated_at = NOW()
       WHERE id = $1 AND user_id = $2
       RETURNING *`,
      [
        req.params.id, req.user!.id,
        body.companyName, body.companyDomain, body.companyLinkedin,
        body.companySize, body.industry, body.location,
        body.contactName, body.contactTitle, body.contactEmail, body.contactPhone, body.contactLinkedin,
        body.status, body.priority, body.estimatedValue, body.tags, body.notes,
        body.nextFollowupAt ? new Date(body.nextFollowupAt) : null
      ]
    );

    const lead = result.rows[0];

    // Create sync event
    const versionResult = await pool.query(
      `SELECT COALESCE(MAX(version), 0) + 1 as version FROM sync_events WHERE entity_type = 'lead' AND entity_id = $1`,
      [lead.id]
    );
    const version = versionResult.rows[0].version;

    await pool.query(
      `INSERT INTO sync_events (user_id, entity_type, entity_id, event_type, payload, version)
       VALUES ($1, 'lead', $2, 'updated', $3, $4)`,
      [req.user!.id, lead.id, JSON.stringify(lead), version]
    );

    res.json(formatLead(lead));
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({ error: 'Validation error', details: err.errors });
    }
    console.error('Update lead error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// DELETE /leads/:id
router.delete('/:id', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const result = await pool.query(
      'DELETE FROM leads WHERE id = $1 AND user_id = $2 RETURNING id',
      [req.params.id, req.user!.id]
    );

    if (result.rowCount === 0) {
      return res.status(404).json({ error: 'Lead not found' });
    }

    // Create sync event for deletion
    await pool.query(
      `INSERT INTO sync_events (user_id, entity_type, entity_id, event_type, payload, version)
       VALUES ($1, 'lead', $2, 'deleted', '{}',
         COALESCE((SELECT MAX(version) FROM sync_events WHERE entity_type = 'lead' AND entity_id = $2), 0) + 1)`,
      [req.user!.id, req.params.id]
    );

    res.json({ success: true });
  } catch (err) {
    console.error('Delete lead error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// Helper function to format lead response
function formatLead(row: any) {
  return {
    id: row.id,
    userId: row.user_id,
    companyName: row.company_name,
    companyDomain: row.company_domain,
    companyLinkedin: row.company_linkedin,
    companySize: row.company_size,
    industry: row.industry,
    location: row.location,
    contactName: row.contact_name,
    contactTitle: row.contact_title,
    contactEmail: row.contact_email,
    contactPhone: row.contact_phone,
    contactLinkedin: row.contact_linkedin,
    status: row.status,
    priority: row.priority,
    estimatedValue: row.estimated_value,
    techStack: row.tech_stack,
    fundingInfo: row.funding_info,
    recentNews: row.recent_news,
    employeeCount: row.employee_count,
    source: row.source,
    tags: row.tags,
    notes: row.notes,
    customFields: row.custom_fields,
    lastContactedAt: row.last_contacted_at,
    nextFollowupAt: row.next_followup_at,
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
}

export default router;
