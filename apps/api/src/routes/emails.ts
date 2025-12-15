/**
 * Email Routes
 * Handles sending, tracking, and managing emails
 */

import { Router, Response } from 'express';
import { z } from 'zod';
import { pool } from '../db.js';
import { authMiddleware, AuthRequest } from '../middleware/auth.js';
import emailService from '../services/email.js';
import aiService from '../services/ai.js';

const router = Router();

// Validation schemas
const sendEmailSchema = z.object({
  leadId: z.string().uuid(),
  templateId: z.string().uuid().optional(),
  subject: z.string().min(1).max(200),
  bodyHtml: z.string().min(1),
  bodyText: z.string().optional(),
  purpose: z.string().min(1),
  attachments: z.array(z.object({
    filename: z.string(),
    type: z.string(),
    url: z.string(),
  })).optional(),
});

const bulkEmailSchema = z.object({
  leadIds: z.array(z.string().uuid()).min(1).max(100),
  templateId: z.string().uuid().optional(),
  subject: z.string().min(1).max(200),
  bodyHtml: z.string().min(1),
  bodyText: z.string().optional(),
  purpose: z.string().min(1),
});

const generateEmailSchema = z.object({
  leadId: z.string().uuid(),
  purpose: z.enum(['cold_outreach', 'follow_up', 'cv_submission', 'meeting_request', 'thank_you']),
  tone: z.enum(['formal', 'professional', 'casual', 'friendly']).default('professional'),
  includeCV: z.boolean().default(false),
  customInstructions: z.string().optional(),
});

// POST /emails/send - Send single email
router.post('/send', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const body = sendEmailSchema.parse(req.body);

    const result = await emailService.sendTrackedEmail({
      userId: req.user!.id,
      leadId: body.leadId,
      templateId: body.templateId,
      subject: body.subject,
      bodyHtml: body.bodyHtml,
      bodyText: body.bodyText,
      purpose: body.purpose,
      attachments: body.attachments,
    });

    res.status(201).json({
      success: true,
      emailId: result.emailId,
      messageId: result.messageId,
    });
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({ error: 'Validation error', details: err.errors });
    }
    console.error('Send email error:', err);
    res.status(500).json({ error: (err as Error).message || 'Failed to send email' });
  }
});

// POST /emails/bulk - Send bulk emails
router.post('/bulk', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const body = bulkEmailSchema.parse(req.body);

    const result = await emailService.sendBulkEmails({
      userId: req.user!.id,
      leadIds: body.leadIds,
      templateId: body.templateId || '',
      subject: body.subject,
      bodyHtml: body.bodyHtml,
      bodyText: body.bodyText,
      purpose: body.purpose,
    });

    res.status(201).json({
      success: true,
      queued: result.queued,
      failed: result.failed,
      emailIds: result.emailIds,
    });
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({ error: 'Validation error', details: err.errors });
    }
    console.error('Bulk email error:', err);
    res.status(500).json({ error: (err as Error).message || 'Failed to send bulk emails' });
  }
});

// POST /emails/generate - AI generate email for lead
router.post('/generate', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const body = generateEmailSchema.parse(req.body);

    const email = await aiService.generateEmail({
      leadId: body.leadId,
      userId: req.user!.id,
      purpose: body.purpose,
      tone: body.tone,
      includeCV: body.includeCV,
      customInstructions: body.customInstructions,
    });

    res.json(email);
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({ error: 'Validation error', details: err.errors });
    }
    console.error('Generate email error:', err);
    res.status(500).json({ error: (err as Error).message || 'Failed to generate email' });
  }
});

// GET /emails - List sent emails
router.get('/', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const page = parseInt(req.query.page as string) || 1;
    const perPage = Math.min(parseInt(req.query.perPage as string) || 20, 100);
    const status = req.query.status as string;
    const leadId = req.query.leadId as string;
    const offset = (page - 1) * perPage;

    const result = await pool.query(
      `SELECT
        e.id, e.to_email, e.subject, e.status, e.sent_at, e.opened_at, e.clicked_at,
        e.metadata,
        l.company_name, l.contact_name
      FROM email_log e
      LEFT JOIN leads l ON (e.metadata->>'leadId')::uuid = l.id
      WHERE e.user_id = $1
        AND ($2::text IS NULL OR e.status = $2)
        AND ($3::uuid IS NULL OR (e.metadata->>'leadId')::uuid = $3)
      ORDER BY e.sent_at DESC NULLS LAST
      LIMIT $4 OFFSET $5`,
      [req.user!.id, status || null, leadId || null, perPage, offset]
    );

    const countResult = await pool.query(
      `SELECT COUNT(*) FROM email_log
       WHERE user_id = $1
        AND ($2::text IS NULL OR status = $2)
        AND ($3::uuid IS NULL OR (metadata->>'leadId')::uuid = $3)`,
      [req.user!.id, status || null, leadId || null]
    );

    const total = parseInt(countResult.rows[0].count, 10);

    res.json({
      emails: result.rows.map(row => ({
        id: row.id,
        toEmail: row.to_email,
        subject: row.subject,
        status: row.status,
        sentAt: row.sent_at,
        openedAt: row.opened_at,
        clickedAt: row.clicked_at,
        leadId: row.metadata?.leadId,
        purpose: row.metadata?.purpose,
        companyName: row.company_name,
        contactName: row.contact_name,
      })),
      total,
      page,
      perPage,
      totalPages: Math.ceil(total / perPage),
    });
  } catch (err) {
    console.error('List emails error:', err);
    res.status(500).json({ error: 'Failed to list emails' });
  }
});

