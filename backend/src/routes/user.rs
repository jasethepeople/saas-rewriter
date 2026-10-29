use axum::{extract::State, routing::get, Extension, Json, Router};
use serde::Serialize;
use uuid::Uuid;

use crate::{models::{Claims, Plan, User}, AppError, AppState, Result};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_me))
        .route("/usage", get(get_usage))
}

#[derive(Serialize)]
struct UserResponse {
    id: Uuid,
    email: String,
    plan: Plan,
    rewrites_used: i32,
    rewrites_limit: i32,
}

async fn get_me(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<UserResponse>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user".into()))?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    Ok(Json(UserResponse {
        id: user.id,
        email: user.email,
        plan: user.plan,
        rewrites_used: user.rewrites_used,
        rewrites_limit: user.rewrites_limit,
    }))
}

#[derive(Serialize)]
struct UsageResponse {
    rewrites_used: i32,
    rewrites_limit: i32,
    rewrites_remaining: i32,
    percentage_used: f32,
}

async fn get_usage(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<UsageResponse>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user".into()))?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    let remaining = (user.rewrites_limit - user.rewrites_used).max(0);
    let pct = if user.rewrites_limit > 0 {
        user.rewrites_used as f32 / user.rewrites_limit as f32 * 100.0
    } else {
        0.0
    };

    Ok(Json(UsageResponse {
        rewrites_used: user.rewrites_used,
        rewrites_limit: user.rewrites_limit,
        rewrites_remaining: remaining,
        percentage_used: pct,
    }))
}
