/**
 * Email Service - AWS SES Integration
 * Handles all email sending, tracking, and management
 */

import { SESClient, SendEmailCommand, SendRawEmailCommand } from '@aws-sdk/client-ses';
import { config } from '../config.js';
import { pool } from '../db.js';
import { v4 as uuidv4 } from 'uuid';
import crypto from 'crypto';

// Initialize SES Client
const ses = new SESClient({
  region: config.awsRegion,
  credentials: {
    accessKeyId: config.awsAccessKeyId,
    secretAccessKey: config.awsSecretAccessKey,
  },
});

// Types
export interface EmailOptions {
  to: string;
  subject: string;
  html: string;
  text?: string;
  replyTo?: string;
  attachments?: Array<{
    filename: string;
    content: Buffer;
    contentType: string;
  }>;
}

export interface SendEmailParams {
  userId: string;
  leadId: string;
  templateId?: string;
  subject: string;
  bodyHtml: string;
  bodyText?: string;
  purpose: string;
  attachments?: Array<{
    filename: string;
    type: string;
    url: string;
  }>;
}

export interface BulkEmailParams {
  userId: string;
  leadIds: string[];
  templateId: string;
  subject: string;
  bodyHtml: string;
  bodyText?: string;
  purpose: string;
  personalizePerLead?: boolean;
}

// Generate tracking pixel URL
export function generateTrackingPixel(emailId: string): string {
  const trackingId = crypto.createHash('sha256').update(emailId + config.jwtSecret).digest('hex').slice(0, 16);
  return `${config.webAppUrl}/api/track/open/${emailId}/${trackingId}`;
}

// Generate tracked link
export function generateTrackedLink(emailId: string, originalUrl: string, linkIndex: number): string {
  const linkId = crypto.createHash('sha256').update(`${emailId}:${linkIndex}:${config.jwtSecret}`).digest('hex').slice(0, 16);
  // Store the original URL mapping
  return `${config.webAppUrl}/api/track/click/${emailId}/${linkId}?url=${encodeURIComponent(originalUrl)}`;
}

// Inject tracking into HTML
export function injectTracking(emailId: string, html: string): string {
  // Add tracking pixel before </body>
  const trackingPixel = `<img src="${generateTrackingPixel(emailId)}" width="1" height="1" style="display:none" alt="" />`;

  let trackedHtml = html;

  // Replace links with tracked versions
  let linkIndex = 0;
  trackedHtml = trackedHtml.replace(/href="(https?:\/\/[^"]+)"/g, (match, url) => {
    // Don't track unsubscribe links
    if (url.includes('unsubscribe')) return match;
    const trackedUrl = generateTrackedLink(emailId, url, linkIndex++);
    return `href="${trackedUrl}"`;
  });

  // Inject pixel
  if (trackedHtml.includes('</body>')) {
    trackedHtml = trackedHtml.replace('</body>', `${trackingPixel}</body>`);
  } else {
    trackedHtml += trackingPixel;
  }

  return trackedHtml;
}

// Add unsubscribe link
export function addUnsubscribeLink(emailId: string, html: string): string {
  const unsubscribeUrl = `${config.webAppUrl}/unsubscribe/${emailId}`;
  const unsubscribeHtml = `
    <div style="margin-top: 40px; padding-top: 20px; border-top: 1px solid #eee; text-align: center; font-size: 12px; color: #666;">
      <p>You're receiving this because you were added as a lead.</p>
      <p><a href="${unsubscribeUrl}" style="color: #666;">Unsubscribe</a></p>
    </div>
  `;

  if (html.includes('</body>')) {
    return html.replace('</body>', `${unsubscribeHtml}</body>`);
  }
  return html + unsubscribeHtml;
}

// Send single email via SES
export async function sendEmail(options: EmailOptions): Promise<string> {
  const command = new SendEmailCommand({
    Source: `${config.sesFromName} <${config.sesFromEmail}>`,
    Destination: {
      ToAddresses: [options.to],
    },
    Message: {
      Subject: {
        Data: options.subject,
        Charset: 'UTF-8',
      },
      Body: {
        Html: {
          Data: options.html,
          Charset: 'UTF-8',
        },
        ...(options.text && {
          Text: {
            Data: options.text,
            Charset: 'UTF-8',
          },
        }),
      },
    },
    ReplyToAddresses: options.replyTo ? [options.replyTo] : undefined,
  });

  const response = await ses.send(command);
  return response.MessageId || '';
}

