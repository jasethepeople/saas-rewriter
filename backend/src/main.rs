use axum::{Router, middleware};
use std::net::SocketAddr;
use tower_http::{cors::{CorsLayer, Any}, trace::TraceLayer, compression::CompressionLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod error;
mod models;
mod routes;
mod services;
mod middleware as mw;

pub use config::Config;
pub use error::{AppError, Result};

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: Config,
    pub openai: services::openai::OpenAIClient,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,saas_rewriter=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;
    let db = db::connect(&config.database_url).await?;
    let openai = services::openai::OpenAIClient::new(config.openai_api_key.clone());

    let state = AppState { db, config: config.clone(), openai };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api/v1", routes::api_router(state.clone()))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("🚀 Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
