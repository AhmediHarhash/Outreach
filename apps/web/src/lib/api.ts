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

export const api = new ApiClient();
