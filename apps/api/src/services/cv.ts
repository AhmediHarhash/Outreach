/**
 * CV Processing Service
 * Handles CV parsing, analysis, generation, and tailoring
 */

import { pool } from '../db.js';
import { v4 as uuidv4 } from 'uuid';
import aiService from './ai.js';

// ============================================================================
// TYPES
// ============================================================================

export interface Skill {
  name: string;
  level?: 'beginner' | 'intermediate' | 'advanced' | 'expert';
  category?: 'language' | 'framework' | 'tool' | 'soft_skill' | 'other';
  yearsUsed?: number;
}

export interface Experience {
  company: string;
  title: string;
  startDate: string;
  endDate?: string;
  current?: boolean;
  location?: string;
  description?: string;
  achievements: string[];
  technologies?: string[];
}

export interface Education {
  institution: string;
  degree: string;
  field: string;
  startDate?: string;
  endDate: string;
  gpa?: string;
  achievements?: string[];
}

export interface Project {
  name: string;
  description: string;
  url?: string;
  technologies: string[];
  highlights: string[];
}

export interface UserProfile {
  fullName: string;
  email: string;
  phone?: string;
  linkedin?: string;
  github?: string;
  portfolioUrl?: string;
  location?: string;
  headline?: string;
  summary?: string;
  yearsExperience?: number;
  skills: Skill[];
  languages?: Array<{ language: string; proficiency: string }>;
  certifications?: Array<{ name: string; issuer: string; date: string; url?: string }>;
  experience: Experience[];
  education: Education[];
  projects?: Project[];
  desiredRoles?: string[];
  desiredIndustries?: string[];
  remotePreference?: string;
}

export interface ATSIssue {
  issue: string;
  severity: 'high' | 'medium' | 'low';
  suggestion: string;
  section?: string;
}

export interface CVAnalysis {
  atsScore: number;
  issues: ATSIssue[];
  suggestions: Array<{
    section: string;
    suggestion: string;
    priority: 'high' | 'medium' | 'low';
  }>;
  keywordMatch?: {
    matched: string[];
    missing: string[];
    score: number;
  };
}

export interface TailorCVParams {
  userId: string;
  profileId: string;
  leadId?: string;
  jobTitle?: string;
  jobDescription?: string;
  companyName?: string;
  templateId?: string;
}

// ============================================================================
// CV PARSING
// ============================================================================

/**
 * Extract text from a document (PDF/DOCX)
 * In production, use pdf-parse or mammoth libraries
 */
export async function extractTextFromDocument(
  fileBuffer: Buffer,
  fileType: string
): Promise<string> {
  // For now, return placeholder - in production, use:
  // - pdf-parse for PDFs
  // - mammoth for DOCX
  // These require native dependencies that need to be added to package.json

  if (fileType === 'pdf') {
    // const pdfParse = await import('pdf-parse');
    // const data = await pdfParse.default(fileBuffer);
    // return data.text;
    throw new Error('PDF parsing requires pdf-parse package. Install with: npm install pdf-parse');
  }

  if (fileType === 'docx') {
    // const mammoth = await import('mammoth');
    // const result = await mammoth.extractRawText({ buffer: fileBuffer });
    // return result.value;
    throw new Error('DOCX parsing requires mammoth package. Install with: npm install mammoth');
  }

  throw new Error(`Unsupported file type: ${fileType}`);
}

/**
 * Parse extracted text into structured CV data using AI
 */
export async function parseResumeText(text: string): Promise<UserProfile> {
  const prompt = `Parse this resume/CV text into structured JSON format. Extract all available information.

RESUME TEXT:
${text.slice(0, 8000)}

Respond with ONLY valid JSON in this exact format:
{
  "fullName": "string",
  "email": "string",
  "phone": "string or null",
  "linkedin": "string or null",
  "github": "string or null",
  "portfolioUrl": "string or null",
  "location": "string or null",
  "headline": "short professional headline or null",
  "summary": "professional summary paragraph or null",
  "yearsExperience": number or null,
  "skills": [{"name": "string", "level": "beginner|intermediate|advanced|expert", "category": "language|framework|tool|soft_skill|other"}],
  "languages": [{"language": "string", "proficiency": "native|fluent|professional|conversational|basic"}],
  "certifications": [{"name": "string", "issuer": "string", "date": "YYYY or YYYY-MM"}],
  "experience": [
    {
      "company": "string",
      "title": "string",
      "startDate": "YYYY-MM or YYYY",
      "endDate": "YYYY-MM or YYYY or null if current",
      "current": boolean,
      "location": "string or null",
      "achievements": ["achievement with metrics where possible"]
    }
  ],
  "education": [
    {
      "institution": "string",
      "degree": "string",
      "field": "string",
      "endDate": "YYYY",
      "gpa": "string or null"
    }
  ],
  "projects": [{"name": "string", "description": "string", "technologies": ["tech"], "highlights": ["highlight"]}]
}

Important:
- Extract ALL information available
- For achievements, convert bullet points to action-oriented statements with metrics where possible
- Categorize skills appropriately
- If information is not available, use null
- Ensure dates are formatted consistently`;

  const response = await aiService.callClaude(prompt);

  // Parse JSON from response
  let jsonStr = response;
  if (response.includes('```')) {
    const match = response.match(/```(?:json)?\s*([\s\S]*?)```/);
    jsonStr = match ? match[1] : response;
  }

  try {
    const parsed = JSON.parse(jsonStr.trim());
    return parsed as UserProfile;
  } catch (error) {
    console.error('Failed to parse CV:', error);
    throw new Error('Failed to parse resume text');
  }
}

