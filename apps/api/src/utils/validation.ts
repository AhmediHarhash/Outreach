/**
 * Validation Utilities
 * Common validation functions for input sanitization
 */

import { z } from 'zod';

// UUID v4 regex pattern
const UUID_REGEX = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

// More permissive UUID pattern (accepts any version)
const UUID_ANY_REGEX = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

/**
 * Validate UUID format (any version)
 */
export function isValidUUID(id: string): boolean {
  return UUID_ANY_REGEX.test(id);
}

/**
 * Validate UUID v4 specifically
 */
export function isValidUUIDv4(id: string): boolean {
  return UUID_REGEX.test(id);
}

/**
 * Zod schema for UUID
 */
export const uuidSchema = z.string().regex(UUID_ANY_REGEX, 'Invalid UUID format');

/**
 * Sanitize string input - removes control characters and trims
 */
export function sanitizeString(input: string): string {
  // Remove control characters except newlines and tabs
  // eslint-disable-next-line no-control-regex
  return input.replace(/[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]/g, '').trim();
}

/**
 * Validate email format
 */
export function isValidEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

/**
 * Sanitize HTML - basic XSS prevention
 * For more robust sanitization, use DOMPurify on the client side
 */
export function sanitizeHtml(html: string): string {
  // Remove script tags and their contents
  let sanitized = html.replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '');

  // Remove on* event handlers
  sanitized = sanitized.replace(/\s*on\w+\s*=\s*"[^"]*"/gi, '');
  sanitized = sanitized.replace(/\s*on\w+\s*=\s*'[^']*'/gi, '');
  sanitized = sanitized.replace(/\s*on\w+\s*=[^\s>]*/gi, '');

  // Remove javascript: URLs
  sanitized = sanitized.replace(/javascript:/gi, '');

  // Remove data: URLs in href (potential XSS vector)
  sanitized = sanitized.replace(/href\s*=\s*"data:/gi, 'href="');
  sanitized = sanitized.replace(/href\s*=\s*'data:/gi, "href='");

  return sanitized;
}

/**
 * Validate and sanitize template variables
 */
export function extractTemplateVariables(template: string): string[] {
  const matches = template.match(/\{\{(\w+)\}\}/g) || [];
  return [...new Set(matches.map(v => v.replace(/[{}]/g, '')))];
}

/**
 * Rate limit key generators
 */
export function getUserRateLimitKey(userId: string, action: string): string {
  return `user:${userId}:${action}`;
}

export function getIpRateLimitKey(ip: string, action: string): string {
  return `ip:${ip}:${action}`;
}

/**
 * Allowed template generation parameters
 */
export const ALLOWED_PURPOSES = [
  'cold_outreach',
  'follow_up',
  'cv_submission',
  'meeting_request',
  'thank_you',
  'introduction',
  'proposal',
  'reminder',
] as const;

export const ALLOWED_TONES = [
  'formal',
  'professional',
  'casual',
  'friendly',
  'enthusiastic',
  'urgent',
] as const;

export type Purpose = typeof ALLOWED_PURPOSES[number];
export type Tone = typeof ALLOWED_TONES[number];
