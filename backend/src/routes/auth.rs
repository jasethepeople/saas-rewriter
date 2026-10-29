use axum::{extract::State, routing::post, Json, Router};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::{Plan, User},
    services::auth::AuthService,
    AppError, AppState, Result,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}

#[derive(Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct AuthResponse {
    token: String,
    user: UserPublic,
}

#[derive(Serialize)]
struct UserPublic {
    id: Uuid,
    email: String,
    plan: Plan,
    rewrites_used: i32,
    rewrites_limit: i32,
}

async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>> {
    // Check if user exists
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE email = $1"
    )
    .bind(&req.email)
    .fetch_one(&state.db)
    .await?;

    if existing > 0 {
        return Err(AppError::BadRequest("Email already registered".into()));
    }

    if req.password.len() < 8 {
        return Err(AppError::BadRequest("Password must be at least 8 characters".into()));
    }

    let password_hash = hash(&req.password, DEFAULT_COST)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Bcrypt error: {}", e)))?;

    let plan = Plan::Free;
    let limit = plan.monthly_rewrite_limit();

    let user = sqlx::query_as::<_, User>(
        r#"INSERT INTO users (id, email, password_hash, plan, rewrites_used, rewrites_limit, created_at, updated_at)
           VALUES ($1, $2, $3, $4, 0, $5, $6, $6)
           RETURNING *"#,
    )
    .bind(Uuid::new_v4())
    .bind(&req.email)
    .bind(&password_hash)
    .bind(&plan)
    .bind(limit)
    .bind(Utc::now())
    .fetch_one(&state.db)
    .await?;

    let auth = AuthService::new(state.config.jwt_secret.clone());
    let token = auth.generate_token(user.id, &user.email)?;

    Ok(Json(AuthResponse {
        token,
        user: UserPublic {
            id: user.id,
            email: user.email,
            plan: user.plan,
            rewrites_used: user.rewrites_used,
            rewrites_limit: user.rewrites_limit,
        },
    }))
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid email or password".into()))?;

    let valid = verify(&req.password, &user.password_hash)
        .map_err(|_| AppError::Unauthorized("Invalid email or password".into()))?;

    if !valid {
        return Err(AppError::Unauthorized("Invalid email or password".into()));
    }

    let auth = AuthService::new(state.config.jwt_secret.clone());
    let token = auth.generate_token(user.id, &user.email)?;

    Ok(Json(AuthResponse {
        token,
        user: UserPublic {
            id: user.id,
            email: user.email,
            plan: user.plan,
            rewrites_used: user.rewrites_used,
            rewrites_limit: user.rewrites_limit,
        },
    }))
}
