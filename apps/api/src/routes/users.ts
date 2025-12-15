import { Router, Response } from 'express';
import { z } from 'zod';
import { pool } from '../db.js';
import { authMiddleware, AuthRequest } from '../middleware/auth.js';

const router = Router();

// Validation schemas
const updateSettingsSchema = z.object({
  defaultMode: z.string().optional(),
  autoRecord: z.boolean().optional(),
  stealthModeDefault: z.boolean().optional(),
  theme: z.string().optional(),
});

// GET /users/settings
router.get('/settings', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    let settingsResult = await pool.query(
      'SELECT * FROM user_settings WHERE user_id = $1',
      [req.user!.id]
    );

    // Create default settings if not exists
    if (settingsResult.rows.length === 0) {
      settingsResult = await pool.query(
        `INSERT INTO user_settings (user_id) VALUES ($1) RETURNING *`,
        [req.user!.id]
      );
    }

    const settings = settingsResult.rows[0];
    res.json({
      id: settings.id,
      userId: settings.user_id,
      defaultMode: settings.default_mode,
      autoRecord: settings.auto_record,
      stealthModeDefault: settings.stealth_mode_default,
      theme: settings.theme,
      createdAt: settings.created_at,
      updatedAt: settings.updated_at,
    });
  } catch (err) {
    console.error('Get settings error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// PUT /users/settings
router.put('/settings', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const body = updateSettingsSchema.parse(req.body);

    const settingsResult = await pool.query(
      `UPDATE user_settings SET
        default_mode = COALESCE($2, default_mode),
        auto_record = COALESCE($3, auto_record),
        stealth_mode_default = COALESCE($4, stealth_mode_default),
        theme = COALESCE($5, theme),
        updated_at = NOW()
       WHERE user_id = $1
       RETURNING *`,
      [req.user!.id, body.defaultMode, body.autoRecord, body.stealthModeDefault, body.theme]
    );

    if (settingsResult.rows.length === 0) {
      // Create settings if they don't exist
      const newSettings = await pool.query(
        `INSERT INTO user_settings (user_id, default_mode, auto_record, stealth_mode_default, theme)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING *`,
        [req.user!.id, body.defaultMode || 'sales', body.autoRecord ?? true, body.stealthModeDefault ?? false, body.theme || 'dark']
      );
      const settings = newSettings.rows[0];
      return res.json({
        id: settings.id,
        userId: settings.user_id,
        defaultMode: settings.default_mode,
        autoRecord: settings.auto_record,
        stealthModeDefault: settings.stealth_mode_default,
        theme: settings.theme,
        createdAt: settings.created_at,
        updatedAt: settings.updated_at,
      });
    }

    const settings = settingsResult.rows[0];
    res.json({
      id: settings.id,
      userId: settings.user_id,
      defaultMode: settings.default_mode,
      autoRecord: settings.auto_record,
      stealthModeDefault: settings.stealth_mode_default,
      theme: settings.theme,
      createdAt: settings.created_at,
      updatedAt: settings.updated_at,
    });
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({ error: 'Validation error', details: err.errors });
    }
    console.error('Update settings error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

export default router;
