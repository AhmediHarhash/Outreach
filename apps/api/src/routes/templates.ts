/**
 * Email Template Routes
 * CRUD operations for email templates
 */

import { Router, Response } from 'express';
import { z } from 'zod';
import { pool } from '../db.js';
import { authMiddleware, AuthRequest } from '../middleware/auth.js';
import aiService from '../services/ai.js';

const router = Router();

// Validation schemas
const createTemplateSchema = z.object({
  name: z.string().min(1).max(100),
  subject: z.string().min(1).max(200),
  bodyHtml: z.string().min(1),
  bodyText: z.string().optional(),
  category: z.string().optional(),
  variables: z.array(z.string()).optional(),
});

const updateTemplateSchema = createTemplateSchema.partial();

// GET /templates - List templates
router.get('/', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const category = req.query.category as string;

    const result = await pool.query(
      `SELECT
        id, name, subject, body_html, body_text, variables, locale, is_active, created_at, updated_at
      FROM email_templates
      WHERE ($1::text IS NULL OR id LIKE $1 || '%')
      ORDER BY name ASC`,
      [category || null]
    );

    // Also get user's custom templates
    const userResult = await pool.query(
      `SELECT * FROM user_email_templates
       WHERE user_id = $1
       ORDER BY created_at DESC`,
      [req.user!.id]
    );

    res.json({
      systemTemplates: result.rows.map(row => ({
        id: row.id,
        name: row.name,
        subject: row.subject,
        bodyHtml: row.body_html,
        bodyText: row.body_text,
        variables: row.variables,
        isSystem: true,
        createdAt: row.created_at,
      })),
      userTemplates: userResult.rows.map(row => ({
        id: row.id,
        name: row.name,
        subject: row.subject,
        bodyHtml: row.body_html,
        bodyText: row.body_text,
        category: row.category,
        variables: row.variables,
        isSystem: false,
        createdAt: row.created_at,
        updatedAt: row.updated_at,
      })),
    });
  } catch (err) {
    console.error('List templates error:', err);
    res.status(500).json({ error: 'Failed to list templates' });
  }
});

// POST /templates - Create user template
router.post('/', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const body = createTemplateSchema.parse(req.body);

    // Extract variables from template
    const variableMatches = body.bodyHtml.match(/\{\{(\w+)\}\}/g) || [];
    const extractedVariables = [...new Set(variableMatches.map(v => v.replace(/[{}]/g, '')))];
    const variables = body.variables || extractedVariables;

    const result = await pool.query(
      `INSERT INTO user_email_templates (user_id, name, subject, body_html, body_text, category, variables)
       VALUES ($1, $2, $3, $4, $5, $6, $7)
       RETURNING *`,
      [req.user!.id, body.name, body.subject, body.bodyHtml, body.bodyText, body.category, variables]
    );

    const template = result.rows[0];

    res.status(201).json({
      id: template.id,
      name: template.name,
      subject: template.subject,
      bodyHtml: template.body_html,
      bodyText: template.body_text,
      category: template.category,
      variables: template.variables,
      createdAt: template.created_at,
    });
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({ error: 'Validation error', details: err.errors });
    }
    console.error('Create template error:', err);
    res.status(500).json({ error: 'Failed to create template' });
  }
});

// GET /templates/:id - Get single template
router.get('/:id', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    // Try user templates first
    let result = await pool.query(
      `SELECT * FROM user_email_templates WHERE id = $1 AND user_id = $2`,
      [req.params.id, req.user!.id]
    );

    if (result.rows.length === 0) {
      // Try system templates
      result = await pool.query(
        `SELECT * FROM email_templates WHERE id = $1`,
        [req.params.id]
      );
    }

    if (result.rows.length === 0) {
      return res.status(404).json({ error: 'Template not found' });
    }

    const template = result.rows[0];

    res.json({
      id: template.id,
      name: template.name,
      subject: template.subject,
      bodyHtml: template.body_html,
      bodyText: template.body_text,
      category: template.category,
      variables: template.variables,
      isSystem: !template.user_id,
      createdAt: template.created_at,
      updatedAt: template.updated_at,
    });
  } catch (err) {
    console.error('Get template error:', err);
    res.status(500).json({ error: 'Failed to get template' });
  }
});

