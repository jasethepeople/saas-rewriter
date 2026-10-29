use anyhow::Context;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub openai_api_key: String,
    pub jwt_secret: String,
    pub port: u16,
    pub environment: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL must be set")?,
            openai_api_key: std::env::var("OPENAI_API_KEY")
                .context("OPENAI_API_KEY must be set")?,
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "change-me-in-production-32-chars-min".into()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3001".into())
                .parse()
                .context("PORT must be a number")?,
            environment: std::env::var("ENVIRONMENT")
                .unwrap_or_else(|_| "development".into()),
        })
    }

    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }
}
