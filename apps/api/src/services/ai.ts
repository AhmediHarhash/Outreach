/**
 * AI Service - Claude + OpenAI Integration
 * Handles all AI-powered features: content generation, analysis, personalization
 */

import { config } from '../config.js';
import { pool } from '../db.js';

// Types
export interface LeadAnalysis {
  matchScore: number;
  pros: Array<{ point: string; reasoning: string }>;
  cons: Array<{ point: string; mitigation: string }>;
  opportunities: Array<{ description: string; howToLeverage: string; potentialValue: string }>;
  skillsMatch: Array<{ skill: string; userLevel: string; requiredLevel: string; gap: boolean }>;
  nextSteps: string[];
  talkingPoints: string[];
  questionsToAsk: string[];
  summary: string;
}

export interface EmailGenerationParams {
  leadId: string;
  userId: string;
  purpose: 'cold_outreach' | 'follow_up' | 'cv_submission' | 'meeting_request' | 'thank_you';
  tone: 'formal' | 'professional' | 'casual' | 'friendly';
  includeCV?: boolean;
  customInstructions?: string;
}

export interface CVAnalysis {
  strengths: string[];
  weaknesses: string[];
  suggestions: Array<{ section: string; suggestion: string; priority: 'high' | 'medium' | 'low' }>;
  skillsExtracted: string[];
  experienceYears: number;
  overallScore: number;
}

export interface LearningRecommendation {
  skill: string;
  reason: string;
  priority: 'high' | 'medium' | 'low';
  resources: Array<{
    title: string;
    type: 'video' | 'course' | 'article' | 'book';
    platform: string;
    url: string;
    isFree: boolean;
    duration?: string;
  }>;
}

// Claude API call
async function callClaude(prompt: string, systemPrompt?: string): Promise<string> {
  const response = await fetch('https://api.anthropic.com/v1/messages', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'x-api-key': config.anthropicApiKey || '',
      'anthropic-version': '2023-06-01',
    },
    body: JSON.stringify({
      model: 'claude-sonnet-4-20250514',
      max_tokens: 4096,
      system: systemPrompt || 'You are a professional business communication expert.',
      messages: [{ role: 'user', content: prompt }],
    }),
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Claude API error: ${error}`);
  }

  const data = await response.json() as { content: Array<{ text: string }> };
  return data.content[0].text;
}

// OpenAI API call
async function callOpenAI(prompt: string, systemPrompt?: string): Promise<string> {
  const response = await fetch('https://api.openai.com/v1/chat/completions', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${config.openaiApiKey}`,
    },
    body: JSON.stringify({
      model: 'gpt-4o-mini',
      messages: [
        { role: 'system', content: systemPrompt || 'You are a helpful business analyst.' },
        { role: 'user', content: prompt },
      ],
      temperature: 0.7,
    }),
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`OpenAI API error: ${error}`);
  }

  const data = await response.json() as { choices: Array<{ message: { content: string } }> };
  return data.choices[0].message.content;
}

