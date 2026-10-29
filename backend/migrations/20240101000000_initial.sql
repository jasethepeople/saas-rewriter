-- Create enum types
CREATE TYPE plan AS ENUM ('free', 'pro', 'enterprise');
CREATE TYPE rewrite_tone AS ENUM ('professional', 'casual', 'persuasive', 'concise', 'academic', 'empathetic');
CREATE TYPE rewrite_style AS ENUM ('improve', 'simplify', 'expand', 'condense', 'fix');
CREATE TYPE job_status AS ENUM ('pending', 'processing', 'completed', 'failed');

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    plan plan NOT NULL DEFAULT 'free',
    rewrites_used INTEGER NOT NULL DEFAULT 0,
    rewrites_limit INTEGER NOT NULL DEFAULT 10,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX users_email_idx ON users(email);

-- Rewrite jobs table
CREATE TABLE rewrite_jobs (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    original_text TEXT NOT NULL,
    rewritten_text TEXT,
    tone rewrite_tone NOT NULL DEFAULT 'professional',
    style rewrite_style NOT NULL DEFAULT 'improve',
    status job_status NOT NULL DEFAULT 'pending',
    analysis JSONB,
    created_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ
);

CREATE INDEX rewrite_jobs_user_id_idx ON rewrite_jobs(user_id);
CREATE INDEX rewrite_jobs_status_idx ON rewrite_jobs(status);
CREATE INDEX rewrite_jobs_created_at_idx ON rewrite_jobs(created_at DESC);
