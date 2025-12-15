/**
 * Authentication Routes
 * With rate limiting and enhanced security
 */

import { Router, Response } from 'express';
import bcrypt from 'bcryptjs';
import jwt from 'jsonwebtoken';
import crypto from 'crypto';
import { z } from 'zod';
import { v4 as uuidv4 } from 'uuid';
import { pool } from '../db.js';
import { config } from '../config.js';
import { authMiddleware, AuthRequest } from '../middleware/auth.js';
import {
  loginLimiter,
  registerLimiter,
  refreshLimiter,
  bruteForceProtection,
  recordFailedAttempt,
  resetFailedAttempts,
} from '../middleware/rateLimiter.js';

const router = Router();

// ============================================================================
// JWT CONFIGURATION
// ============================================================================

const JWT_ISSUER = 'outreach-api';
const JWT_AUDIENCE = 'outreach-app';

// ============================================================================
// VALIDATION SCHEMAS
// ============================================================================

const registerSchema = z.object({
  email: z.string().email().max(255).transform(e => e.toLowerCase().trim()),
  password: z.string().min(8).max(128),
  fullName: z.string().min(1).max(100).optional(),
});

const loginSchema = z.object({
  email: z.string().email().max(255).transform(e => e.toLowerCase().trim()),
  password: z.string().max(128),
  deviceId: z.string().uuid().optional().default(() => uuidv4()),
  deviceName: z.string().max(100).optional(),
});

const refreshSchema = z.object({
  refreshToken: z.string().min(1).max(256),
  deviceId: z.string().uuid(),
});

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

interface TokenUser {
  id: string;
  email: string;
  subscription_tier: string;
  token_version: number;
}

function createAccessToken(user: TokenUser): string {
  return jwt.sign(
    {
      sub: user.id, // Standard claim for subject
      email: user.email,
      subscriptionTier: user.subscription_tier,
      tokenVersion: user.token_version,
    },
    config.jwtSecret,
    {
      expiresIn: config.jwtAccessExpirySecs,
      issuer: JWT_ISSUER,
      audience: JWT_AUDIENCE,
      algorithm: 'HS256',
    }
  );
}

async function createRefreshToken(
  userId: string,
  deviceId: string,
  deviceName: string | null,
  tokenVersion: number
): Promise<string> {
  const token = crypto.randomBytes(64).toString('hex');
  const tokenHash = crypto.createHash('sha256').update(token).digest('hex');
  const expiresAt = new Date(Date.now() + config.jwtRefreshExpiryDays * 24 * 60 * 60 * 1000);

  await pool.query(
    `INSERT INTO refresh_tokens (user_id, token_hash, device_id, device_name, token_version, expires_at)
     VALUES ($1, $2, $3, $4, $5, $6)
     ON CONFLICT (user_id, device_id)
     DO UPDATE SET token_hash = $2, token_version = $5, expires_at = $6, last_used_at = NOW()`,
    [userId, tokenHash, deviceId, deviceName, tokenVersion, expiresAt]
  );

  return token;
}

function sanitizeUser(user: any) {
  return {
    id: user.id,
    email: user.email,
    fullName: user.full_name,
    avatarUrl: user.avatar_url,
    subscriptionTier: user.subscription_tier,
    emailVerified: user.email_verified,
    locale: user.locale,
    timezone: user.timezone,
    createdAt: user.created_at,
  };
}

// ============================================================================
// ROUTES
// ============================================================================

