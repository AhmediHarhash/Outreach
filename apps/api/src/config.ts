import dotenv from 'dotenv';
dotenv.config();

export const config = {
  // Server
  host: process.env.HOST || '0.0.0.0',
  port: parseInt(process.env.PORT || '3001', 10),
  environment: process.env.ENVIRONMENT || 'development',

  // Database
  databaseUrl: process.env.DATABASE_URL || '',

  // JWT
  jwtSecret: process.env.JWT_SECRET || '',
  jwtAccessExpirySecs: parseInt(process.env.JWT_ACCESS_EXPIRY_SECS || '900', 10),
  jwtRefreshExpiryDays: parseInt(process.env.JWT_REFRESH_EXPIRY_DAYS || '30', 10),
  encryptionKey: process.env.ENCRYPTION_KEY || '',

  // AI
  openaiApiKey: process.env.OPENAI_API_KEY || '',

  // OAuth
  googleClientId: process.env.GOOGLE_CLIENT_ID || '',
  googleClientSecret: process.env.GOOGLE_CLIENT_SECRET || '',
  googleRedirectUri: process.env.GOOGLE_REDIRECT_URI || '',

  // CORS & Frontend
  allowedOrigins: (process.env.ALLOWED_ORIGINS || '').split(',').filter(Boolean),
  webAppUrl: process.env.WEB_APP_URL || '',
  desktopScheme: process.env.DESKTOP_SCHEME || 'hekax',

  // AWS SES
  awsRegion: process.env.AWS_REGION || 'us-east-1',
  awsAccessKeyId: process.env.AWS_ACCESS_KEY_ID || '',
  awsSecretAccessKey: process.env.AWS_SECRET_ACCESS_KEY || '',
  sesFromEmail: process.env.SES_FROM_EMAIL || '',
  sesFromName: process.env.SES_FROM_NAME || 'Outreach',
};

// Validate required config
export function validateConfig() {
  const required = [
    'databaseUrl',
    'jwtSecret',
  ] as const;

  const missing = required.filter(key => !config[key]);
  if (missing.length > 0) {
    throw new Error(`Missing required environment variables: ${missing.join(', ')}`);
  }
}
