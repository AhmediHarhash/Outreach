/**
 * Outreach API Server
 * Production-ready with security hardening
 */

import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import morgan from 'morgan';
import { config, validateConfig } from './config.js';
import { testConnection } from './db.js';
import { generalLimiter } from './middleware/rateLimiter.js';

// Routes
import healthRouter from './routes/health.js';
import authRouter from './routes/auth.js';
import usersRouter from './routes/users.js';
import leadsRouter from './routes/leads.js';
import recordingsRouter from './routes/recordings.js';
import emailsRouter from './routes/emails.js';
import templatesRouter from './routes/templates.js';
import trackingRouter from './routes/tracking.js';
import cvRouter from './routes/cv.js';

async function main() {
  // Validate config
  validateConfig();

  // Test database connection
  const dbConnected = await testConnection();
  if (!dbConnected) {
    console.error('Failed to connect to database. Exiting...');
    process.exit(1);
  }

  const app = express();

  // Trust proxy for rate limiting behind Railway/load balancer
  app.set('trust proxy', 1);

  // ============================================================================
  // SECURITY MIDDLEWARE
  // ============================================================================

  // Helmet for security headers
  app.use(helmet({
    contentSecurityPolicy: {
      directives: {
        defaultSrc: ["'self'"],
        styleSrc: ["'self'", "'unsafe-inline'"],
        imgSrc: ["'self'", 'data:', 'https:'],
        scriptSrc: ["'self'"],
      },
    },
    crossOriginEmbedderPolicy: false, // Allow embedding for tracking pixel
    hsts: {
      maxAge: 31536000, // 1 year
      includeSubDomains: true,
      preload: true,
    },
  }));

  // CORS - Strict in production, configurable in development
  const corsOptions: cors.CorsOptions = {
    origin: (origin, callback) => {
      // Allow requests with no origin (mobile apps, Postman, etc.)
      if (!origin) {
        return callback(null, true);
      }

      // In production, strictly check against allowed origins
      if (config.environment === 'production') {
        if (config.allowedOrigins.includes(origin)) {
          callback(null, true);
        } else {
          console.warn(`CORS blocked origin: ${origin}`);
          callback(new Error('Not allowed by CORS'));
        }
      } else {
        // In development, allow localhost variants
        const devOrigins = [
          /^http:\/\/localhost(:\d+)?$/,
          /^http:\/\/127\.0\.0\.1(:\d+)?$/,
          /^http:\/\/\[::1\](:\d+)?$/,
        ];
        const isDevOrigin = devOrigins.some(pattern => pattern.test(origin));

        if (isDevOrigin || config.allowedOrigins.includes(origin)) {
          callback(null, true);
        } else {
          callback(new Error('Not allowed by CORS'));
        }
      }
    },
    credentials: true,
    methods: ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'OPTIONS'],
    allowedHeaders: ['Content-Type', 'Authorization', 'X-Requested-With'],
    exposedHeaders: ['X-RateLimit-Limit', 'X-RateLimit-Remaining', 'X-RateLimit-Reset', 'Retry-After'],
    maxAge: 86400, // 24 hours
  };
  app.use(cors(corsOptions));

  // Body parsing with size limits
  app.use(express.json({ limit: '10mb' }));
  app.use(express.urlencoded({ extended: true, limit: '10mb' }));

  // Request logging
  app.use(morgan(config.environment === 'production' ? 'combined' : 'dev'));

  // General rate limiting (applies to all routes)
  app.use(generalLimiter);

  // ============================================================================
  // ROUTES
  // ============================================================================

  // Health check (no auth, no rate limit beyond general)
  app.use('/health', healthRouter);

  // Authentication (has its own specific rate limiters)
  app.use('/auth', authRouter);

  // Protected routes
  app.use('/users', usersRouter);
  app.use('/leads', leadsRouter);
  app.use('/recordings', recordingsRouter);
  app.use('/emails', emailsRouter);
  app.use('/templates', templatesRouter);
  app.use('/cv', cvRouter);

  // Tracking (public, no auth - but has HMAC verification)
  app.use('/track', trackingRouter);

  // ============================================================================
  // ERROR HANDLING
  // ============================================================================

  // Global error handler
  app.use((err: Error, req: express.Request, res: express.Response, _next: express.NextFunction) => {
    // Log error details
    console.error('Unhandled error:', {
      message: err.message,
      stack: config.environment === 'development' ? err.stack : undefined,
      path: req.path,
      method: req.method,
    });

    // Don't leak error details in production
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: config.environment === 'production'
          ? 'An unexpected error occurred'
          : err.message,
      },
    });
  });

  // 404 handler
  app.use((req, res) => {
    res.status(404).json({
      error: {
        code: 'NOT_FOUND',
        message: `Route ${req.method} ${req.path} not found`,
      },
    });
  });

  // ============================================================================
  // SERVER START
  // ============================================================================

  app.listen(config.port, config.host, () => {
    console.log(`Outreach API running on http://${config.host}:${config.port}`);
    console.log(`Environment: ${config.environment}`);
    console.log(`Allowed origins: ${config.allowedOrigins.join(', ') || '(none configured)'}`);
  });
}

main().catch((err) => {
  console.error('Failed to start server:', err);
  process.exit(1);
});
