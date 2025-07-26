use async_graphql::*;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(SimpleObject)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::entities::user::Model> for User {
    fn from(user: crate::entities::user::Model) -> Self {
        Self {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            is_email_verified: user.is_email_verified,
            created_at: user.created_at.into(),
            updated_at: user.updated_at.into(),
        }
    }
}

#[derive(InputObject)]
pub struct RegisterInput {
    pub email: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(InputObject)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(SimpleObject)]
pub struct AuthPayload {
    pub user: User,
    pub token: String,
}

#[derive(SimpleObject)]
pub struct MessageResponse {
    pub message: String,
}