// Analyze lead - pros, cons, opportunities
export async function analyzeLead(leadId: string, userId: string): Promise<LeadAnalysis> {
  // Get lead data
  const leadResult = await pool.query(
    `SELECT * FROM leads WHERE id = $1 AND user_id = $2`,
    [leadId, userId]
  );

  if (leadResult.rows.length === 0) {
    throw new Error('Lead not found');
  }

  const lead = leadResult.rows[0];

  // Get user profile/skills for matching
  const userResult = await pool.query(
    `SELECT u.*, us.* FROM users u
     LEFT JOIN user_settings us ON us.user_id = u.id
     WHERE u.id = $1`,
    [userId]
  );
  const user = userResult.rows[0];

  const prompt = `Analyze this business lead and provide insights in JSON format.

LEAD INFORMATION:
- Company: ${lead.company_name}
- Industry: ${lead.industry || 'Unknown'}
- Company Size: ${lead.company_size || 'Unknown'}
- Contact: ${lead.contact_name || 'Unknown'} - ${lead.contact_title || 'Unknown'}
- Location: ${lead.location || 'Unknown'}
- Tech Stack: ${JSON.stringify(lead.tech_stack) || 'Unknown'}
- Notes: ${lead.notes || 'None'}
- Source: ${lead.source || 'Unknown'}

USER CONTEXT:
- Name: ${user.full_name || 'Unknown'}
- Role: Looking to do outreach for business/opportunities

Provide analysis in this exact JSON format:
{
  "matchScore": <number 0-100>,
  "pros": [{"point": "<strength>", "reasoning": "<why this is good>"}],
  "cons": [{"point": "<challenge>", "mitigation": "<how to handle>"}],
  "opportunities": [{"description": "<opportunity>", "howToLeverage": "<action>", "potentialValue": "<value>"}],
  "nextSteps": ["<step1>", "<step2>"],
  "talkingPoints": ["<point1>", "<point2>"],
  "questionsToAsk": ["<question1>", "<question2>"],
  "summary": "<2-3 sentence summary>"
}

Be specific and actionable. Focus on real business value.`;

  const response = await callOpenAI(prompt, 'You are a business analyst expert. Always respond with valid JSON only, no markdown.');

  try {
    // Parse JSON from response (handle potential markdown code blocks)
    let jsonStr = response;
    if (response.includes('```json')) {
      jsonStr = response.split('```json')[1].split('```')[0];
    } else if (response.includes('```')) {
      jsonStr = response.split('```')[1].split('```')[0];
    }

    const analysis = JSON.parse(jsonStr.trim());

    // Store analysis in database
    await pool.query(
      `UPDATE leads SET
        custom_fields = custom_fields || $2
      WHERE id = $1`,
      [leadId, JSON.stringify({ aiAnalysis: analysis, analyzedAt: new Date().toISOString() })]
    );

    return {
      ...analysis,
      skillsMatch: analysis.skillsMatch || [],
    };
  } catch (error) {
    console.error('Failed to parse AI response:', response);
    throw new Error('Failed to analyze lead');
  }
}

// Generate personalized email using Claude
export async function generateEmail(params: EmailGenerationParams): Promise<{
  subject: string;
  bodyHtml: string;
  bodyText: string;
}> {
  // Get lead data
  const leadResult = await pool.query(
    `SELECT * FROM leads WHERE id = $1 AND user_id = $2`,
    [params.leadId, params.userId]
  );

  if (leadResult.rows.length === 0) {
    throw new Error('Lead not found');
  }

  const lead = leadResult.rows[0];

  // Get user data
  const userResult = await pool.query(
    `SELECT * FROM users WHERE id = $1`,
    [params.userId]
  );
  const user = userResult.rows[0];

  const purposeInstructions: Record<string, string> = {
    cold_outreach: 'This is a first-time outreach. Be engaging but not pushy. Focus on value proposition.',
    follow_up: 'This is a follow-up email. Reference previous communication and add new value.',
    cv_submission: 'This email accompanies a CV/resume. Highlight key qualifications that match their needs.',
    meeting_request: 'Request a meeting or call. Provide clear value and make it easy to say yes.',
    thank_you: 'Thank them for their time/consideration. Reinforce key points and next steps.',
  };

  const toneInstructions: Record<string, string> = {
    formal: 'Use formal, business language. Proper salutations and closings.',
    professional: 'Professional but approachable. Clear and direct.',
    casual: 'Conversational and friendly, but still business-appropriate.',
    friendly: 'Warm and personable, like writing to a colleague.',
  };

  const prompt = `Write a professional outreach email.

RECIPIENT:
- Name: ${lead.contact_name || 'there'}
- Title: ${lead.contact_title || 'Professional'}
- Company: ${lead.company_name}
- Industry: ${lead.industry || 'their industry'}

SENDER:
- Name: ${user.full_name || 'Professional'}

PURPOSE: ${purposeInstructions[params.purpose]}
TONE: ${toneInstructions[params.tone]}
${params.customInstructions ? `SPECIAL INSTRUCTIONS: ${params.customInstructions}` : ''}
${params.includeCV ? 'NOTE: A CV/resume will be attached.' : ''}

Write the email with:
1. A compelling subject line (not generic)
2. Personalized opening (reference something specific about them/company if possible)
3. Clear value proposition
4. Soft call-to-action
5. Professional signature placeholder

Respond in this JSON format:
{
  "subject": "<compelling subject line>",
  "bodyHtml": "<full HTML email with proper formatting>",
  "bodyText": "<plain text version>"
}

Make it feel human-written, not templated. Max 150 words for the body.`;

  const response = await callClaude(prompt, 'You are an expert at writing compelling business emails that get responses. Always respond with valid JSON only.');

  try {
    let jsonStr = response;
    if (response.includes('```json')) {
      jsonStr = response.split('```json')[1].split('```')[0];
    } else if (response.includes('```')) {
      jsonStr = response.split('```')[1].split('```')[0];
    }

    return JSON.parse(jsonStr.trim());
  } catch (error) {
    console.error('Failed to parse Claude response:', response);
    throw new Error('Failed to generate email');
  }
}

