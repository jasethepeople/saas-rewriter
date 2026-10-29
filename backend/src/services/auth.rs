use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

use crate::{models::Claims, AppError, Result};

pub struct AuthService {
    secret: String,
}

impl AuthService {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn generate_token(&self, user_id: Uuid, email: &str) -> Result<String> {
        let exp = (Utc::now() + Duration::days(30)).timestamp() as usize;
        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            exp,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(anyhow::anyhow!("JWT encode error: {}", e)))
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))
    }
}