// ============================================================================
// ATS ANALYSIS
// ============================================================================

/**
 * Analyze CV for ATS compatibility and optimization opportunities
 */
export async function analyzeCV(
  profile: UserProfile,
  jobDescription?: string
): Promise<CVAnalysis> {
  const prompt = `Analyze this CV/resume for ATS (Applicant Tracking System) compatibility and provide improvement suggestions.

CV DATA:
${JSON.stringify(profile, null, 2)}

${jobDescription ? `TARGET JOB DESCRIPTION:\n${jobDescription.slice(0, 3000)}` : ''}

Provide analysis in this JSON format:
{
  "atsScore": <0-100 score>,
  "issues": [
    {
      "issue": "description of the issue",
      "severity": "high|medium|low",
      "suggestion": "how to fix it",
      "section": "which part of CV is affected"
    }
  ],
  "suggestions": [
    {
      "section": "experience|summary|skills|education|format",
      "suggestion": "specific improvement suggestion",
      "priority": "high|medium|low"
    }
  ]${jobDescription ? `,
  "keywordMatch": {
    "matched": ["keywords from job description found in CV"],
    "missing": ["important keywords from job description NOT in CV"],
    "score": <0-100 keyword match percentage>
  }` : ''}
}

ATS Scoring Criteria:
- Clear section headings (10 points)
- Consistent date formats (10 points)
- No tables/graphics that confuse parsers (10 points)
- Standard fonts and formatting (10 points)
- Keywords match job requirements (20 points)
- Quantified achievements (15 points)
- Complete contact information (5 points)
- Appropriate length (5 points)
- Action verbs in experience (10 points)
- No spelling/grammar errors (5 points)

Respond with ONLY valid JSON.`;

  const response = await aiService.callClaude(prompt);

  let jsonStr = response;
  if (response.includes('```')) {
    const match = response.match(/```(?:json)?\s*([\s\S]*?)```/);
    jsonStr = match ? match[1] : response;
  }

  try {
    return JSON.parse(jsonStr.trim()) as CVAnalysis;
  } catch (error) {
    console.error('Failed to parse CV analysis:', error);
    return {
      atsScore: 50,
      issues: [{
        issue: 'Could not analyze CV',
        severity: 'medium',
        suggestion: 'Please try again',
      }],
      suggestions: [],
    };
  }
}

// ============================================================================
// CV TAILORING
// ============================================================================

/**
 * Tailor a CV for a specific job/lead using AI
 */
