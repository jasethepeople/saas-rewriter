use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ── User ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub plan: Plan,
    pub rewrites_used: i32,
    pub rewrites_limit: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "plan", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Plan {
    Free,
    Pro,
    Enterprise,
}

impl Plan {
    pub fn monthly_rewrite_limit(&self) -> i32 {
        match self {
            Plan::Free => 10,
            Plan::Pro => 500,
            Plan::Enterprise => i32::MAX,
        }
    }
}

// ── Rewrite Job ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RewriteJob {
    pub id: Uuid,
    pub user_id: Uuid,
    pub original_text: String,
    pub rewritten_text: Option<String>,
    pub tone: RewriteTone,
    pub style: RewriteStyle,
    pub status: JobStatus,
    pub analysis: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "rewrite_tone", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum RewriteTone {
    Professional,
    Casual,
    Persuasive,
    Concise,
    Academic,
    Empathetic,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "rewrite_style", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum RewriteStyle {
    Improve,
    Simplify,
    Expand,
    Condense,
    Fix,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "job_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

// ── Auth ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub email: String,
    pub exp: usize,
}
