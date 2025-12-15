/**
 * Tracking Routes
 * Handles email open/click tracking
 */

import { Router, Response, Request } from 'express';
import { pool } from '../db.js';
import emailService from '../services/email.js';

const router = Router();

// 1x1 transparent GIF
const TRACKING_PIXEL = Buffer.from(
  'R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7',
  'base64'
);

// GET /track/open/:emailId/:trackingId - Track email open
router.get('/open/:emailId/:trackingId', async (req: Request, res: Response) => {
  try {
    const { emailId } = req.params;

    // Record the open
    await emailService.recordEmailOpen(emailId);

    // Return transparent 1x1 GIF
    res.set({
      'Content-Type': 'image/gif',
      'Content-Length': TRACKING_PIXEL.length,
      'Cache-Control': 'no-store, no-cache, must-revalidate, private',
      'Pragma': 'no-cache',
      'Expires': '0',
    });

    res.send(TRACKING_PIXEL);
  } catch (err) {
    console.error('Track open error:', err);
    // Still return pixel even on error
    res.set('Content-Type', 'image/gif');
    res.send(TRACKING_PIXEL);
  }
});

// GET /track/click/:emailId/:linkId - Track link click and redirect
router.get('/click/:emailId/:linkId', async (req: Request, res: Response) => {
  try {
    const { emailId } = req.params;
    const { url } = req.query;

    if (!url || typeof url !== 'string') {
      return res.status(400).send('Invalid redirect URL');
    }

    // Record the click
    await emailService.recordLinkClick(emailId);

    // Redirect to original URL
    res.redirect(302, url);
  } catch (err) {
    console.error('Track click error:', err);
    // Try to redirect anyway
    const { url } = req.query;
    if (url && typeof url === 'string') {
      res.redirect(302, url);
    } else {
      res.status(500).send('Tracking error');
    }
  }
});

// POST /webhooks/ses - AWS SES webhook for bounces/complaints
router.post('/webhooks/ses', async (req: Request, res: Response) => {
  try {
    const body = req.body;

    // Handle SNS subscription confirmation
    if (body.Type === 'SubscriptionConfirmation') {
      // Auto-confirm by fetching the SubscribeURL
      if (body.SubscribeURL) {
        await fetch(body.SubscribeURL);
      }
      return res.status(200).send('OK');
    }

    // Handle notifications
    if (body.Type === 'Notification') {
      const message = JSON.parse(body.Message);

      if (message.notificationType === 'Bounce') {
        const bounce = message.bounce;
        const recipients = bounce.bouncedRecipients || [];

        for (const recipient of recipients) {
          // Update email status
          await pool.query(
            `UPDATE email_log SET
              status = 'bounced',
              bounce_type = $2,
              bounce_subtype = $3
            WHERE to_email = $1 AND status != 'bounced'`,
            [recipient.emailAddress, bounce.bounceType, bounce.bounceSubType]
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
        const recipients = complaint.complainedRecipients || [];

        for (const recipient of recipients) {
          // Update email status
          await pool.query(
            `UPDATE email_log SET
              status = 'complained',
              complaint_type = $2
            WHERE to_email = $1`,
            [recipient.emailAddress, complaint.complaintFeedbackType]
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
        const recipients = delivery.recipients || [];

        for (const email of recipients) {
          await pool.query(
            `UPDATE email_log SET
              status = CASE WHEN status = 'sent' THEN 'delivered' ELSE status END,
              delivered_at = COALESCE(delivered_at, NOW())
            WHERE to_email = $1 AND ses_message_id = $2`,
            [email, message.mail.messageId]
          );
        }
      }
    }

    res.status(200).send('OK');
  } catch (err) {
    console.error('SES webhook error:', err);
    res.status(500).send('Error processing webhook');
  }
});

export default router;
