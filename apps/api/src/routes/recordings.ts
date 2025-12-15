import { Router, Response } from 'express';
import { z } from 'zod';
import { pool } from '../db.js';
import { authMiddleware, AuthRequest } from '../middleware/auth.js';

const router = Router();

// Validation schemas
const createRecordingSchema = z.object({
  leadId: z.string().uuid().optional(),
  mode: z.string(),
  startTime: z.string().datetime(),
  transcriptTurns: z.any().optional(),
});

const uploadRecordingSchema = z.object({
  transcriptTurns: z.any().optional(),
  endTime: z.string().datetime(),
  durationSeconds: z.number(),
  talkRatio: z.number().optional(),
  userWordCount: z.number().optional(),
  otherWordCount: z.number().optional(),
  userWpm: z.number().optional(),
});

const listQuerySchema = z.object({
  page: z.coerce.number().min(1).default(1),
  perPage: z.coerce.number().min(1).max(100).default(20),
  leadId: z.string().uuid().optional(),
  mode: z.string().optional(),
  status: z.string().optional(),
  fromDate: z.string().datetime().optional(),
  toDate: z.string().datetime().optional(),
});

// GET /recordings
router.get('/', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const query = listQuerySchema.parse(req.query);
    const offset = (query.page - 1) * query.perPage;

    const result = await pool.query(
      `SELECT
        r.id, r.lead_id, r.mode, r.status, r.start_time,
        r.duration_seconds, r.summary, r.outcome, r.sentiment_score,
        l.company_name as lead_name
       FROM recordings r
       LEFT JOIN leads l ON l.id = r.lead_id
       WHERE r.user_id = $1
         AND ($2::uuid IS NULL OR r.lead_id = $2)
         AND ($3::text IS NULL OR r.mode = $3)
         AND ($4::text IS NULL OR r.status = $4)
         AND ($5::timestamptz IS NULL OR r.start_time >= $5)
         AND ($6::timestamptz IS NULL OR r.start_time <= $6)
       ORDER BY r.start_time DESC
       LIMIT $7 OFFSET $8`,
      [
        req.user!.id,
        query.leadId || null,
        query.mode || null,
        query.status || null,
        query.fromDate ? new Date(query.fromDate) : null,
        query.toDate ? new Date(query.toDate) : null,
        query.perPage,
        offset
      ]
    );

    // Get total count
    const countResult = await pool.query(
      `SELECT COUNT(*) FROM recordings
       WHERE user_id = $1
         AND ($2::uuid IS NULL OR lead_id = $2)
         AND ($3::text IS NULL OR mode = $3)
         AND ($4::text IS NULL OR status = $4)`,
      [req.user!.id, query.leadId || null, query.mode || null, query.status || null]
    );

    const total = parseInt(countResult.rows[0].count, 10);

    res.json({
      recordings: result.rows.map(row => ({
        id: row.id,
        leadId: row.lead_id,
        leadName: row.lead_name,
        mode: row.mode,
        status: row.status,
        startTime: row.start_time,
        durationSeconds: row.duration_seconds,
        summary: row.summary,
        outcome: row.outcome,
        sentimentScore: row.sentiment_score,
      })),
      total,
      page: query.page,
      perPage: query.perPage,
    });
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({ error: 'Validation error', details: err.errors });
    }
    console.error('List recordings error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// POST /recordings
router.post('/', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const body = createRecordingSchema.parse(req.body);

    const result = await pool.query(
      `INSERT INTO recordings (user_id, lead_id, mode, status, start_time, transcript_turns)
       VALUES ($1, $2, $3, 'recording', $4, $5)
       RETURNING *`,
      [req.user!.id, body.leadId, body.mode, new Date(body.startTime), body.transcriptTurns]
    );

    const recording = result.rows[0];

    // Log activity
    await pool.query(
      `INSERT INTO activity_log (user_id, activity_type, entity_type, entity_id, metadata)
       VALUES ($1, 'call_started', 'recording', $2, $3)`,
      [req.user!.id, recording.id, JSON.stringify({ mode: body.mode, lead_id: body.leadId })]
    );

    res.status(201).json(formatRecording(recording));
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({ error: 'Validation error', details: err.errors });
    }
    console.error('Create recording error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// GET /recordings/:id
router.get('/:id', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const result = await pool.query(
      'SELECT * FROM recordings WHERE id = $1 AND user_id = $2',
      [req.params.id, req.user!.id]
    );

    if (result.rows.length === 0) {
      return res.status(404).json({ error: 'Recording not found' });
    }

    res.json(formatRecording(result.rows[0]));
  } catch (err) {
    console.error('Get recording error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// POST /recordings/:id/upload
router.post('/:id/upload', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const body = uploadRecordingSchema.parse(req.body);

    const result = await pool.query(
      `UPDATE recordings SET
        status = 'processing',
        transcript_turns = $3,
        end_time = $4,
        duration_seconds = $5,
        talk_ratio = $6,
        user_word_count = $7,
        other_word_count = $8,
        user_wpm = $9
       WHERE id = $1 AND user_id = $2
       RETURNING *`,
      [
        req.params.id, req.user!.id,
        body.transcriptTurns,
        new Date(body.endTime),
        body.durationSeconds,
        body.talkRatio,
        body.userWordCount,
        body.otherWordCount,
        body.userWpm
      ]
    );

    if (result.rows.length === 0) {
      return res.status(404).json({ error: 'Recording not found' });
    }

    const recording = result.rows[0];

    // Queue summary generation job
    await pool.query(
      `INSERT INTO jobs (user_id, job_type, input_params)
       VALUES ($1, 'generate_summary', $2)`,
      [req.user!.id, JSON.stringify({ recording_id: req.params.id })]
    );

    // Log activity
    await pool.query(
      `INSERT INTO activity_log (user_id, activity_type, entity_type, entity_id, metadata)
       VALUES ($1, 'call_ended', 'recording', $2, $3)`,
      [req.user!.id, req.params.id, JSON.stringify({ duration_seconds: body.durationSeconds })]
    );

    res.json(formatRecording(recording));
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({ error: 'Validation error', details: err.errors });
    }
    console.error('Upload recording error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// GET /recordings/:id/presigned-url
router.get('/:id/presigned-url', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    // Verify ownership
    const result = await pool.query(
      'SELECT id FROM recordings WHERE id = $1 AND user_id = $2',
      [req.params.id, req.user!.id]
    );

    if (result.rows.length === 0) {
      return res.status(404).json({ error: 'Recording not found' });
    }

    // Generate R2 key
    const r2Key = `recordings/${req.user!.id}/${req.params.id}.webm`;

    // TODO: Generate presigned URL using AWS SDK
    const r2Url = process.env.R2_PUBLIC_URL || 'https://storage.hekax.com';

    res.json({
      uploadUrl: `${r2Url}/${r2Key}`,
      r2Key,
      expiresIn: 3600,
    });
  } catch (err) {
    console.error('Get presigned URL error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// Helper function to format recording response
function formatRecording(row: any) {
  return {
    id: row.id,
    userId: row.user_id,
    leadId: row.lead_id,
    mode: row.mode,
    status: row.status,
    startTime: row.start_time,
    endTime: row.end_time,
    durationSeconds: row.duration_seconds,
    transcriptTurns: row.transcript_turns,
    summary: row.summary,
    keyPoints: row.key_points,
    actionItems: row.action_items,
    talkRatio: row.talk_ratio,
    userWordCount: row.user_word_count,
    otherWordCount: row.other_word_count,
    userWpm: row.user_wpm,
    questionCount: row.question_count,
    objectionCount: row.objection_count,
    sentimentScore: row.sentiment_score,
    performanceScore: row.performance_score,
    outcome: row.outcome,
    audioR2Key: row.audio_r2_key,
    transcriptR2Key: row.transcript_r2_key,
    createdAt: row.created_at,
  };
}

export default router;
