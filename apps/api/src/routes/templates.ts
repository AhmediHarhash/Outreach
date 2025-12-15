/**
 * Email Template Routes
 * CRUD operations for email templates with security hardening
 */

import { Router, Response } from 'express';
import { z } from 'zod';
import { pool } from '../db.js';
import { authMiddleware, AuthRequest } from '../middleware/auth.js';
import { aiGenerationLimiter } from '../middleware/rateLimiter.js';
import aiService from '../services/ai.js';
import {
  isValidUUID,
  sanitizeHtml,
  ALLOWED_PURPOSES,
  ALLOWED_TONES,
} from '../utils/validation.js';

const router = Router();

// ============================================================================
// VALIDATION SCHEMAS
// ============================================================================

const createTemplateSchema = z.object({
  name: z.string().min(1).max(100).transform(s => s.trim()),
  subject: z.string().min(1).max(200).transform(s => s.trim()),
  bodyHtml: z.string().min(1).max(50000).transform(sanitizeHtml),
  bodyText: z.string().max(50000).optional(),
  category: z.string().max(50).optional(),
  variables: z.array(z.string().max(50)).max(20).optional(),
});

const updateTemplateSchema = createTemplateSchema.partial();

const generateTemplateSchema = z.object({
  purpose: z.enum(ALLOWED_PURPOSES).optional().default('cold_outreach'),
  tone: z.enum(ALLOWED_TONES).optional().default('professional'),
  industry: z.string().max(100).optional().transform(s => s?.trim() || 'general business'),
  description: z.string().max(500).optional().transform(s => s?.trim() || ''),
});

const previewTemplateSchema = z.object({
  leadId: z.string().refine(isValidUUID, 'Invalid lead ID format'),
});

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

function formatTemplateResponse(row: any, isSystem: boolean = false) {
  return {
    id: row.id,
    name: row.name,
    subject: row.subject,
    bodyHtml: row.body_html,
    bodyText: row.body_text,
    category: row.category,
    variables: row.variables,
    isSystem,
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
}

// ============================================================================
// ROUTES
// ============================================================================

// GET /templates - List templates
router.get('/', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const category = req.query.category as string | undefined;

    // Get system templates
    const result = await pool.query(
      `SELECT
        id, name, subject, body_html, body_text, variables, locale, is_active, created_at, updated_at
      FROM email_templates
      WHERE is_active = true
      ORDER BY name ASC`
    );

    // Get user's custom templates
    const userResult = await pool.query(
      `SELECT * FROM user_email_templates
       WHERE user_id = $1
       ${category ? 'AND category = $2' : ''}
       ORDER BY created_at DESC`,
      category ? [req.user!.id, category] : [req.user!.id]
    );

    res.json({
      systemTemplates: result.rows.map(row => formatTemplateResponse(row, true)),
      userTemplates: userResult.rows.map(row => formatTemplateResponse(row, false)),
    });
  } catch (err) {
    console.error('List templates error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'Failed to list templates',
      },
    });
  }
});

// POST /templates - Create user template
router.post('/', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const body = createTemplateSchema.parse(req.body);

    // Extract variables from template if not provided
    const variableMatches = body.bodyHtml.match(/\{\{(\w+)\}\}/g) || [];
    const extractedVariables = [...new Set(variableMatches.map(v => v.replace(/[{}]/g, '')))];
    const variables = body.variables || extractedVariables;

    const result = await pool.query(
      `INSERT INTO user_email_templates (user_id, name, subject, body_html, body_text, category, variables)
       VALUES ($1, $2, $3, $4, $5, $6, $7)
       RETURNING *`,
      [req.user!.id, body.name, body.subject, body.bodyHtml, body.bodyText, body.category, variables]
    );

    res.status(201).json(formatTemplateResponse(result.rows[0], false));
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({
        error: {
          code: 'VALIDATION_ERROR',
          message: 'Invalid input',
          details: err.errors,
        },
      });
    }
    console.error('Create template error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'Failed to create template',
      },
    });
  }
});

// GET /templates/:id - Get single template
router.get('/:id', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const { id } = req.params;

    // Validate UUID format
    if (!isValidUUID(id)) {
      return res.status(400).json({
        error: {
          code: 'INVALID_ID',
          message: 'Invalid template ID format',
        },
      });
    }

    // Try user templates first
    let result = await pool.query(
      `SELECT * FROM user_email_templates WHERE id = $1 AND user_id = $2`,
      [id, req.user!.id]
    );

    let isSystem = false;
    if (result.rows.length === 0) {
      // Try system templates
      result = await pool.query(
        `SELECT * FROM email_templates WHERE id = $1`,
        [id]
      );
      isSystem = true;
    }

    if (result.rows.length === 0) {
      return res.status(404).json({
        error: {
          code: 'NOT_FOUND',
          message: 'Template not found',
        },
      });
    }

    res.json(formatTemplateResponse(result.rows[0], isSystem));
  } catch (err) {
    console.error('Get template error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'Failed to get template',
      },
    });
  }
});

