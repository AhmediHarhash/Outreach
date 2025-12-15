/**
 * CV Processing Routes
 * Profile management, CV parsing, tailoring, and generation
 */

import { Router, Response } from 'express';
import { z } from 'zod';
import { authMiddleware, AuthRequest } from '../middleware/auth.js';
import { aiGenerationLimiter } from '../middleware/rateLimiter.js';
import { isValidUUID } from '../utils/validation.js';
import cvService, { UserProfile, Skill, Experience, Education } from '../services/cv.js';

const router = Router();

// ============================================================================
// VALIDATION SCHEMAS
// ============================================================================

const skillSchema = z.object({
  name: z.string().min(1).max(100),
  level: z.enum(['beginner', 'intermediate', 'advanced', 'expert']).optional(),
  category: z.enum(['language', 'framework', 'tool', 'soft_skill', 'other']).optional(),
  yearsUsed: z.number().min(0).max(50).optional(),
});

const experienceSchema = z.object({
  company: z.string().min(1).max(200),
  title: z.string().min(1).max(200),
  startDate: z.string().max(20),
  endDate: z.string().max(20).optional(),
  current: z.boolean().optional(),
  location: z.string().max(200).optional(),
  description: z.string().max(2000).optional(),
  achievements: z.array(z.string().max(500)).max(10),
  technologies: z.array(z.string().max(50)).max(20).optional(),
});

const educationSchema = z.object({
  institution: z.string().min(1).max(200),
  degree: z.string().min(1).max(200),
  field: z.string().min(1).max(200),
  startDate: z.string().max(20).optional(),
  endDate: z.string().max(20),
  gpa: z.string().max(20).optional(),
  achievements: z.array(z.string().max(500)).max(5).optional(),
});

const profileSchema = z.object({
  fullName: z.string().min(1).max(200),
  email: z.string().email().max(255),
  phone: z.string().max(50).optional(),
  linkedin: z.string().max(255).optional(),
  github: z.string().max(255).optional(),
  portfolioUrl: z.string().max(500).optional(),
  location: z.string().max(200).optional(),
  headline: z.string().max(255).optional(),
  summary: z.string().max(2000).optional(),
  yearsExperience: z.number().min(0).max(50).optional(),
  skills: z.array(skillSchema).max(100).optional(),
  languages: z.array(z.object({
    language: z.string().max(50),
    proficiency: z.enum(['native', 'fluent', 'professional', 'conversational', 'basic']),
  })).max(20).optional(),
  certifications: z.array(z.object({
    name: z.string().max(200),
    issuer: z.string().max(200),
    date: z.string().max(20),
    url: z.string().max(500).optional(),
  })).max(20).optional(),
  experience: z.array(experienceSchema).max(20).optional(),
  education: z.array(educationSchema).max(10).optional(),
  projects: z.array(z.object({
    name: z.string().max(200),
    description: z.string().max(1000),
    url: z.string().max(500).optional(),
    technologies: z.array(z.string().max(50)).max(20),
    highlights: z.array(z.string().max(500)).max(5),
  })).max(20).optional(),
  desiredRoles: z.array(z.string().max(100)).max(10).optional(),
  desiredIndustries: z.array(z.string().max(100)).max(10).optional(),
  remotePreference: z.enum(['remote', 'hybrid', 'onsite']).optional(),
});

const tailorCVSchema = z.object({
  leadId: z.string().refine(isValidUUID, 'Invalid lead ID').optional(),
  jobTitle: z.string().max(200).optional(),
  jobDescription: z.string().max(10000).optional(),
  companyName: z.string().max(200).optional(),
  templateId: z.string().max(50).optional(),
});

const analyzeCVSchema = z.object({
  jobDescription: z.string().max(10000).optional(),
});

const generateHtmlSchema = z.object({
  templateId: z.string().max(50).optional(),
});

const parseResumeSchema = z.object({
  text: z.string().min(100).max(50000),
});

// ============================================================================
// PROFILE ROUTES
// ============================================================================

// GET /cv/profile - Get user's CV profile
router.get('/profile', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const profile = await cvService.getUserProfile(req.user!.id);

    if (!profile) {
      return res.status(404).json({
        error: {
          code: 'PROFILE_NOT_FOUND',
          message: 'No CV profile found. Please create one first.',
        },
      });
    }

    res.json(profile);
  } catch (err) {
    console.error('Get profile error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'Failed to get profile',
      },
    });
  }
});

// POST /cv/profile - Create or update CV profile
router.post('/profile', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const body = profileSchema.parse(req.body);

    const profileId = await cvService.saveUserProfile(req.user!.id, body as UserProfile);

    res.json({
      success: true,
      profileId,
      message: 'Profile saved successfully',
    });
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({
        error: {
          code: 'VALIDATION_ERROR',
          message: 'Invalid profile data',
          details: err.errors,
        },
      });
    }
    console.error('Save profile error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'Failed to save profile',
      },
    });
  }
});

// PATCH /cv/profile - Partial update CV profile
router.patch('/profile', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const body = profileSchema.partial().parse(req.body);

    if (Object.keys(body).length === 0) {
      return res.status(400).json({
        error: {
          code: 'EMPTY_UPDATE',
          message: 'No fields to update',
        },
      });
    }

    const profileId = await cvService.saveUserProfile(req.user!.id, body as Partial<UserProfile>);

    res.json({
      success: true,
      profileId,
      message: 'Profile updated successfully',
    });
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({
        error: {
          code: 'VALIDATION_ERROR',
          message: 'Invalid profile data',
          details: err.errors,
        },
      });
    }
    console.error('Update profile error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'Failed to update profile',
      },
    });
  }
});