// Send and track email
export async function sendTrackedEmail(params: SendEmailParams): Promise<{ emailId: string; messageId: string }> {
  const emailId = uuidv4();

  // Get lead info for personalization context
  const leadResult = await pool.query(
    'SELECT email, company_name, contact_name FROM leads WHERE id = $1 AND user_id = $2',
    [params.leadId, params.userId]
  );

  if (leadResult.rows.length === 0) {
    throw new Error('Lead not found');
  }

  const lead = leadResult.rows[0];

  // Inject tracking
  let trackedHtml = injectTracking(emailId, params.bodyHtml);
  trackedHtml = addUnsubscribeLink(emailId, trackedHtml);

  // Calculate expiry (2 years from now)
  const expiresAt = new Date();
  expiresAt.setFullYear(expiresAt.getFullYear() + 2);

  // Store email in database
  await pool.query(
    `INSERT INTO email_log (
      id, user_id, to_email, from_email, subject, template_id, status, metadata
    ) VALUES ($1, $2, $3, $4, $5, $6, 'queued', $7)`,
    [
      emailId,
      params.userId,
      lead.email,
      config.sesFromEmail,
      params.subject,
      params.templateId || null,
      JSON.stringify({
        leadId: params.leadId,
        purpose: params.purpose,
        bodyHtml: params.bodyHtml,
        bodyText: params.bodyText,
        attachments: params.attachments,
        expiresAt: expiresAt.toISOString(),
      }),
    ]
  );

  try {
    // Send via SES
    const messageId = await sendEmail({
      to: lead.email,
      subject: params.subject,
      html: trackedHtml,
      text: params.bodyText,
    });

    // Update with SES message ID and sent status
    await pool.query(
      `UPDATE email_log SET
        ses_message_id = $2,
        status = 'sent',
        sent_at = NOW()
      WHERE id = $1`,
      [emailId, messageId]
    );

    // Update lead's last contacted timestamp
    await pool.query(
      `UPDATE leads SET last_contacted_at = NOW() WHERE id = $1`,
      [params.leadId]
    );

    // Log activity
    await pool.query(
      `INSERT INTO activity_log (user_id, activity_type, entity_type, entity_id, metadata)
       VALUES ($1, 'email_sent', 'email', $2, $3)`,
      [params.userId, emailId, JSON.stringify({ leadId: params.leadId, subject: params.subject })]
    );

    return { emailId, messageId };
  } catch (error) {
    // Update status to failed
    await pool.query(
      `UPDATE email_log SET status = 'failed', error_message = $2 WHERE id = $1`,
      [emailId, (error as Error).message]
    );
    throw error;
  }
}

// Send bulk emails
export async function sendBulkEmails(params: BulkEmailParams): Promise<{
  queued: number;
  failed: number;
  emailIds: string[]
}> {
  const results = {
    queued: 0,
    failed: 0,
    emailIds: [] as string[],
  };

  // Create a job for bulk sending
  const jobId = uuidv4();
  await pool.query(
    `INSERT INTO jobs (id, user_id, job_type, status, input_params)
     VALUES ($1, $2, 'bulk_email', 'pending', $3)`,
    [jobId, params.userId, JSON.stringify(params)]
  );

  // Process each lead
  for (const leadId of params.leadIds) {
    try {
      const { emailId } = await sendTrackedEmail({
        userId: params.userId,
        leadId,
        templateId: params.templateId,
        subject: params.subject,
        bodyHtml: params.bodyHtml,
        bodyText: params.bodyText,
        purpose: params.purpose,
      });

      results.queued++;
      results.emailIds.push(emailId);

      // Small delay between sends to avoid rate limiting
      await new Promise(resolve => setTimeout(resolve, 100));
    } catch (error) {
      console.error(`Failed to send email to lead ${leadId}:`, error);
      results.failed++;
    }
  }

  // Update job status
  await pool.query(
    `UPDATE jobs SET status = 'completed', result = $2, completed_at = NOW() WHERE id = $1`,
    [jobId, JSON.stringify(results)]
  );

  return results;
}

// Record email open
export async function recordEmailOpen(emailId: string): Promise<void> {
  await pool.query(
    `UPDATE email_log SET
      status = CASE WHEN status = 'sent' OR status = 'delivered' THEN 'opened' ELSE status END,
      opened_at = COALESCE(opened_at, NOW())
    WHERE id = $1`,
    [emailId]
  );
}

// Record link click
export async function recordLinkClick(emailId: string): Promise<void> {
  await pool.query(
    `UPDATE email_log SET
      status = CASE WHEN status IN ('sent', 'delivered', 'opened') THEN 'clicked' ELSE status END,
      clicked_at = COALESCE(clicked_at, NOW())
    WHERE id = $1`,
    [emailId]
  );
}

// Mark email as replied
export async function markEmailReplied(emailId: string, userId: string): Promise<void> {
  // Update email status
  await pool.query(
    `UPDATE email_log SET
      status = 'replied',
      delivered_at = COALESCE(delivered_at, NOW())
    WHERE id = $1 AND user_id = $2`,
    [emailId, userId]
  );

  // Get lead ID from email metadata
  const emailResult = await pool.query(
    `SELECT metadata FROM email_log WHERE id = $1`,
    [emailId]
  );

  if (emailResult.rows.length > 0) {
    const metadata = emailResult.rows[0].metadata;
    if (metadata?.leadId) {
      // Update lead status
      await pool.query(
        `UPDATE leads SET status = 'replied' WHERE id = $1 AND status IN ('new', 'contacted')`,
        [metadata.leadId]
      );
    }
  }
}

// Get email with full details
export async function getEmailWithAnalysis(emailId: string, userId: string): Promise<any> {
  const result = await pool.query(
    `SELECT
      e.*,
      l.id as lead_id,
      l.company_name,
      l.contact_name,
      l.contact_title,
      l.contact_email,
      l.contact_linkedin,
      l.industry,
      l.company_size,
      l.status as lead_status
    FROM email_log e
    LEFT JOIN leads l ON (e.metadata->>'leadId')::uuid = l.id
    WHERE e.id = $1 AND e.user_id = $2`,
    [emailId, userId]
  );

  if (result.rows.length === 0) {
    return null;
  }

  return result.rows[0];
}

// Cleanup old unreplied emails (run as cron job)
export async function cleanupOldEmails(): Promise<number> {
  const twoYearsAgo = new Date();
  twoYearsAgo.setFullYear(twoYearsAgo.getFullYear() - 2);

  const result = await pool.query(
    `DELETE FROM email_log
     WHERE status NOT IN ('replied')
     AND sent_at < $1
     RETURNING id`,
    [twoYearsAgo]
  );

  return result.rowCount || 0;
}

export default {
  sendEmail,
  sendTrackedEmail,
  sendBulkEmails,
  recordEmailOpen,
  recordLinkClick,
  markEmailReplied,
  getEmailWithAnalysis,
  cleanupOldEmails,
};
