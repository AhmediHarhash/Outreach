/**
 * Tracking Routes
 * Handles email open/click tracking with security hardening
 */

import { Router, Response, Request } from 'express';
import crypto from 'crypto';
import { pool } from '../db.js';
import { config } from '../config.js';
import emailService from '../services/email.js';

const router = Router();

// 1x1 transparent GIF
const TRACKING_PIXEL = Buffer.from(
  'R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7',
  'base64'
);

// ============================================================================
// SECURITY HELPERS
// ============================================================================

/**
 * Generate a secure tracking ID using HMAC
 * This prevents fake tracking requests
 */
export function generateTrackingId(emailId: string): string {
  return crypto
    .createHmac('sha256', config.jwtSecret)
    .update(emailId)
    .digest('hex')
    .slice(0, 16);
}

/**
 * Verify tracking ID is valid for given email
 */
function verifyTrackingId(emailId: string, trackingId: string): boolean {
  const expected = generateTrackingId(emailId);
  try {
    return crypto.timingSafeEqual(
      Buffer.from(trackingId),
      Buffer.from(expected)
    );
  } catch {
    return false;
  }
}

/**
 * Validate redirect URL to prevent open redirect attacks
 * Only allows URLs that:
 * 1. Are valid URLs
 * 2. Use HTTPS (or HTTP for localhost in dev)
 * 3. Are not javascript: or data: URLs
 */
function isValidRedirectUrl(url: string): boolean {
  try {
    const parsed = new URL(url);

    // Block dangerous protocols
    if (!['http:', 'https:'].includes(parsed.protocol)) {
      return false;
    }

    // In production, only allow HTTPS
    if (config.environment === 'production' && parsed.protocol !== 'https:') {
      return false;
    }

    // Block localhost in production
    if (config.environment === 'production') {
      const hostname = parsed.hostname.toLowerCase();
      if (hostname === 'localhost' || hostname === '127.0.0.1' || hostname.startsWith('192.168.')) {
        return false;
      }
    }

    return true;
  } catch {
    return false;
  }
}

/**
 * Verify SNS message signature (simplified - checks required fields)
 * For full verification, use @aws-sdk/client-sns MessageValidator
 */
function isValidSnsMessage(body: any): boolean {
  // Check required SNS fields are present
  const requiredFields = ['Type', 'MessageId', 'TopicArn', 'Timestamp'];

  for (const field of requiredFields) {
    if (!body[field]) {
      return false;
    }
  }

  // Verify TopicArn matches our expected ARN pattern
  if (body.TopicArn && !body.TopicArn.includes('ses-notifications')) {
    // Be lenient during setup, just log warning
    console.warn('SNS TopicArn mismatch:', body.TopicArn);
  }

  return true;
}

// ============================================================================
// ROUTES
// ============================================================================

// GET /track/open/:emailId/:trackingId - Track email open
router.get('/open/:emailId/:trackingId', async (req: Request, res: Response) => {
  try {
    const { emailId, trackingId } = req.params;

    // Validate UUID format
    const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
    if (!uuidRegex.test(emailId)) {
      // Return pixel anyway to not break email rendering
      res.set('Content-Type', 'image/gif');
      return res.send(TRACKING_PIXEL);
    }

    // Verify tracking ID to prevent fake opens
    if (!verifyTrackingId(emailId, trackingId)) {
      console.warn('Invalid tracking ID for email:', emailId);
      // Return pixel anyway
      res.set('Content-Type', 'image/gif');
      return res.send(TRACKING_PIXEL);
    }

    // Record the open
    await emailService.recordEmailOpen(emailId);

    // Return transparent 1x1 GIF with anti-cache headers
    res.set({
      'Content-Type': 'image/gif',
      'Content-Length': String(TRACKING_PIXEL.length),
      'Cache-Control': 'no-store, no-cache, must-revalidate, private',
      'Pragma': 'no-cache',
      'Expires': '0',
      'X-Content-Type-Options': 'nosniff',
    });

    res.send(TRACKING_PIXEL);
  } catch (err) {
    console.error('Track open error:', err);
    // Still return pixel even on error to not break email rendering
    res.set('Content-Type', 'image/gif');
    res.send(TRACKING_PIXEL);
  }
});

