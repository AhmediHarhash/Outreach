/**
 * Authentication Middleware
 * JWT verification with issuer/audience validation
 */

import { Request, Response, NextFunction } from 'express';
import jwt from 'jsonwebtoken';
import { config } from '../config.js';

// JWT Configuration - must match auth.ts
const JWT_ISSUER = 'outreach-api';
const JWT_AUDIENCE = 'outreach-app';

export interface AuthUser {
  id: string;
  email: string;
  subscriptionTier: string;
  tokenVersion: number;
}

export interface AuthRequest extends Request {
  user?: AuthUser;
}

interface JWTPayload {
  sub: string;           // User ID (standard claim)
  email: string;
  subscriptionTier: string;
  tokenVersion: number;
  iat: number;
  exp: number;
  iss: string;
  aud: string;
}

export function authMiddleware(req: AuthRequest, res: Response, next: NextFunction) {
  const authHeader = req.headers.authorization;

  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    return res.status(401).json({
      error: {
        code: 'MISSING_AUTH_HEADER',
        message: 'Missing or invalid authorization header',
      },
    });
  }

  const token = authHeader.slice(7);

  // Basic token format validation
  if (!token || token.split('.').length !== 3) {
    return res.status(401).json({
      error: {
        code: 'INVALID_TOKEN_FORMAT',
        message: 'Invalid token format',
      },
    });
  }

  try {
    const payload = jwt.verify(token, config.jwtSecret, {
      issuer: JWT_ISSUER,
      audience: JWT_AUDIENCE,
      algorithms: ['HS256'],
    }) as JWTPayload;

    // Map payload to AuthUser
    req.user = {
      id: payload.sub,
      email: payload.email,
      subscriptionTier: payload.subscriptionTier,
      tokenVersion: payload.tokenVersion,
    };

    next();
  } catch (err) {
    if (err instanceof jwt.TokenExpiredError) {
      return res.status(401).json({
        error: {
          code: 'TOKEN_EXPIRED',
          message: 'Token has expired',
        },
      });
    }
    if (err instanceof jwt.JsonWebTokenError) {
      return res.status(401).json({
        error: {
          code: 'INVALID_TOKEN',
          message: 'Invalid token',
        },
      });
    }
    return res.status(401).json({
      error: {
        code: 'AUTH_ERROR',
        message: 'Authentication failed',
      },
    });
  }
}

export function optionalAuth(req: AuthRequest, res: Response, next: NextFunction) {
  const authHeader = req.headers.authorization;

  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    return next();
  }

  const token = authHeader.slice(7);

  if (!token || token.split('.').length !== 3) {
    return next();
  }

  try {
    const payload = jwt.verify(token, config.jwtSecret, {
      issuer: JWT_ISSUER,
      audience: JWT_AUDIENCE,
      algorithms: ['HS256'],
    }) as JWTPayload;

    req.user = {
      id: payload.sub,
      email: payload.email,
      subscriptionTier: payload.subscriptionTier,
      tokenVersion: payload.tokenVersion,
    };
  } catch {
    // Ignore errors for optional auth - just don't set user
  }

  next();
}

// Middleware to require specific subscription tier
export function requireTier(...allowedTiers: string[]) {
  return (req: AuthRequest, res: Response, next: NextFunction) => {
    if (!req.user) {
      return res.status(401).json({
        error: {
          code: 'UNAUTHORIZED',
          message: 'Authentication required',
        },
      });
    }

    if (!allowedTiers.includes(req.user.subscriptionTier)) {
      return res.status(403).json({
        error: {
          code: 'INSUFFICIENT_TIER',
          message: `This feature requires one of: ${allowedTiers.join(', ')}`,
        },
      });
    }

    next();
  };
}
