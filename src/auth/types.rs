use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid, // user id
    pub email: String,
    pub exp: i64,   // expiration timestamp
    pub iat: i64,   // issued at timestamp
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub email: String,
}

impl From<Claims> for AuthenticatedUser {
    fn from(claims: Claims) -> Self {
        Self {
            id: claims.sub,
            email: claims.email,
        }
    }
}