// GET /emails/:id - Get email with full analysis
router.get('/:id', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const email = await emailService.getEmailWithAnalysis(req.params.id, req.user!.id);

    if (!email) {
      return res.status(404).json({ error: 'Email not found' });
    }

    // Get lead analysis if available
    let leadAnalysis = null;
    if (email.lead_id) {
      const leadResult = await pool.query(
        `SELECT custom_fields FROM leads WHERE id = $1`,
        [email.lead_id]
      );
      if (leadResult.rows.length > 0 && leadResult.rows[0].custom_fields?.aiAnalysis) {
        leadAnalysis = leadResult.rows[0].custom_fields.aiAnalysis;
      }
    }

    res.json({
      id: email.id,
      toEmail: email.to_email,
      fromEmail: email.from_email,
      subject: email.subject,
      status: email.status,
      sentAt: email.sent_at,
      openedAt: email.opened_at,
      clickedAt: email.clicked_at,
      metadata: email.metadata,
      // Lead info
      lead: email.lead_id ? {
        id: email.lead_id,
        companyName: email.company_name,
        contactName: email.contact_name,
        contactTitle: email.contact_title,
        contactEmail: email.contact_email,
        contactLinkedin: email.contact_linkedin,
        industry: email.industry,
        companySize: email.company_size,
        status: email.lead_status,
      } : null,
      // AI analysis
      analysis: leadAnalysis,
    });
  } catch (err) {
    console.error('Get email error:', err);
    res.status(500).json({ error: 'Failed to get email' });
  }
});

// POST /emails/:id/mark-replied - Mark email as replied
router.post('/:id/mark-replied', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    await emailService.markEmailReplied(req.params.id, req.user!.id);
    res.json({ success: true });
  } catch (err) {
    console.error('Mark replied error:', err);
    res.status(500).json({ error: 'Failed to mark as replied' });
  }
});

// DELETE /emails/:id - Delete email
router.delete('/:id', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const result = await pool.query(
      `DELETE FROM email_log WHERE id = $1 AND user_id = $2 RETURNING id`,
      [req.params.id, req.user!.id]
    );

    if (result.rowCount === 0) {
      return res.status(404).json({ error: 'Email not found' });
    }

    res.json({ success: true });
  } catch (err) {
    console.error('Delete email error:', err);
    res.status(500).json({ error: 'Failed to delete email' });
  }
});

// GET /emails/stats - Get email statistics
router.get('/stats/overview', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const result = await pool.query(
      `SELECT
        COUNT(*) as total,
        COUNT(*) FILTER (WHERE status = 'sent') as sent,
        COUNT(*) FILTER (WHERE status = 'delivered') as delivered,
        COUNT(*) FILTER (WHERE status = 'opened') as opened,
        COUNT(*) FILTER (WHERE status = 'clicked') as clicked,
        COUNT(*) FILTER (WHERE status = 'replied') as replied,
        COUNT(*) FILTER (WHERE status = 'bounced') as bounced
      FROM email_log
      WHERE user_id = $1
        AND sent_at >= NOW() - INTERVAL '30 days'`,
      [req.user!.id]
    );

    const stats = result.rows[0];
    const total = parseInt(stats.total, 10) || 1;

    res.json({
      total: parseInt(stats.total, 10),
      sent: parseInt(stats.sent, 10),
      delivered: parseInt(stats.delivered, 10),
      opened: parseInt(stats.opened, 10),
      clicked: parseInt(stats.clicked, 10),
      replied: parseInt(stats.replied, 10),
      bounced: parseInt(stats.bounced, 10),
      openRate: ((parseInt(stats.opened, 10) / total) * 100).toFixed(1),
      clickRate: ((parseInt(stats.clicked, 10) / total) * 100).toFixed(1),
      replyRate: ((parseInt(stats.replied, 10) / total) * 100).toFixed(1),
    });
  } catch (err) {
    console.error('Get stats error:', err);
    res.status(500).json({ error: 'Failed to get statistics' });
  }
});

export default router;
