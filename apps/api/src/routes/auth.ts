import { Router, Response } from 'express';
import bcrypt from 'bcryptjs';
import jwt from 'jsonwebtoken';
import crypto from 'crypto';
import { z } from 'zod';
import { v4 as uuidv4 } from 'uuid';
import { pool } from '../db.js';
import { config } from '../config.js';
import { authMiddleware, AuthRequest } from '../middleware/auth.js';

const router = Router();

// Validation schemas
const registerSchema = z.object({
  email: z.string().email(),
  password: z.string().min(8),
  fullName: z.string().min(1).optional(),
});

const loginSchema = z.object({
  email: z.string().email(),
  password: z.string(),
  deviceId: z.string().default(() => uuidv4()),
  deviceName: z.string().optional(),
});

const refreshSchema = z.object({
  refreshToken: z.string(),
  deviceId: z.string(),
});

// Helper functions
function createAccessToken(user: { id: string; email: string; subscription_tier: string; token_version: number }) {
  return jwt.sign(
    {
      id: user.id,
      email: user.email,
      subscriptionTier: user.subscription_tier,
      tokenVersion: user.token_version,
    },
    config.jwtSecret,
    { expiresIn: config.jwtAccessExpirySecs }
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

// POST /auth/register
router.post('/register', async (req, res) => {
  try {
    const body = registerSchema.parse(req.body);

    // Check if user exists
    const existing = await pool.query('SELECT id FROM users WHERE email = $1', [body.email.toLowerCase()]);
    if (existing.rows.length > 0) {
      return res.status(409).json({ error: 'User already exists' });
    }

    // Hash password
    const passwordHash = await bcrypt.hash(body.password, 12);

    // Create user
    const userResult = await pool.query(
      `INSERT INTO users (email, password_hash, full_name)
       VALUES ($1, $2, $3)
       RETURNING *`,
      [body.email.toLowerCase(), passwordHash, body.fullName || null]
    );
    const user = userResult.rows[0];

    // Create default settings
    await pool.query('INSERT INTO user_settings (user_id) VALUES ($1)', [user.id]);

    // Generate device ID
    const deviceId = uuidv4();

    // Create tokens
    const accessToken = createAccessToken(user);
    const refreshToken = await createRefreshToken(user.id, deviceId, 'Web Registration', user.token_version);

    // Log activity
    await pool.query(
      `INSERT INTO activity_log (user_id, activity_type, metadata) VALUES ($1, 'register', '{}')`,
      [user.id]
    );

    res.status(201).json({
      user: {
        id: user.id,
        email: user.email,
        fullName: user.full_name,
        subscriptionTier: user.subscription_tier,
        emailVerified: user.email_verified,
        locale: user.locale,
        timezone: user.timezone,
        createdAt: user.created_at,
      },
      accessToken,
      refreshToken,
      expiresIn: config.jwtAccessExpirySecs,
    });
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({ error: 'Validation error', details: err.errors });
    }
    console.error('Register error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// POST /auth/login
router.post('/login', async (req, res) => {
  try {
    const body = loginSchema.parse(req.body);

    // Find user
    const userResult = await pool.query('SELECT * FROM users WHERE email = $1', [body.email.toLowerCase()]);
    if (userResult.rows.length === 0) {
      return res.status(401).json({ error: 'Invalid credentials' });
    }
    const user = userResult.rows[0];

    // Verify password
    if (!user.password_hash || !(await bcrypt.compare(body.password, user.password_hash))) {
      return res.status(401).json({ error: 'Invalid credentials' });
    }

    // Create tokens
    const accessToken = createAccessToken(user);
    const refreshToken = await createRefreshToken(user.id, body.deviceId, body.deviceName || null, user.token_version);

    // Log activity
    await pool.query(
      `INSERT INTO activity_log (user_id, activity_type, metadata) VALUES ($1, 'login', $2)`,
      [user.id, JSON.stringify({ device_id: body.deviceId })]
    );

    res.json({
      user: {
        id: user.id,
        email: user.email,
        fullName: user.full_name,
        subscriptionTier: user.subscription_tier,
        emailVerified: user.email_verified,
        locale: user.locale,
        timezone: user.timezone,
        createdAt: user.created_at,
      },
      accessToken,
      refreshToken,
      expiresIn: config.jwtAccessExpirySecs,
    });
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({ error: 'Validation error', details: err.errors });
    }
    console.error('Login error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// POST /auth/refresh
router.post('/refresh', async (req, res) => {
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
      return res.status(401).json({ error: 'Invalid or expired refresh token' });
    }

    const storedToken = tokenResult.rows[0];

    // Check token version matches
    if (storedToken.token_version !== storedToken.user_token_version) {
      return res.status(401).json({ error: 'Token has been revoked' });
    }

    // Get user
    const userResult = await pool.query('SELECT * FROM users WHERE id = $1', [storedToken.user_id]);
    const user = userResult.rows[0];

    // Rotate refresh token
    const newRefreshToken = await createRefreshToken(user.id, body.deviceId, storedToken.device_name, user.token_version);

    // Create new access token
    const accessToken = createAccessToken(user);

    res.json({
      user: {
        id: user.id,
        email: user.email,
        fullName: user.full_name,
        subscriptionTier: user.subscription_tier,
        emailVerified: user.email_verified,
        locale: user.locale,
        timezone: user.timezone,
        createdAt: user.created_at,
      },
      accessToken,
      refreshToken: newRefreshToken,
      expiresIn: config.jwtAccessExpirySecs,
    });
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({ error: 'Validation error', details: err.errors });
    }
    console.error('Refresh error:', err);
    res.status(500).json({ error: 'Internal server error' });
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
    res.status(500).json({ error: 'Internal server error' });
  }
});

// POST /auth/logout-all
router.post('/logout-all', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    // Increment token version to invalidate all refresh tokens
    await pool.query('UPDATE users SET token_version = token_version + 1 WHERE id = $1', [req.user!.id]);

    // Log activity
    await pool.query(
      `INSERT INTO activity_log (user_id, activity_type) VALUES ($1, 'logout_all')`,
      [req.user!.id]
    );

    res.json({ success: true });
  } catch (err) {
    console.error('Logout all error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// GET /auth/me
router.get('/me', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const userResult = await pool.query('SELECT * FROM users WHERE id = $1', [req.user!.id]);
    if (userResult.rows.length === 0) {
      return res.status(404).json({ error: 'User not found' });
    }
    const user = userResult.rows[0];

    res.json({
      id: user.id,
      email: user.email,
      fullName: user.full_name,
      avatarUrl: user.avatar_url,
      subscriptionTier: user.subscription_tier,
      subscriptionExpiresAt: user.subscription_expires_at,
      emailVerified: user.email_verified,
      locale: user.locale,
      timezone: user.timezone,
      dataRegion: user.data_region,
      createdAt: user.created_at,
      updatedAt: user.updated_at,
    });
  } catch (err) {
    console.error('Get me error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

export default router;
