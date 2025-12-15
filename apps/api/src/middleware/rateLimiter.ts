/**
 * Rate Limiter Middleware
 * In-memory rate limiting for auth endpoints
 *
 * For production at scale, consider Redis-based rate limiting
 */

import { Request, Response, NextFunction } from 'express';

interface RateLimitEntry {
  count: number;
  resetTime: number;
}

interface RateLimitStore {
  [key: string]: RateLimitEntry;
}

interface RateLimitConfig {
  windowMs: number;      // Time window in milliseconds
  maxRequests: number;   // Max requests per window
  message?: string;      // Error message
  keyGenerator?: (req: Request) => string;
  skipSuccessfulRequests?: boolean;
  skipFailedRequests?: boolean;
}

// In-memory store (per-instance)
// For multi-instance deployments, use Redis
const stores: Map<string, RateLimitStore> = new Map();

// Cleanup old entries every 5 minutes
setInterval(() => {
  const now = Date.now();
  for (const [, store] of stores) {
    for (const key of Object.keys(store)) {
      if (store[key].resetTime < now) {
        delete store[key];
      }
    }
  }
}, 5 * 60 * 1000);

function getClientIdentifier(req: Request): string {
  // Use X-Forwarded-For for proxied requests (Railway, etc.)
  const forwarded = req.headers['x-forwarded-for'];
  if (forwarded) {
    const ips = Array.isArray(forwarded) ? forwarded[0] : forwarded.split(',')[0];
    return ips.trim();
  }
  return req.ip || req.socket.remoteAddress || 'unknown';
}

export function createRateLimiter(name: string, config: RateLimitConfig) {
  // Initialize store for this limiter
  if (!stores.has(name)) {
    stores.set(name, {});
  }

  const {
    windowMs,
    maxRequests,
    message = 'Too many requests, please try again later',
    keyGenerator = getClientIdentifier,
  } = config;

  return (req: Request, res: Response, next: NextFunction) => {
    const store = stores.get(name)!;
    const key = keyGenerator(req);
    const now = Date.now();

    // Get or create entry
    let entry = store[key];
    if (!entry || entry.resetTime < now) {
      entry = {
        count: 0,
        resetTime: now + windowMs,
      };
      store[key] = entry;
    }

    // Increment count
    entry.count++;

    // Set rate limit headers
    const remaining = Math.max(0, maxRequests - entry.count);
    const resetSeconds = Math.ceil((entry.resetTime - now) / 1000);

    res.setHeader('X-RateLimit-Limit', maxRequests);
    res.setHeader('X-RateLimit-Remaining', remaining);
    res.setHeader('X-RateLimit-Reset', Math.ceil(entry.resetTime / 1000));
    res.setHeader('Retry-After', resetSeconds);

    // Check if over limit
    if (entry.count > maxRequests) {
      return res.status(429).json({
        error: {
          code: 'RATE_LIMIT_EXCEEDED',
          message,
          retryAfter: resetSeconds,
        },
      });
    }

    next();
  };
}

// ============================================================================
// PRE-CONFIGURED LIMITERS
// ============================================================================

// Strict limiter for login attempts: 5 attempts per 15 minutes per IP
export const loginLimiter = createRateLimiter('login', {
  windowMs: 15 * 60 * 1000, // 15 minutes
  maxRequests: 5,
  message: 'Too many login attempts. Please try again in 15 minutes.',
});

// Registration limiter: 3 registrations per hour per IP
export const registerLimiter = createRateLimiter('register', {
  windowMs: 60 * 60 * 1000, // 1 hour
  maxRequests: 3,
  message: 'Too many registration attempts. Please try again later.',
});

// Password reset limiter: 3 requests per hour per IP
export const passwordResetLimiter = createRateLimiter('passwordReset', {
  windowMs: 60 * 60 * 1000, // 1 hour
  maxRequests: 3,
  message: 'Too many password reset requests. Please try again later.',
});

// Token refresh limiter: 30 refreshes per hour per IP
export const refreshLimiter = createRateLimiter('refresh', {
  windowMs: 60 * 60 * 1000, // 1 hour
  maxRequests: 30,
  message: 'Too many token refresh requests. Please try again later.',
});

// Email sending limiter: 50 emails per hour per user
export const emailSendLimiter = createRateLimiter('emailSend', {
  windowMs: 60 * 60 * 1000, // 1 hour
  maxRequests: 50,
  message: 'Email sending limit reached. Please try again later.',
});

// AI generation limiter: 20 generations per hour per user
export const aiGenerationLimiter = createRateLimiter('aiGeneration', {
  windowMs: 60 * 60 * 1000, // 1 hour
  maxRequests: 20,
  message: 'AI generation limit reached. Please try again later.',
});

// General API limiter: 100 requests per minute per IP
export const generalLimiter = createRateLimiter('general', {
  windowMs: 60 * 1000, // 1 minute
  maxRequests: 100,
  message: 'Too many requests. Please slow down.',
});

// Strict brute force protection: block IP after repeated failures
const failedAttempts: Map<string, { count: number; blockedUntil: number }> = new Map();

export function bruteForceProtection(req: Request, res: Response, next: NextFunction) {
  const ip = getClientIdentifier(req);
  const now = Date.now();
  const entry = failedAttempts.get(ip);

  // Check if IP is blocked
  if (entry && entry.blockedUntil > now) {
    const retryAfter = Math.ceil((entry.blockedUntil - now) / 1000);
    res.setHeader('Retry-After', retryAfter);
    return res.status(429).json({
      error: {
        code: 'IP_BLOCKED',
        message: 'Your IP has been temporarily blocked due to too many failed attempts.',
        retryAfter,
      },
    });
  }

  // Clean up expired blocks
  if (entry && entry.blockedUntil < now) {
    failedAttempts.delete(ip);
  }

  next();
}

// Call this after a failed login attempt
export function recordFailedAttempt(req: Request): void {
  const ip = getClientIdentifier(req);
  const entry = failedAttempts.get(ip) || { count: 0, blockedUntil: 0 };

  entry.count++;

  // Progressive blocking:
  // 5 failures = 5 min block
  // 10 failures = 30 min block
  // 15+ failures = 1 hour block
  if (entry.count >= 15) {
    entry.blockedUntil = Date.now() + 60 * 60 * 1000; // 1 hour
  } else if (entry.count >= 10) {
    entry.blockedUntil = Date.now() + 30 * 60 * 1000; // 30 minutes
  } else if (entry.count >= 5) {
    entry.blockedUntil = Date.now() + 5 * 60 * 1000; // 5 minutes
  }

  failedAttempts.set(ip, entry);
}

// Call this after a successful login to reset the counter
export function resetFailedAttempts(req: Request): void {
  const ip = getClientIdentifier(req);
  failedAttempts.delete(ip);
}
