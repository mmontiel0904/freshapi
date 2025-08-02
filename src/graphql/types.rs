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
    pub role: Option<Role>,
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
            role: None, // Will be populated by resolver when needed
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
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(InputObject)]
pub struct RefreshTokenInput {
    pub refresh_token: String,
}

#[derive(SimpleObject)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(SimpleObject)]
pub struct Invitation {
    pub id: Uuid,
    pub email: String,
    pub inviter_user_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub is_used: bool,
    pub used_at: Option<DateTime<Utc>>,
    pub role: Option<Role>,
    pub created_at: DateTime<Utc>,
}

impl From<crate::entities::invitation::Model> for Invitation {
    fn from(invitation: crate::entities::invitation::Model) -> Self {
        Self {
            id: invitation.id,
            email: invitation.email,
            inviter_user_id: invitation.inviter_user_id,
            expires_at: invitation.expires_at.into(),
            is_used: invitation.is_used,
            used_at: invitation.used_at.map(|dt| dt.into()),
            role: None, // Will be populated by resolver when needed
            created_at: invitation.created_at.into(),
        }
    }
}

#[derive(InputObject)]
pub struct InviteUserInput {
    pub email: String,
}

#[derive(InputObject)]
pub struct AcceptInvitationInput {
    pub invitation_token: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(InputObject)]
pub struct RequestPasswordResetInput {
    pub email: String,
}

#[derive(InputObject)]
pub struct ResetPasswordInput {
    pub token: String,
    pub new_password: String,
}

// RBAC Types
#[derive(SimpleObject)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub level: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::entities::role::Model> for Role {
    fn from(role: crate::entities::role::Model) -> Self {
        Self {
            id: role.id,
            name: role.name,
            description: role.description,
            level: role.level,
            is_active: role.is_active,
            created_at: role.created_at.into(),
            updated_at: role.updated_at.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct Permission {
    pub id: Uuid,
    pub action: String,
    pub resource_name: String,
    pub description: Option<String>,
    pub is_active: bool,
}

#[derive(SimpleObject)]
pub struct UserWithRole {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_email_verified: bool,
    pub role: Option<Role>,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(InputObject)]
pub struct AssignRoleInput {
    pub user_id: Uuid,
    pub role_id: Uuid,
}

#[derive(InputObject)]
pub struct InviteUserWithRoleInput {
    pub email: String,
    pub role_id: Option<Uuid>,
}