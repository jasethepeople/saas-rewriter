use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::{services::auth::AuthService, AppError, AppState, Result};

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".into()))?;

    let auth = AuthService::new(state.config.jwt_secret.clone());
    let claims = auth.verify_token(token)?;

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}
