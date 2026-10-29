use axum::{middleware, Router};
use crate::AppState;

mod auth;
mod rewrite;
mod user;

pub fn api_router(state: AppState) -> Router {
    let protected = Router::new()
        .nest("/rewrite", rewrite::router())
        .nest("/user", user::router())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::mw::require_auth,
        ));

    Router::new()
        .nest("/auth", auth::router())
        .merge(protected)
        .with_state(state)
}