// Personalize template for specific lead
export async function personalizeTemplate(
  templateContent: string,
  leadId: string,
  userId: string
): Promise<string> {
  // Get lead data
  const leadResult = await pool.query(
    `SELECT * FROM leads WHERE id = $1 AND user_id = $2`,
    [leadId, userId]
  );

  if (leadResult.rows.length === 0) {
    throw new Error('Lead not found');
  }

  const lead = leadResult.rows[0];

  // Get user data
  const userResult = await pool.query(
    `SELECT * FROM users WHERE id = $1`,
    [userId]
  );
  const user = userResult.rows[0];

  // Replace standard variables
  let personalized = templateContent
    .replace(/\{\{firstName\}\}/g, lead.contact_name?.split(' ')[0] || 'there')
    .replace(/\{\{fullName\}\}/g, lead.contact_name || '')
    .replace(/\{\{companyName\}\}/g, lead.company_name || '')
    .replace(/\{\{title\}\}/g, lead.contact_title || '')
    .replace(/\{\{industry\}\}/g, lead.industry || '')
    .replace(/\{\{senderName\}\}/g, user.full_name || '')
    .replace(/\{\{senderEmail\}\}/g, user.email || '');

  // AI-generate dynamic content for special tags
  if (personalized.includes('{{icebreaker}}') || personalized.includes('{{painPoint}}')) {
    const prompt = `Generate personalized content for an email to ${lead.contact_name} at ${lead.company_name} (${lead.industry || 'unknown industry'}).

Generate:
1. icebreaker: A personalized opening line that shows you've done research
2. painPoint: A specific challenge their company/role might face

Respond in JSON: {"icebreaker": "...", "painPoint": "..."}`;

    const response = await callOpenAI(prompt);
    try {
      let jsonStr = response;
      if (response.includes('```')) {
        jsonStr = response.split('```')[1].split('```')[0].replace('json', '');
      }
      const dynamic = JSON.parse(jsonStr.trim());

      personalized = personalized
        .replace(/\{\{icebreaker\}\}/g, dynamic.icebreaker || '')
        .replace(/\{\{painPoint\}\}/g, dynamic.painPoint || '');
    } catch (e) {
      // Remove unfilled tags
      personalized = personalized
        .replace(/\{\{icebreaker\}\}/g, '')
        .replace(/\{\{painPoint\}\}/g, '');
    }
  }

  return personalized;
}