// PUT /templates/:id - Update user template
router.put('/:id', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const body = updateTemplateSchema.parse(req.body);

    // Extract variables if body changed
    let variables = body.variables;
    if (body.bodyHtml && !variables) {
      const variableMatches = body.bodyHtml.match(/\{\{(\w+)\}\}/g) || [];
      variables = [...new Set(variableMatches.map(v => v.replace(/[{}]/g, '')))];
    }

    const result = await pool.query(
      `UPDATE user_email_templates SET
        name = COALESCE($3, name),
        subject = COALESCE($4, subject),
        body_html = COALESCE($5, body_html),
        body_text = COALESCE($6, body_text),
        category = COALESCE($7, category),
        variables = COALESCE($8, variables),
        updated_at = NOW()
      WHERE id = $1 AND user_id = $2
      RETURNING *`,
      [req.params.id, req.user!.id, body.name, body.subject, body.bodyHtml, body.bodyText, body.category, variables]
    );

    if (result.rows.length === 0) {
      return res.status(404).json({ error: 'Template not found' });
    }

    const template = result.rows[0];

    res.json({
      id: template.id,
      name: template.name,
      subject: template.subject,
      bodyHtml: template.body_html,
      bodyText: template.body_text,
      category: template.category,
      variables: template.variables,
      updatedAt: template.updated_at,
    });
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({ error: 'Validation error', details: err.errors });
    }
    console.error('Update template error:', err);
    res.status(500).json({ error: 'Failed to update template' });
  }
});

// DELETE /templates/:id - Delete user template
router.delete('/:id', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const result = await pool.query(
      `DELETE FROM user_email_templates WHERE id = $1 AND user_id = $2 RETURNING id`,
      [req.params.id, req.user!.id]
    );

    if (result.rowCount === 0) {
      return res.status(404).json({ error: 'Template not found' });
    }

    res.json({ success: true });
  } catch (err) {
    console.error('Delete template error:', err);
    res.status(500).json({ error: 'Failed to delete template' });
  }
});

// POST /templates/:id/preview - Preview template with lead data
router.post('/:id/preview', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const { leadId } = req.body;

    if (!leadId) {
      return res.status(400).json({ error: 'leadId is required' });
    }

    // Get template
    let templateResult = await pool.query(
      `SELECT * FROM user_email_templates WHERE id = $1 AND user_id = $2`,
      [req.params.id, req.user!.id]
    );

    if (templateResult.rows.length === 0) {
      templateResult = await pool.query(
        `SELECT * FROM email_templates WHERE id = $1`,
        [req.params.id]
      );
    }

    if (templateResult.rows.length === 0) {
      return res.status(404).json({ error: 'Template not found' });
    }

    const template = templateResult.rows[0];

    // Personalize for lead
    const personalizedHtml = await aiService.personalizeTemplate(
      template.body_html,
      leadId,
      req.user!.id
    );

    const personalizedSubject = await aiService.personalizeTemplate(
      template.subject,
      leadId,
      req.user!.id
    );

    res.json({
      subject: personalizedSubject,
      bodyHtml: personalizedHtml,
    });
  } catch (err) {
    console.error('Preview template error:', err);
    res.status(500).json({ error: 'Failed to preview template' });
  }
});

// POST /templates/generate - AI generate template
router.post('/generate', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const { purpose, tone, industry, description } = req.body;

    const prompt = `Create a professional email template for the following:
- Purpose: ${purpose || 'cold outreach'}
- Tone: ${tone || 'professional'}
- Industry: ${industry || 'general business'}
- Description: ${description || 'reaching out to potential clients'}

The template should use these variables where appropriate:
- {{firstName}} - recipient's first name
- {{companyName}} - recipient's company
- {{title}} - recipient's job title
- {{icebreaker}} - AI-generated opener (will be personalized)
- {{senderName}} - sender's name

Create a template that's:
1. Under 150 words
2. Personalized but not creepy
3. Has a clear value proposition
4. Ends with a soft call-to-action

Respond in JSON format:
{
  "name": "<template name>",
  "subject": "<subject line with optional {{variables}}>",
  "bodyHtml": "<full HTML email template>",
  "bodyText": "<plain text version>"
}`;

    const response = await aiService.callClaude(prompt);

    let jsonStr = response;
    if (response.includes('```')) {
      jsonStr = response.split('```')[1].split('```')[0].replace('json', '');
    }

    const template = JSON.parse(jsonStr.trim());

    res.json(template);
  } catch (err) {
    console.error('Generate template error:', err);
    res.status(500).json({ error: 'Failed to generate template' });
  }
});

export default router;
