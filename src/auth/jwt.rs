use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

use crate::auth::types::Claims;

#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_token_expiration_hours: i64,
    refresh_token_expiration_days: i64,
}

impl JwtService {
    pub fn new(secret: &str, access_token_expiration_hours: i64, refresh_token_expiration_days: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
            access_token_expiration_hours,
            refresh_token_expiration_days,
        }
    }

    pub fn generate_access_token(&self, user_id: Uuid, email: &str) -> Result<String, jsonwebtoken::errors::Error> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.access_token_expiration_hours);
        
        let claims = Claims {
            sub: user_id,
            email: email.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
    }

    pub fn generate_refresh_token(&self) -> String {
        Uuid::new_v4().to_string()
    }

    pub fn get_refresh_token_expiration(&self) -> chrono::DateTime<Utc> {
        Utc::now() + Duration::days(self.refresh_token_expiration_days)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &Validation::default())?;
        Ok(token_data.claims)
    }

    // Legacy method for backward compatibility
    pub fn generate_token(&self, user_id: Uuid, email: &str) -> Result<String, jsonwebtoken::errors::Error> {
        self.generate_access_token(user_id, email)
    }
}