// Analyze CV and suggest improvements
export async function analyzeCV(cvText: string, targetJobDescription?: string): Promise<CVAnalysis> {
  const prompt = `Analyze this CV/resume and provide improvement suggestions.

CV CONTENT:
${cvText}

${targetJobDescription ? `TARGET JOB DESCRIPTION:\n${targetJobDescription}` : ''}

Analyze and respond in this JSON format:
{
  "strengths": ["<strength1>", "<strength2>"],
  "weaknesses": ["<weakness1>", "<weakness2>"],
  "suggestions": [
    {"section": "<section name>", "suggestion": "<specific improvement>", "priority": "high|medium|low"}
  ],
  "skillsExtracted": ["<skill1>", "<skill2>"],
  "experienceYears": <number>,
  "overallScore": <number 0-100>
}

Be specific and actionable. Focus on impact and results.`;

  const response = await callClaude(prompt, 'You are a professional career coach and CV expert. Respond with valid JSON only.');

  try {
    let jsonStr = response;
    if (response.includes('```')) {
      jsonStr = response.split('```')[1].split('```')[0].replace('json', '');
    }
    return JSON.parse(jsonStr.trim());
  } catch (error) {
    console.error('Failed to parse CV analysis:', response);
    throw new Error('Failed to analyze CV');
  }
}

// Generate learning recommendations based on skill gaps
export async function generateLearningRecommendations(
  userId: string,
  targetSkills: string[]
): Promise<LearningRecommendation[]> {
  const prompt = `Generate learning recommendations for someone who needs to develop these skills:
${targetSkills.map(s => `- ${s}`).join('\n')}

For each skill, provide FREE resources from:
- YouTube (free tutorials)
- edX (free courses)
- Coursera (free audit option)
- freeCodeCamp
- Khan Academy

Respond in JSON format:
[
  {
    "skill": "<skill name>",
    "reason": "<why this skill is important>",
    "priority": "high|medium|low",
    "resources": [
      {
        "title": "<resource title>",
        "type": "video|course|article|book",
        "platform": "youtube|edx|coursera|freecodecamp|khan",
        "url": "<actual URL if known, otherwise leave as suggested search>",
        "isFree": true,
        "duration": "<estimated time>"
      }
    ]
  }
]

Prioritize free, high-quality resources. Be specific with resource names.`;

  const response = await callOpenAI(prompt);

  try {
    let jsonStr = response;
    if (response.includes('```')) {
      jsonStr = response.split('```')[1].split('```')[0].replace('json', '');
    }
    return JSON.parse(jsonStr.trim());
  } catch (error) {
    console.error('Failed to parse learning recommendations:', response);
    return [];
  }
}

// Generate subject line variations for A/B testing
export async function generateSubjectLines(
  leadId: string,
  userId: string,
  purpose: string,
  count: number = 3
): Promise<string[]> {
  const leadResult = await pool.query(
    `SELECT company_name, contact_name, industry FROM leads WHERE id = $1 AND user_id = $2`,
    [leadId, userId]
  );

  if (leadResult.rows.length === 0) {
    throw new Error('Lead not found');
  }

  const lead = leadResult.rows[0];

  const prompt = `Generate ${count} email subject line variations for:
- Recipient: ${lead.contact_name} at ${lead.company_name}
- Industry: ${lead.industry || 'unknown'}
- Purpose: ${purpose}

Requirements:
- Under 60 characters each
- No spam trigger words
- Personalized when possible
- Create curiosity or value

Respond as JSON array: ["subject1", "subject2", "subject3"]`;

  const response = await callOpenAI(prompt);

  try {
    let jsonStr = response;
    if (response.includes('```')) {
      jsonStr = response.split('```')[1].split('```')[0].replace('json', '');
    }
    return JSON.parse(jsonStr.trim());
  } catch (error) {
    return [`Quick question for ${lead.contact_name}`, `Opportunity for ${lead.company_name}`, `Following up`];
  }
}

export default {
  analyzeLead,
  generateEmail,
  personalizeTemplate,
  analyzeCV,
  generateLearningRecommendations,
  generateSubjectLines,
  callClaude,
  callOpenAI,
};