// POST /auth/register
// Rate limited: 3 registrations per hour per IP
router.post('/register', registerLimiter, async (req, res) => {
  try {
    const body = registerSchema.parse(req.body);

    // Check if user exists
    const existing = await pool.query(
      'SELECT id FROM users WHERE email = $1',
      [body.email]
    );
    if (existing.rows.length > 0) {
      return res.status(409).json({
        error: {
          code: 'USER_EXISTS',
          message: 'An account with this email already exists',
        },
      });
    }

    // Hash password with cost factor 12
    const passwordHash = await bcrypt.hash(body.password, 12);

    // Create user
    const userResult = await pool.query(
      `INSERT INTO users (email, password_hash, full_name)
       VALUES ($1, $2, $3)
       RETURNING *`,
      [body.email, passwordHash, body.fullName || null]
    );
    const user = userResult.rows[0];

    // Create default settings
    await pool.query('INSERT INTO user_settings (user_id) VALUES ($1)', [user.id]);

    // Generate device ID for the new registration
    const deviceId = uuidv4();

    // Create tokens
    const accessToken = createAccessToken(user);
    const refreshToken = await createRefreshToken(
      user.id,
      deviceId,
      'Web Registration',
      user.token_version
    );

    // Log activity
    await pool.query(
      `INSERT INTO activity_log (user_id, activity_type, metadata) VALUES ($1, 'register', $2)`,
      [user.id, JSON.stringify({ ip: req.ip })]
    );

    res.status(201).json({
      user: sanitizeUser(user),
      accessToken,
      refreshToken,
      expiresIn: config.jwtAccessExpirySecs,
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
    console.error('Register error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'An error occurred during registration',
      },
    });
  }
});

// POST /auth/login
// Rate limited: 5 attempts per 15 minutes per IP
// Brute force protection: progressive blocking after repeated failures
router.post('/login', bruteForceProtection, loginLimiter, async (req, res) => {
  try {
    const body = loginSchema.parse(req.body);

    // Find user
    const userResult = await pool.query(
      'SELECT * FROM users WHERE email = $1',
      [body.email]
    );

    if (userResult.rows.length === 0) {
      recordFailedAttempt(req);
      return res.status(401).json({
        error: {
          code: 'INVALID_CREDENTIALS',
          message: 'Invalid email or password',
        },
      });
    }

    const user = userResult.rows[0];

    // Verify password
    if (!user.password_hash || !(await bcrypt.compare(body.password, user.password_hash))) {
      recordFailedAttempt(req);
      return res.status(401).json({
        error: {
          code: 'INVALID_CREDENTIALS',
          message: 'Invalid email or password',
        },
      });
    }

    // Success - reset failed attempts
    resetFailedAttempts(req);

    // Create tokens
    const accessToken = createAccessToken(user);
    const refreshToken = await createRefreshToken(
      user.id,
      body.deviceId,
      body.deviceName || null,
      user.token_version
    );

    // Log activity
    await pool.query(
      `INSERT INTO activity_log (user_id, activity_type, metadata) VALUES ($1, 'login', $2)`,
      [user.id, JSON.stringify({ device_id: body.deviceId, ip: req.ip })]
    );

    res.json({
      user: sanitizeUser(user),
      accessToken,
      refreshToken,
      expiresIn: config.jwtAccessExpirySecs,
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
    console.error('Login error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'An error occurred during login',
      },
    });
  }
});

// POST /auth/refresh
// Rate limited: 30 refreshes per hour per IP
router.post('/refresh', refreshLimiter, async (req, res) => {
  try {
    const body = refreshSchema.parse(req.body);
    const tokenHash = crypto.createHash('sha256').update(body.refreshToken).digest('hex');

    // Find and validate refresh token
    const tokenResult = await pool.query(
      `SELECT rt.*, u.token_version as user_token_version
       FROM refresh_tokens rt
       JOIN users u ON u.id = rt.user_id
       WHERE rt.token_hash = $1 AND rt.device_id = $2 AND rt.expires_at > NOW()`,
      [tokenHash, body.deviceId]
    );

    if (tokenResult.rows.length === 0) {
      return res.status(401).json({
        error: {
          code: 'INVALID_REFRESH_TOKEN',
          message: 'Invalid or expired refresh token',
        },
      });
    }

    const storedToken = tokenResult.rows[0];

    // Check token version matches (for global logout)
    if (storedToken.token_version !== storedToken.user_token_version) {
      return res.status(401).json({
        error: {
          code: 'TOKEN_REVOKED',
          message: 'This token has been revoked',
        },
      });
    }

    // Get user
    const userResult = await pool.query('SELECT * FROM users WHERE id = $1', [storedToken.user_id]);
    const user = userResult.rows[0];

    // Rotate refresh token (security best practice)
    const newRefreshToken = await createRefreshToken(
      user.id,
      body.deviceId,
      storedToken.device_name,
      user.token_version
    );

    // Create new access token
    const accessToken = createAccessToken(user);

    res.json({
      user: sanitizeUser(user),
      accessToken,
      refreshToken: newRefreshToken,
      expiresIn: config.jwtAccessExpirySecs,
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
    console.error('Refresh error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'An error occurred during token refresh',
      },
    });
  }
});

