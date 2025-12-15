// API client for hekax-api backend

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

interface ApiResponse<T> {
  data?: T;
  error?: {
    code: string;
    message: string;
  };
}

class ApiClient {
  private accessToken: string | null = null;
  private refreshToken: string | null = null;
  private deviceId: string | null = null;

  constructor() {
    if (typeof window !== 'undefined') {
      this.accessToken = localStorage.getItem('access_token');
      this.refreshToken = localStorage.getItem('refresh_token');
      this.deviceId = localStorage.getItem('device_id') || this.generateDeviceId();
    }
  }

  private generateDeviceId(): string {
    const id = crypto.randomUUID();
    if (typeof window !== 'undefined') {
      localStorage.setItem('device_id', id);
    }
    return id;
  }

  setTokens(accessToken: string, refreshToken: string) {
    this.accessToken = accessToken;
    this.refreshToken = refreshToken;
    if (typeof window !== 'undefined') {
      localStorage.setItem('access_token', accessToken);
      localStorage.setItem('refresh_token', refreshToken);
    }
  }

  clearTokens() {
    this.accessToken = null;
    this.refreshToken = null;
    if (typeof window !== 'undefined') {
      localStorage.removeItem('access_token');
      localStorage.removeItem('refresh_token');
    }
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...(options.headers as Record<string, string>),
    };

    if (this.accessToken) {
      headers['Authorization'] = `Bearer ${this.accessToken}`;
    }

    const response = await fetch(`${API_URL}${endpoint}`, {
      ...options,
      headers,
    });

    // Handle token refresh
    if (response.status === 401 && this.refreshToken) {
      const refreshed = await this.refreshAccessToken();
      if (refreshed) {
        headers['Authorization'] = `Bearer ${this.accessToken}`;
        const retryResponse = await fetch(`${API_URL}${endpoint}`, {
          ...options,
          headers,
        });
        return this.handleResponse<T>(retryResponse);
      }
    }