export async function tailorCV(params: TailorCVParams): Promise<{
  tailoredProfile: UserProfile;
  changes: string[];
  generationId: string;
}> {
  // Get user profile
  const profileResult = await pool.query(
    `SELECT * FROM user_profiles WHERE id = $1 AND user_id = $2`,
    [params.profileId, params.userId]
  );

  if (profileResult.rows.length === 0) {
    throw new Error('Profile not found');
  }

  const profile = profileResult.rows[0];
  const baseProfile: UserProfile = {
    fullName: profile.full_name || '',
    email: profile.email || '',
    phone: profile.phone,
    linkedin: profile.linkedin,
    github: profile.github,
    portfolioUrl: profile.portfolio_url,
    location: profile.location,
    headline: profile.headline,
    summary: profile.summary,
    yearsExperience: profile.years_experience,
    skills: profile.skills || [],
    languages: profile.languages || [],
    certifications: profile.certifications || [],
    experience: profile.experience || [],
    education: profile.education || [],
    projects: profile.projects || [],
  };

  // Get lead info if provided
  let leadInfo = null;
  if (params.leadId) {
    const leadResult = await pool.query(
      `SELECT company_name, industry, tech_stack, contact_title, contact_name
       FROM leads WHERE id = $1 AND user_id = $2`,
      [params.leadId, params.userId]
    );
    if (leadResult.rows.length > 0) {
      leadInfo = leadResult.rows[0];
    }
  }

  const prompt = `Tailor this CV for a specific job opportunity. Rewrite and optimize the content to maximize relevance.

ORIGINAL CV:
${JSON.stringify(baseProfile, null, 2)}

TARGET:
- Job Title: ${params.jobTitle || leadInfo?.contact_title || 'Not specified'}
- Company: ${params.companyName || leadInfo?.company_name || 'Not specified'}
- Industry: ${leadInfo?.industry || 'Not specified'}
${leadInfo?.tech_stack ? `- Tech Stack: ${JSON.stringify(leadInfo.tech_stack)}` : ''}
${params.jobDescription ? `\nJOB DESCRIPTION:\n${params.jobDescription.slice(0, 3000)}` : ''}

Instructions:
1. Rewrite the summary to highlight relevant experience for this role
2. Reorder and emphasize relevant skills
3. Highlight achievements that align with the target role
4. Adjust headline to match the opportunity
5. Keep the same structure but optimize content

Respond with JSON:
{
  "tailoredProfile": <the modified profile in same format as input>,
  "changes": ["list of specific changes made"]
}`;

  const response = await aiService.callClaude(prompt);

  let jsonStr = response;
  if (response.includes('```')) {
    const match = response.match(/```(?:json)?\s*([\s\S]*?)```/);
    jsonStr = match ? match[1] : response;
  }

  const result = JSON.parse(jsonStr.trim());

  // Save generation record
  const generationId = uuidv4();
  await pool.query(
    `INSERT INTO cv_generations (id, user_id, lead_id, job_title, job_description, company_name, customizations)
     VALUES ($1, $2, $3, $4, $5, $6, $7)`,
    [
      generationId,
      params.userId,
      params.leadId,
      params.jobTitle,
      params.jobDescription?.slice(0, 10000),
      params.companyName,
      JSON.stringify({ changes: result.changes }),
    ]
  );

  return {
    tailoredProfile: result.tailoredProfile,
    changes: result.changes,
    generationId,
  };
}

// ============================================================================
// CV GENERATION (HTML -> PDF ready)
// ============================================================================

/**
 * Generate HTML CV from profile and template
 */
