import axios from 'axios';

const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:3001/api/v1';

export const api = axios.create({ baseURL: API_BASE });

// Attach JWT token
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('token');
  if (token) config.headers.Authorization = `Bearer ${token}`;
  return config;
});

// Types mirroring backend
export type RewriteTone = 'professional' | 'casual' | 'persuasive' | 'concise' | 'academic' | 'empathetic';
export type RewriteStyle = 'improve' | 'simplify' | 'expand' | 'condense' | 'fix';
export type JobStatus = 'pending' | 'processing' | 'completed' | 'failed';

export interface Improvement {
  category: 'clarity' | 'grammar' | 'flow' | 'vocabulary' | 'structure' | 'tone';
  description: string;
}

export interface RewriteAnalysis {
  rewritten_text: string;
  changes_summary: string;
  readability_score: number;
  tone_match_score: number;
  word_count_original: number;
  word_count_rewritten: number;
  improvements: Improvement[];
  warnings: string[];
}

export interface TextAnalysis {
  overall_quality: number;
  readability_level: string;
  tone_detected: string;
  issues: { severity: 'low' | 'medium' | 'high'; category: string; description: string }[];
  strengths: string[];
  suggested_tone: string;
  word_count: number;
  avg_sentence_length: number;
}

export interface RewriteJob {
  id: string;
  user_id: string;
  original_text: string;
  rewritten_text?: string;
  tone: RewriteTone;
  style: RewriteStyle;
  status: JobStatus;
  analysis?: RewriteAnalysis;
  created_at: string;
  completed_at?: string;
}

export interface User {
  id: string;
  email: string;
  plan: 'free' | 'pro' | 'enterprise';
  rewrites_used: number;
  rewrites_limit: number;
}

// Auth
export const authApi = {
  register: (email: string, password: string) =>
    api.post<{ token: string; user: User }>('/auth/register', { email, password }),
  login: (email: string, password: string) =>
    api.post<{ token: string; user: User }>('/auth/login', { email, password }),
};

// Rewrites
export const rewriteApi = {
  create: (text: string, tone: RewriteTone, style: RewriteStyle) =>
    api.post<{ job: RewriteJob }>('/rewrite', { text, tone, style }),

  auto: (text: string) =>
    api.post<{ job_id: string; analysis: TextAnalysis; rewrite: RewriteAnalysis }>('/rewrite/auto', { text }),

  analyze: (text: string) =>
    api.post<TextAnalysis>('/rewrite/analyze', { text }),

  get: (id: string) =>
    api.get<{ job: RewriteJob }>(`/rewrite/${id}`),

  history: () =>
    api.get<RewriteJob[]>('/rewrite/history'),
};

// User
export const userApi = {
  me: () => api.get<User>('/user/me'),
  usage: () => api.get<{ rewrites_used: number; rewrites_limit: number; rewrites_remaining: number; percentage_used: number }>('/user/usage'),
};
