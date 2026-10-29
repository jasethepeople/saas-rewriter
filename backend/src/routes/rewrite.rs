use axum::{
    extract::{Path, State},
    routing::{get, post},
    Extension, Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::{Claims, JobStatus, RewriteJob, RewriteStyle, RewriteTone},
    AppError, AppState, Result,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_rewrite))
        .route("/auto", post(auto_rewrite))
        .route("/analyze", post(analyze_text))
        .route("/history", get(get_history))
        .route("/:id", get(get_rewrite))
}

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Deserialize)]
struct CreateRewriteRequest {
    text: String,
    tone: RewriteTone,
    style: RewriteStyle,
}

#[derive(Deserialize)]
struct AutoRewriteRequest {
    text: String,
}

#[derive(Deserialize)]
struct AnalyzeRequest {
    text: String,
}

#[derive(Serialize)]
struct RewriteJobResponse {
    job: RewriteJob,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn check_and_decrement_quota(
    db: &sqlx::PgPool,
    user_id: Uuid,
) -> Result<()> {
    let row = sqlx::query!(
        "SELECT rewrites_used, rewrites_limit FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    if row.rewrites_used >= row.rewrites_limit {
        return Err(AppError::RateLimitExceeded);
    }

    sqlx::query!(
        "UPDATE users SET rewrites_used = rewrites_used + 1, updated_at = $1 WHERE id = $2",
        Utc::now(),
        user_id
    )
    .execute(db)
    .await?;

    Ok(())
}

async fn create_rewrite(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateRewriteRequest>,
) -> Result<Json<RewriteJobResponse>> {
    if req.text.trim().is_empty() {
        return Err(AppError::BadRequest("Text cannot be empty".into()));
    }
    if req.text.len() > 10_000 {
        return Err(AppError::BadRequest("Text exceeds 10,000 character limit".into()));
    }

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".into()))?;

    check_and_decrement_quota(&state.db, user_id).await?;

    // Create job record
    let job_id = Uuid::new_v4();
    let now = Utc::now();

    let job = sqlx::query_as::<_, RewriteJob>(
        r#"INSERT INTO rewrite_jobs (id, user_id, original_text, tone, style, status, created_at)
           VALUES ($1, $2, $3, $4, $5, 'processing', $6)
           RETURNING *"#,
    )
    .bind(job_id)
    .bind(user_id)
    .bind(&req.text)
    .bind(&req.tone)
    .bind(&req.style)
    .bind(now)
    .fetch_one(&state.db)
    .await?;

    // Call OpenAI
    let openai = state.openai.clone();
    let text = req.text.clone();
    let tone = req.tone;
    let style = req.style;
    let db = state.db.clone();

    tokio::spawn(async move {
        match openai.rewrite_text(&text, &tone, &style).await {
            Ok(resp) => {
                let analysis_json = serde_json::to_value(&resp).ok();
                let _ = sqlx::query!(
                    r#"UPDATE rewrite_jobs
                       SET rewritten_text = $1, status = 'completed', analysis = $2, completed_at = $3
                       WHERE id = $4"#,
                    resp.rewritten_text,
                    analysis_json,
                    Utc::now(),
                    job_id
                )
                .execute(&db)
                .await;
            }
            Err(e) => {
                tracing::error!("Rewrite job {} failed: {:?}", job_id, e);
                let _ = sqlx::query!(
                    "UPDATE rewrite_jobs SET status = 'failed' WHERE id = $1",
                    job_id
                )
                .execute(&db)
                .await;
            }
        }
    });

    Ok(Json(RewriteJobResponse { job }))
}

async fn auto_rewrite(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<AutoRewriteRequest>,
) -> Result<Json<serde_json::Value>> {
    if req.text.trim().is_empty() {
        return Err(AppError::BadRequest("Text cannot be empty".into()));
    }

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".into()))?;

    check_and_decrement_quota(&state.db, user_id).await?;

    let (analysis, rewrite) = state.openai.auto_rewrite(&req.text).await?;

    // Persist the job
    let job_id = Uuid::new_v4();
    let analysis_json = serde_json::to_value(&rewrite).ok();

    sqlx::query!(
        r#"INSERT INTO rewrite_jobs
           (id, user_id, original_text, rewritten_text, tone, style, status, analysis, created_at, completed_at)
           VALUES ($1, $2, $3, $4, 'professional', 'improve', 'completed', $5, $6, $6)"#,
        job_id,
        user_id,
        req.text,
        rewrite.rewritten_text,
        analysis_json,
        Utc::now()
    )
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({
        "job_id": job_id,
        "analysis": analysis,
        "rewrite": rewrite
    })))
}

async fn analyze_text(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Json(req): Json<AnalyzeRequest>,
) -> Result<Json<serde_json::Value>> {
    let analysis = state.openai.analyze_text(&req.text).await?;
    Ok(Json(serde_json::to_value(analysis).unwrap()))
}

async fn get_history(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<RewriteJob>>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".into()))?;

    let jobs = sqlx::query_as::<_, RewriteJob>(
        "SELECT * FROM rewrite_jobs WHERE user_id = $1 ORDER BY created_at DESC LIMIT 50"
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(jobs))
}

async fn get_rewrite(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<RewriteJobResponse>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".into()))?;

    let job = sqlx::query_as::<_, RewriteJob>(
        "SELECT * FROM rewrite_jobs WHERE id = $1 AND user_id = $2"
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Job {} not found", id)))?;

    Ok(Json(RewriteJobResponse { job }))
}