// PUT /templates/:id - Update user template
router.put('/:id', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const { id } = req.params;

    // Validate UUID format
    if (!isValidUUID(id)) {
      return res.status(400).json({
        error: {
          code: 'INVALID_ID',
          message: 'Invalid template ID format',
        },
      });
    }

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
      [id, req.user!.id, body.name, body.subject, body.bodyHtml, body.bodyText, body.category, variables]
    );

    if (result.rows.length === 0) {
      return res.status(404).json({
        error: {
          code: 'NOT_FOUND',
          message: 'Template not found or you do not have permission to update it',
        },
      });
    }

    res.json(formatTemplateResponse(result.rows[0], false));
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({
        error: {
          code: 'VALIDATION_ERROR',
          message: 'Invalid input',
          details: err.errors,
        },
      });
    }
    console.error('Update template error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'Failed to update template',
      },
    });
  }
});

// DELETE /templates/:id - Delete user template
router.delete('/:id', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const { id } = req.params;

    // Validate UUID format
    if (!isValidUUID(id)) {
      return res.status(400).json({
        error: {
          code: 'INVALID_ID',
          message: 'Invalid template ID format',
        },
      });
    }

    const result = await pool.query(
      `DELETE FROM user_email_templates WHERE id = $1 AND user_id = $2 RETURNING id`,
      [id, req.user!.id]
    );

    if (result.rowCount === 0) {
      return res.status(404).json({
        error: {
          code: 'NOT_FOUND',
          message: 'Template not found or you do not have permission to delete it',
        },
      });
    }

    res.json({ success: true });
  } catch (err) {
    console.error('Delete template error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'Failed to delete template',
      },
    });
  }
});

// POST /templates/:id/preview - Preview template with lead data
router.post('/:id/preview', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const { id } = req.params;

    // Validate UUID format
    if (!isValidUUID(id)) {
      return res.status(400).json({
        error: {
          code: 'INVALID_ID',
          message: 'Invalid template ID format',
        },
      });
    }

    const body = previewTemplateSchema.parse(req.body);

    // Get template
    let templateResult = await pool.query(
      `SELECT * FROM user_email_templates WHERE id = $1 AND user_id = $2`,
      [id, req.user!.id]
    );

    if (templateResult.rows.length === 0) {
      templateResult = await pool.query(
        `SELECT * FROM email_templates WHERE id = $1`,
        [id]
      );
    }

    if (templateResult.rows.length === 0) {
      return res.status(404).json({
        error: {
          code: 'NOT_FOUND',
          message: 'Template not found',
        },
      });
    }

    const template = templateResult.rows[0];

    // Personalize for lead
    const personalizedHtml = await aiService.personalizeTemplate(
      template.body_html,
      body.leadId,
      req.user!.id
    );

    const personalizedSubject = await aiService.personalizeTemplate(
      template.subject,
      body.leadId,
      req.user!.id
    );

    res.json({
      subject: personalizedSubject,
      bodyHtml: personalizedHtml,
    });
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({
        error: {
          code: 'VALIDATION_ERROR',
          message: 'Invalid input',
          details: err.errors,
        },
      });
    }
    console.error('Preview template error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'Failed to preview template',
      },
    });
  }
});

// POST /templates/generate - AI generate template
// Rate limited: 20 generations per hour
router.post('/generate', authMiddleware, aiGenerationLimiter, async (req: AuthRequest, res: Response) => {
  try {
    const body = generateTemplateSchema.parse(req.body);

    const prompt = `Create a professional email template for the following:
- Purpose: ${body.purpose}
- Tone: ${body.tone}
- Industry: ${body.industry}
${body.description ? `- Description: ${body.description}` : ''}

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

Respond ONLY with valid JSON in this exact format (no markdown, no explanation):
{
  "name": "<template name>",
  "subject": "<subject line with optional {{variables}}>",
  "bodyHtml": "<full HTML email template>",
  "bodyText": "<plain text version>"
}`;

    const response = await aiService.callClaude(prompt);

    // Parse JSON response
    let jsonStr = response;
    if (response.includes('```')) {
      const match = response.match(/```(?:json)?\s*([\s\S]*?)```/);
      jsonStr = match ? match[1] : response;
    }

    let template;
    try {
      template = JSON.parse(jsonStr.trim());
    } catch {
      console.error('Failed to parse AI response:', jsonStr);
      return res.status(500).json({
        error: {
          code: 'AI_PARSE_ERROR',
          message: 'Failed to parse AI response. Please try again.',
        },
      });
    }

    // Validate response structure
    if (!template.name || !template.subject || !template.bodyHtml) {
      return res.status(500).json({
        error: {
          code: 'AI_INVALID_RESPONSE',
          message: 'AI returned incomplete template. Please try again.',
        },
      });
    }

    // Sanitize HTML output
    template.bodyHtml = sanitizeHtml(template.bodyHtml);

    res.json(template);
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({
        error: {
          code: 'VALIDATION_ERROR',
          message: 'Invalid input parameters',
          details: err.errors,
        },
      });
    }
    console.error('Generate template error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'Failed to generate template',
      },
    });
  }
});

export default router;