// GET /track/click/:emailId/:linkId - Track link click and redirect
router.get('/click/:emailId/:linkId', async (req: Request, res: Response) => {
  try {
    const { emailId, linkId } = req.params;
    const { url } = req.query;

    // Validate URL parameter exists and is a string
    if (!url || typeof url !== 'string') {
      return res.status(400).json({ error: 'Missing redirect URL' });
    }

    // SECURITY: Validate redirect URL to prevent open redirect attacks
    if (!isValidRedirectUrl(url)) {
      console.warn('Blocked invalid redirect URL:', url);
      return res.status(400).json({ error: 'Invalid redirect URL' });
    }

    // Validate UUID format
    const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
    if (!uuidRegex.test(emailId)) {
      // Still redirect but don't track
      return res.redirect(302, url);
    }

    // Verify link ID (using same HMAC approach)
    if (!verifyTrackingId(emailId, linkId)) {
      console.warn('Invalid link ID for email:', emailId);
      // Still redirect but don't track
      return res.redirect(302, url);
    }

    // Record the click
    await emailService.recordLinkClick(emailId);

    // Redirect to original URL
    res.redirect(302, url);
  } catch (err) {
    console.error('Track click error:', err);
    // Try to redirect anyway if URL is valid
    const { url } = req.query;
    if (url && typeof url === 'string' && isValidRedirectUrl(url)) {
      res.redirect(302, url);
    } else {
      res.status(500).json({ error: 'Tracking error' });
    }
  }
});

// POST /webhooks/ses - AWS SES webhook for bounces/complaints
router.post('/webhooks/ses', async (req: Request, res: Response) => {
  try {
    const body = req.body;

    // SECURITY: Validate SNS message structure
    if (!isValidSnsMessage(body)) {
      console.warn('Invalid SNS message structure');
      return res.status(400).send('Invalid message');
    }

    // Handle SNS subscription confirmation
    if (body.Type === 'SubscriptionConfirmation') {
      // Validate SubscribeURL is from AWS
      if (body.SubscribeURL && body.SubscribeURL.includes('.amazonaws.com/')) {
        await fetch(body.SubscribeURL);
        console.log('SNS subscription confirmed');
      } else {
        console.warn('Invalid SubscribeURL:', body.SubscribeURL);
      }
      return res.status(200).send('OK');
    }

    // Handle notifications
    if (body.Type === 'Notification') {
      let message;
      try {
        message = JSON.parse(body.Message);
      } catch (parseErr) {
        console.error('Failed to parse SNS message:', parseErr);
        return res.status(400).send('Invalid message format');
      }

      if (message.notificationType === 'Bounce') {
        const bounce = message.bounce;
        const recipients = bounce?.bouncedRecipients || [];

        for (const recipient of recipients) {
          if (!recipient.emailAddress) continue;

          // Update email status
          await pool.query(
            `UPDATE email_log SET
              status = 'bounced',
              bounce_type = $2,
              bounce_subtype = $3
            WHERE to_email = $1 AND status != 'bounced'`,
            [recipient.emailAddress, bounce.bounceType || 'Unknown', bounce.bounceSubType || 'Unknown']
          );

          // Add to suppression list
          await pool.query(
            `INSERT INTO email_suppression (email, reason, details)
             VALUES ($1, 'bounce', $2)
             ON CONFLICT (email) DO NOTHING`,
            [recipient.emailAddress, JSON.stringify(bounce)]
          );
        }
      }

      if (message.notificationType === 'Complaint') {
        const complaint = message.complaint;
        const recipients = complaint?.complainedRecipients || [];

        for (const recipient of recipients) {
          if (!recipient.emailAddress) continue;

          // Update email status
          await pool.query(
            `UPDATE email_log SET
              status = 'complained',
              complaint_type = $2
            WHERE to_email = $1`,
            [recipient.emailAddress, complaint.complaintFeedbackType || 'Unknown']
          );

          // Add to suppression list
          await pool.query(
            `INSERT INTO email_suppression (email, reason, details)
             VALUES ($1, 'complaint', $2)
             ON CONFLICT (email) DO NOTHING`,
            [recipient.emailAddress, JSON.stringify(complaint)]
          );
        }
      }

      if (message.notificationType === 'Delivery') {
        const delivery = message.delivery;
        const recipients = delivery?.recipients || [];

        for (const email of recipients) {
          if (!email || typeof email !== 'string') continue;

          await pool.query(
            `UPDATE email_log SET
              status = CASE WHEN status = 'sent' THEN 'delivered' ELSE status END,
              delivered_at = COALESCE(delivered_at, NOW())
            WHERE to_email = $1 AND ses_message_id = $2`,
            [email, message.mail?.messageId]
          );
        }
      }
    }

    res.status(200).send('OK');
  } catch (err) {
    console.error('SES webhook error:', err);
    // Return 200 to prevent SNS retry storms
    res.status(200).send('Error logged');
  }
});

export default router;