// POST /auth/logout
router.post('/logout', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const body = refreshSchema.parse(req.body);
    await pool.query(
      'DELETE FROM refresh_tokens WHERE user_id = $1 AND device_id = $2',
      [req.user!.id, body.deviceId]
    );
    res.json({ success: true });
  } catch (err) {
    console.error('Logout error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'An error occurred during logout',
      },
    });
  }
});

// POST /auth/logout-all
router.post('/logout-all', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    // Increment token version to invalidate ALL refresh tokens
    await pool.query(
      'UPDATE users SET token_version = token_version + 1 WHERE id = $1',
      [req.user!.id]
    );

    // Log activity
    await pool.query(
      `INSERT INTO activity_log (user_id, activity_type, metadata) VALUES ($1, 'logout_all', $2)`,
      [req.user!.id, JSON.stringify({ ip: req.ip })]
    );

    res.json({ success: true });
  } catch (err) {
    console.error('Logout all error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'An error occurred',
      },
    });
  }
});

// GET /auth/me
router.get('/me', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const userResult = await pool.query('SELECT * FROM users WHERE id = $1', [req.user!.id]);
    if (userResult.rows.length === 0) {
      return res.status(404).json({
        error: {
          code: 'USER_NOT_FOUND',
          message: 'User not found',
        },
      });
    }

    const user = userResult.rows[0];
    res.json({
      ...sanitizeUser(user),
      subscriptionExpiresAt: user.subscription_expires_at,
      dataRegion: user.data_region,
      updatedAt: user.updated_at,
    });
  } catch (err) {
    console.error('Get me error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'An error occurred',
      },
    });
  }
});

// GET /auth/sessions - List active sessions
router.get('/sessions', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const result = await pool.query(
      `SELECT device_id, device_name, created_at, last_used_at
       FROM refresh_tokens
       WHERE user_id = $1 AND expires_at > NOW()
       ORDER BY last_used_at DESC`,
      [req.user!.id]
    );

    res.json({
      sessions: result.rows.map(row => ({
        deviceId: row.device_id,
        deviceName: row.device_name,
        createdAt: row.created_at,
        lastUsedAt: row.last_used_at,
      })),
    });
  } catch (err) {
    console.error('Get sessions error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'An error occurred',
      },
    });
  }
});

// DELETE /auth/sessions/:deviceId - Revoke specific session
router.delete('/sessions/:deviceId', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const { deviceId } = req.params;

    // Validate UUID format
    const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
    if (!uuidRegex.test(deviceId)) {
      return res.status(400).json({
        error: {
          code: 'INVALID_DEVICE_ID',
          message: 'Invalid device ID format',
        },
      });
    }

    await pool.query(
      'DELETE FROM refresh_tokens WHERE user_id = $1 AND device_id = $2',
      [req.user!.id, deviceId]
    );

    res.json({ success: true });
  } catch (err) {
    console.error('Delete session error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'An error occurred',
      },
    });
  }
});

export default router;