    return this.handleResponse<T>(response);
  }

  private async handleResponse<T>(response: Response): Promise<T> {
    const data = await response.json();

    if (!response.ok) {
      throw new ApiError(
        data.error?.message || 'An error occurred',
        data.error?.code || 'UNKNOWN_ERROR',
        response.status
      );
    }

    return data;
  }

  private async refreshAccessToken(): Promise<boolean> {
    if (!this.refreshToken || !this.deviceId) return false;

    try {
      const response = await fetch(`${API_URL}/auth/refresh`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          refresh_token: this.refreshToken,
          device_id: this.deviceId,
        }),
      });

      if (!response.ok) {
        this.clearTokens();
        return false;
      }

      const data = await response.json();
      this.setTokens(data.access_token, data.refresh_token);
      return true;
    } catch {
      this.clearTokens();
      return false;
    }
  }

  // Auth
  async register(email: string, password: string, fullName: string) {
    return this.request<AuthResponse>('/auth/register', {
      method: 'POST',
      body: JSON.stringify({ email, password, full_name: fullName }),
    });
  }

  async login(email: string, password: string) {
    const response = await this.request<AuthResponse>('/auth/login', {
      method: 'POST',
      body: JSON.stringify({
        email,
        password,
        device_id: this.deviceId,
        device_name: 'Web Browser',
      }),
    });
    this.setTokens(response.access_token, response.refresh_token);
    return response;
  }

  async logout() {
    try {
      await this.request('/auth/logout', {
        method: 'POST',
        body: JSON.stringify({
          refresh_token: this.refreshToken,
          device_id: this.deviceId,
        }),
      });
    } finally {
      this.clearTokens();
    }
  }

  async me() {
    return this.request<User>('/auth/me');
  }

  // Leads
  async getLeads(params?: LeadListParams) {
    const query = new URLSearchParams();
    if (params?.status) query.set('status', params.status);
    if (params?.search) query.set('search', params.search);
    if (params?.page) query.set('page', String(params.page));
    if (params?.perPage) query.set('per_page', String(params.perPage));

    return this.request<LeadListResponse>(`/leads?${query}`);
  }

  async getLead(id: string) {
    return this.request<Lead>(`/leads/${id}`);
  }

  async createLead(data: CreateLeadRequest) {
    return this.request<Lead>('/leads', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async updateLead(id: string, data: UpdateLeadRequest) {
    return this.request<Lead>(`/leads/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  async deleteLead(id: string) {
    return this.request(`/leads/${id}`, { method: 'DELETE' });
  }

  // Recordings
  async getRecordings(params?: RecordingListParams) {
    const query = new URLSearchParams();
    if (params?.leadId) query.set('lead_id', params.leadId);
    if (params?.mode) query.set('mode', params.mode);
    if (params?.outcome) query.set('outcome', params.outcome);
    if (params?.page) query.set('page', String(params.page));

    return this.request<RecordingListResponse>(`/recordings?${query}`);
  }

  async getRecording(id: string) {
    return this.request<Recording>(`/recordings/${id}`);
  }

  // Settings
  async getSettings() {
    return this.request<UserSettings>('/users/settings');
  }

  async updateSettings(data: UpdateSettingsRequest) {
    return this.request<UserSettings>('/users/settings', {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }
}

export class ApiError extends Error {
  constructor(
    message: string,
    public code: string,
    public status: number
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

// Types
export interface User {
  id: string;
  email: string;
  full_name: string | null;
  avatar_url: string | null;
  subscription_tier: string;
  email_verified: boolean;
  created_at: string;
}

export interface AuthResponse {
  user: User;
  access_token: string;
  refresh_token: string;
  expires_in: number;
}

export interface Lead {
  id: string;
  user_id: string;
  company_name: string;
  company_domain: string | null;
  company_linkedin: string | null;
  company_size: string | null;
  contact_name: string | null;
  contact_title: string | null;
  contact_email: string | null;
  contact_phone: string | null;
  contact_linkedin: string | null;
  status: string;
  priority: number;
  industry: string | null;
  notes: string | null;
  tags: string[] | null;
  tech_stack: string[] | null;
  funding_info: Record<string, unknown> | null;
  source: string | null;
  created_at: string;
  updated_at: string;
}

export interface LeadListParams {
  status?: string;
  search?: string;
  page?: number;
  perPage?: number;
}

export interface LeadListResponse {
  leads: Lead[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

export interface CreateLeadRequest {
  company_name: string;
  company_domain?: string;
  company_linkedin?: string;
  company_size?: string;
  contact_name?: string;
  contact_title?: string;
  contact_email?: string;
  contact_phone?: string;
  contact_linkedin?: string;
  industry?: string;
  status?: string;
  priority?: number;
  notes?: string;
  tags?: string[];
  tech_stack?: string[];
  source?: string;
}

export interface UpdateLeadRequest {
  company_name?: string;
  company_domain?: string | null;
  company_linkedin?: string | null;
  company_size?: string | null;
  contact_name?: string | null;
  contact_title?: string | null;
  contact_email?: string | null;
  contact_phone?: string | null;
  contact_linkedin?: string | null;
  industry?: string | null;
  status?: string;
  priority?: number;
  notes?: string | null;
  tags?: string[] | null;
  tech_stack?: string[] | null;
  source?: string | null;
}

export interface Recording {
  id: string;
  user_id: string;
  lead_id: string | null;
  mode: string;
  status: string;
  start_time: string;
  end_time: string | null;
  duration_seconds: number | null;
  summary: string | null;
  outcome: string | null;
  transcript_turns: TranscriptTurn[] | null;
  key_points: string[] | null;
  action_items: string[] | null;
  talk_ratio: number | null;
  sentiment_score: number | null;
}

export interface TranscriptTurn {
  speaker: string;
  text: string;
  timestamp?: number;
}

export interface RecordingListParams {
  leadId?: string;
  mode?: string;
  outcome?: string;
  page?: number;
}

export interface RecordingListResponse {
  recordings: Recording[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

export interface UserSettings {
  default_mode: string;
  auto_record: boolean;
  stealth_mode_default: boolean;
  theme: string;
}

export interface UpdateSettingsRequest extends Partial<UserSettings> {}

// ============================================================================
// EMAIL TYPES
// ============================================================================

export interface Email {
  id: string;
  toEmail: string;
  subject: string;
  status: 'sent' | 'delivered' | 'opened' | 'clicked' | 'replied' | 'bounced';
  sentAt: string;
  openedAt: string | null;
  clickedAt: string | null;
  leadId: string | null;
  purpose: string | null;
  companyName: string | null;
  contactName: string | null;
}

export interface EmailDetail extends Email {
  fromEmail: string;
  metadata: Record<string, unknown>;
  lead: {
    id: string;
    companyName: string;
    contactName: string;
    contactTitle: string;
    contactEmail: string;
    contactLinkedin: string;
    industry: string;
    companySize: string;
    status: string;
  } | null;
  analysis: {
    matchScore: number;
    pros: Array<{ point: string; reasoning: string }>;
    cons: Array<{ point: string; mitigation: string }>;
    opportunities: Array<{ description: string; howToLeverage: string; potentialValue: string }>;
    nextSteps: string[];
    talkingPoints: string[];
    questionsToAsk: string[];
    summary: string;
  } | null;
}

export interface EmailListResponse {
  emails: Email[];
  total: number;
  page: number;
  perPage: number;
  totalPages: number;
}

export interface EmailStats {
  total: number;
  sent: number;
  delivered: number;
  opened: number;
  clicked: number;
  replied: number;
  bounced: number;
  openRate: string;
  clickRate: string;
  replyRate: string;
}

export interface SendEmailRequest {
  leadId: string;
  templateId?: string;
  subject: string;
  bodyHtml: string;
  bodyText?: string;
  purpose: string;
}

export interface GenerateEmailRequest {
  leadId: string;
  purpose: 'cold_outreach' | 'follow_up' | 'cv_submission' | 'meeting_request' | 'thank_you';
  tone: 'formal' | 'professional' | 'casual' | 'friendly';
  includeCV?: boolean;
  customInstructions?: string;
}

export interface GeneratedEmail {
  subject: string;
  bodyHtml: string;
  bodyText: string;
}

export interface EmailTemplate {
  id: string;
  name: string;
  subject: string;
  bodyHtml: string;
  bodyText: string | null;
  category: string | null;
  variables: string[];
  isSystem: boolean;
  createdAt: string;
  updatedAt?: string;
}

export interface TemplateListResponse {
  systemTemplates: EmailTemplate[];
  userTemplates: EmailTemplate[];
}

export interface CreateTemplateRequest {
  name: string;
  subject: string;
  bodyHtml: string;
  bodyText?: string;
  category?: string;
  variables?: string[];
}

export interface GenerateTemplateRequest {
  purpose?: string;
  tone?: string;
  industry?: string;
  description?: string;
}

// ============================================================================
// LEAD ANALYSIS TYPES
// ============================================================================

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

// ============================================================================
// CV PROCESSING TYPES
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

export interface CVProfile {
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
  remotePreference?: 'remote' | 'hybrid' | 'onsite';
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

export interface CVTemplate {
  id: string;
  name: string;
  description: string;
  category: string;
  previewImageUrl?: string;
  isPremium: boolean;
}

export interface TailorCVRequest {
  leadId?: string;
  jobTitle?: string;
  jobDescription?: string;
  companyName?: string;
  templateId?: string;
}

export interface TailorCVResponse {
  tailoredProfile: CVProfile;
  changes: string[];
  generationId: string;
}

// Create base client
const baseApi = new ApiClient();

// Extended API client with email methods
export const api = Object.assign(baseApi, {
  // Emails
  getEmails: async (params?: { status?: string; leadId?: string; page?: number; perPage?: number }): Promise<EmailListResponse> => {
    const query = new URLSearchParams();
    if (params?.status) query.set('status', params.status);
    if (params?.leadId) query.set('leadId', params.leadId);
    if (params?.page) query.set('page', String(params.page));
    if (params?.perPage) query.set('perPage', String(params.perPage));
    return (baseApi as any).request(`/emails?${query}`);
  },

  getEmail: async (id: string): Promise<EmailDetail> => {
    return (baseApi as any).request(`/emails/${id}`);
  },

  sendEmail: async (data: SendEmailRequest): Promise<{ success: boolean; emailId: string; messageId: string }> => {
    return (baseApi as any).request('/emails/send', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  },

  generateEmail: async (data: GenerateEmailRequest): Promise<GeneratedEmail> => {
    return (baseApi as any).request('/emails/generate', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  },

  getEmailStats: async (): Promise<EmailStats> => {
    return (baseApi as any).request('/emails/stats/overview');
  },

  markEmailReplied: async (id: string): Promise<{ success: boolean }> => {
    return (baseApi as any).request(`/emails/${id}/mark-replied`, {
      method: 'POST',
    });
  },

  deleteEmail: async (id: string): Promise<{ success: boolean }> => {
    return (baseApi as any).request(`/emails/${id}`, {
      method: 'DELETE',
    });
  },

  // Templates
  getTemplates: async (category?: string): Promise<TemplateListResponse> => {
    const query = category ? `?category=${category}` : '';
    return (baseApi as any).request(`/templates${query}`);
  },

  getTemplate: async (id: string): Promise<EmailTemplate> => {
    return (baseApi as any).request(`/templates/${id}`);
  },

  createTemplate: async (data: CreateTemplateRequest): Promise<EmailTemplate> => {
    return (baseApi as any).request('/templates', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  },

  updateTemplate: async (id: string, data: Partial<CreateTemplateRequest>): Promise<EmailTemplate> => {
    return (baseApi as any).request(`/templates/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  },

  deleteTemplate: async (id: string): Promise<{ success: boolean }> => {
    return (baseApi as any).request(`/templates/${id}`, {
      method: 'DELETE',
    });
  },

  generateTemplate: async (data: GenerateTemplateRequest): Promise<{ name: string; subject: string; bodyHtml: string; bodyText: string }> => {
    return (baseApi as any).request('/templates/generate', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  },

  previewTemplate: async (id: string, leadId: string): Promise<{ subject: string; bodyHtml: string }> => {
    return (baseApi as any).request(`/templates/${id}/preview`, {
      method: 'POST',
      body: JSON.stringify({ leadId }),
    });
  },

  // Lead Analysis
  analyzeLead: async (leadId: string): Promise<LeadAnalysis> => {
    return (baseApi as any).request(`/leads/${leadId}/analyze`, {
      method: 'POST',
    });
  },

  // ============================================================================
  // CV PROCESSING
  // ============================================================================

  // Get user's CV profile
  getCVProfile: async (): Promise<CVProfile> => {
    return (baseApi as any).request('/cv/profile');
  },

  // Save CV profile
  saveCVProfile: async (profile: Partial<CVProfile>): Promise<{ success: boolean; profileId: string }> => {
    return (baseApi as any).request('/cv/profile', {
      method: 'POST',
      body: JSON.stringify(profile),
    });
  },

  // Update CV profile
  updateCVProfile: async (profile: Partial<CVProfile>): Promise<{ success: boolean; profileId: string }> => {
    return (baseApi as any).request('/cv/profile', {
      method: 'PATCH',
      body: JSON.stringify(profile),
    });
  },

  // Analyze CV for ATS optimization
  analyzeCV: async (jobDescription?: string): Promise<CVAnalysis> => {
    return (baseApi as any).request('/cv/analyze', {
      method: 'POST',
      body: JSON.stringify({ jobDescription }),
    });
  },

  // Tailor CV for specific job/lead
  tailorCV: async (params: TailorCVRequest): Promise<TailorCVResponse> => {
    return (baseApi as any).request('/cv/tailor', {
      method: 'POST',
      body: JSON.stringify(params),
    });
  },

  // Generate HTML version of CV
  generateCVHtml: async (templateId?: string): Promise<{ html: string }> => {
    return (baseApi as any).request('/cv/generate-html', {
      method: 'POST',
      body: JSON.stringify({ templateId }),
    });
  },

  // Generate HTML from custom profile data
  generateCVHtmlCustom: async (profile: CVProfile, templateId?: string): Promise<{ html: string }> => {
    return (baseApi as any).request('/cv/generate-html-custom', {
      method: 'POST',
      body: JSON.stringify({ profile, templateId }),
    });
  },

  // Parse resume text into structured data
  parseCVText: async (text: string): Promise<CVProfile> => {
    return (baseApi as any).request('/cv/parse-text', {
      method: 'POST',
      body: JSON.stringify({ text }),
    });
  },

  // Get CV templates
  getCVTemplates: async (category?: string): Promise<{ templates: CVTemplate[] }> => {
    const query = category ? `?category=${category}` : '';
    return (baseApi as any).request(`/cv/templates${query}`);
  },
});