export async function generateCVHtml(
  profile: UserProfile,
  templateId: string = 'modern_minimal'
): Promise<string> {
  // Get template
  const templateResult = await pool.query(
    `SELECT template_html FROM cv_templates WHERE id = $1 AND is_active = true`,
    [templateId]
  );

  if (templateResult.rows.length === 0) {
    throw new Error('Template not found');
  }

  let html = templateResult.rows[0].template_html;

  // Simple Handlebars-like template replacement
  // In production, use actual Handlebars library for full support

  // Replace simple variables
  html = html.replace(/\{\{(\w+)\}\}/g, (_match: string, key: string) => {
    const value = (profile as any)[key];
    return value !== undefined && value !== null ? String(value) : '';
  });

  // Handle conditionals: {{#if variable}}content{{/if}}
  html = html.replace(/\{\{#if (\w+)\}\}([\s\S]*?)\{\{\/if\}\}/g, (_match: string, key: string, content: string) => {
    const value = (profile as any)[key];
    if (value && (Array.isArray(value) ? value.length > 0 : true)) {
      return content;
    }
    return '';
  });

  // Handle arrays: {{#each array}}content{{/each}}
  // This is simplified - real implementation should use Handlebars
  const arrayPattern = /\{\{#each (\w+)\}\}([\s\S]*?)\{\{\/each\}\}/g;
  html = html.replace(arrayPattern, (_match: string, arrayKey: string, template: string) => {
    const items = (profile as any)[arrayKey];
    if (!Array.isArray(items) || items.length === 0) {
      return '';
    }

    return items.map((item: any) => {
      let itemHtml = template;
      // Replace item properties
      Object.keys(item).forEach(key => {
        const regex = new RegExp(`\\{\\{${key}\\}\\}`, 'g');
        itemHtml = itemHtml.replace(regex, item[key] || '');
      });
      // Handle {{this}} for simple arrays
      itemHtml = itemHtml.replace(/\{\{this\}\}/g, typeof item === 'string' ? item : '');
      return itemHtml;
    }).join('');
  });

  return html;
}

// ============================================================================
// USER PROFILE MANAGEMENT
// ============================================================================

/**
 * Save or update user profile
 */
export async function saveUserProfile(
  userId: string,
  profile: Partial<UserProfile>
): Promise<string> {
  // Check if profile exists
  const existing = await pool.query(
    `SELECT id FROM user_profiles WHERE user_id = $1`,
    [userId]
  );

  if (existing.rows.length > 0) {
    // Update
    await pool.query(
      `UPDATE user_profiles SET
        full_name = COALESCE($2, full_name),
        email = COALESCE($3, email),
        phone = COALESCE($4, phone),
        linkedin = COALESCE($5, linkedin),
        github = COALESCE($6, github),
        portfolio_url = COALESCE($7, portfolio_url),
        location = COALESCE($8, location),
        headline = COALESCE($9, headline),
        summary = COALESCE($10, summary),
        years_experience = COALESCE($11, years_experience),
        skills = COALESCE($12, skills),
        languages = COALESCE($13, languages),
        certifications = COALESCE($14, certifications),
        experience = COALESCE($15, experience),
        education = COALESCE($16, education),
        projects = COALESCE($17, projects),
        desired_roles = COALESCE($18, desired_roles),
        desired_industries = COALESCE($19, desired_industries),
        remote_preference = COALESCE($20, remote_preference),
        updated_at = NOW()
      WHERE user_id = $1`,
      [
        userId,
        profile.fullName,
        profile.email,
        profile.phone,
        profile.linkedin,
        profile.github,
        profile.portfolioUrl,
        profile.location,
        profile.headline,
        profile.summary,
        profile.yearsExperience,
        profile.skills ? JSON.stringify(profile.skills) : null,
        profile.languages ? JSON.stringify(profile.languages) : null,
        profile.certifications ? JSON.stringify(profile.certifications) : null,
        profile.experience ? JSON.stringify(profile.experience) : null,
        profile.education ? JSON.stringify(profile.education) : null,
        profile.projects ? JSON.stringify(profile.projects) : null,
        profile.desiredRoles ? JSON.stringify(profile.desiredRoles) : null,
        profile.desiredIndustries ? JSON.stringify(profile.desiredIndustries) : null,
        profile.remotePreference,
      ]
    );
    return existing.rows[0].id;
  } else {
    // Insert
    const result = await pool.query(
      `INSERT INTO user_profiles (
        user_id, full_name, email, phone, linkedin, github, portfolio_url,
        location, headline, summary, years_experience, skills, languages,
        certifications, experience, education, projects, desired_roles,
        desired_industries, remote_preference
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
      RETURNING id`,
      [
        userId,
        profile.fullName,
        profile.email,
        profile.phone,
        profile.linkedin,
        profile.github,
        profile.portfolioUrl,
        profile.location,
        profile.headline,
        profile.summary,
        profile.yearsExperience,
        JSON.stringify(profile.skills || []),
        JSON.stringify(profile.languages || []),
        JSON.stringify(profile.certifications || []),
        JSON.stringify(profile.experience || []),
        JSON.stringify(profile.education || []),
        JSON.stringify(profile.projects || []),
        JSON.stringify(profile.desiredRoles || []),
        JSON.stringify(profile.desiredIndustries || []),
        profile.remotePreference,
      ]
    );
    return result.rows[0].id;
  }
}

/**
 * Get user profile
 */
export async function getUserProfile(userId: string): Promise<UserProfile | null> {
  const result = await pool.query(
    `SELECT * FROM user_profiles WHERE user_id = $1`,
    [userId]
  );

  if (result.rows.length === 0) {
    return null;
  }

  const row = result.rows[0];
  return {
    fullName: row.full_name,
    email: row.email,
    phone: row.phone,
    linkedin: row.linkedin,
    github: row.github,
    portfolioUrl: row.portfolio_url,
    location: row.location,
    headline: row.headline,
    summary: row.summary,
    yearsExperience: row.years_experience,
    skills: row.skills || [],
    languages: row.languages || [],
    certifications: row.certifications || [],
    experience: row.experience || [],
    education: row.education || [],
    projects: row.projects || [],
    desiredRoles: row.desired_roles || [],
    desiredIndustries: row.desired_industries || [],
    remotePreference: row.remote_preference,
  };
}

/**
 * Get CV templates
 */
export async function getTemplates(options?: { category?: string; includePremium?: boolean }) {
  let query = `SELECT id, name, description, category, preview_image_url, is_premium
               FROM cv_templates WHERE is_active = true`;
  const params: any[] = [];

  if (options?.category) {
    params.push(options.category);
    query += ` AND category = $${params.length}`;
  }

  if (!options?.includePremium) {
    query += ` AND is_premium = false`;
  }

  query += ` ORDER BY sort_order ASC`;

  const result = await pool.query(query, params);
  return result.rows;
}

export default {
  extractTextFromDocument,
  parseResumeText,
  analyzeCV,
  tailorCV,
  generateCVHtml,
  saveUserProfile,
  getUserProfile,
  getTemplates,
};