// ============================================================================
// CV ANALYSIS ROUTES
// ============================================================================

// POST /cv/analyze - Analyze current profile for ATS optimization
router.post('/analyze', authMiddleware, aiGenerationLimiter, async (req: AuthRequest, res: Response) => {
  try {
    const body = analyzeCVSchema.parse(req.body);

    const profile = await cvService.getUserProfile(req.user!.id);
    if (!profile) {
      return res.status(404).json({
        error: {
          code: 'PROFILE_NOT_FOUND',
          message: 'Please create a CV profile first',
        },
      });
    }

    const analysis = await cvService.analyzeCV(profile, body.jobDescription);

    res.json(analysis);
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({
        error: {
          code: 'VALIDATION_ERROR',
          message: 'Invalid request',
          details: err.errors,
        },
      });
    }
    console.error('Analyze CV error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'Failed to analyze CV',
      },
    });
  }
});

// ============================================================================
// CV TAILORING ROUTES
// ============================================================================

// POST /cv/tailor - Tailor CV for a specific job/lead
router.post('/tailor', authMiddleware, aiGenerationLimiter, async (req: AuthRequest, res: Response) => {
  try {
    const body = tailorCVSchema.parse(req.body);

    // Get profile ID
    const profile = await cvService.getUserProfile(req.user!.id);
    if (!profile) {
      return res.status(404).json({
        error: {
          code: 'PROFILE_NOT_FOUND',
          message: 'Please create a CV profile first',
        },
      });
    }

    // Get profile record to get ID
    const { pool } = await import('../db.js');
    const profileResult = await pool.query(
      `SELECT id FROM user_profiles WHERE user_id = $1`,
      [req.user!.id]
    );

    if (profileResult.rows.length === 0) {
      return res.status(404).json({
        error: {
          code: 'PROFILE_NOT_FOUND',
          message: 'Profile not found',
        },
      });
    }

    const result = await cvService.tailorCV({
      userId: req.user!.id,
      profileId: profileResult.rows[0].id,
      leadId: body.leadId,
      jobTitle: body.jobTitle,
      jobDescription: body.jobDescription,
      companyName: body.companyName,
      templateId: body.templateId,
    });

    res.json({
      tailoredProfile: result.tailoredProfile,
      changes: result.changes,
      generationId: result.generationId,
    });
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({
        error: {
          code: 'VALIDATION_ERROR',
          message: 'Invalid request',
          details: err.errors,
        },
      });
    }
    console.error('Tailor CV error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'Failed to tailor CV',
      },
    });
  }
});

// ============================================================================
// CV GENERATION ROUTES
// ============================================================================

// POST /cv/generate-html - Generate HTML version of CV
router.post('/generate-html', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const body = generateHtmlSchema.parse(req.body);

    const profile = await cvService.getUserProfile(req.user!.id);
    if (!profile) {
      return res.status(404).json({
        error: {
          code: 'PROFILE_NOT_FOUND',
          message: 'Please create a CV profile first',
        },
      });
    }

    const html = await cvService.generateCVHtml(profile, body.templateId);

    res.json({ html });
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({
        error: {
          code: 'VALIDATION_ERROR',
          message: 'Invalid request',
          details: err.errors,
        },
      });
    }
    console.error('Generate HTML error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'Failed to generate CV HTML',
      },
    });
  }
});

// POST /cv/generate-html-custom - Generate HTML from custom profile data
router.post('/generate-html-custom', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const profileData = profileSchema.parse(req.body.profile);
    const templateId = req.body.templateId as string | undefined;

    const html = await cvService.generateCVHtml(profileData as UserProfile, templateId);

    res.json({ html });
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({
        error: {
          code: 'VALIDATION_ERROR',
          message: 'Invalid profile data',
          details: err.errors,
        },
      });
    }
    console.error('Generate custom HTML error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'Failed to generate CV HTML',
      },
    });
  }
});

// ============================================================================
// CV PARSING ROUTES
// ============================================================================

// POST /cv/parse-text - Parse resume text into structured data
router.post('/parse-text', authMiddleware, aiGenerationLimiter, async (req: AuthRequest, res: Response) => {
  try {
    const body = parseResumeSchema.parse(req.body);

    const parsed = await cvService.parseResumeText(body.text);

    res.json(parsed);
  } catch (err) {
    if (err instanceof z.ZodError) {
      return res.status(400).json({
        error: {
          code: 'VALIDATION_ERROR',
          message: 'Invalid request',
          details: err.errors,
        },
      });
    }
    console.error('Parse text error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'Failed to parse resume text',
      },
    });
  }
});

// ============================================================================
// TEMPLATE ROUTES
// ============================================================================

// GET /cv/templates - List available CV templates
router.get('/templates', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const category = req.query.category as string | undefined;
    const includePremium = req.query.includePremium === 'true';

    const templates = await cvService.getTemplates({ category, includePremium });

    res.json({ templates });
  } catch (err) {
    console.error('Get templates error:', err);
    res.status(500).json({
      error: {
        code: 'INTERNAL_ERROR',
        message: 'Failed to get templates',
      },
    });
  }
});

export default router;
