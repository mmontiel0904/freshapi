use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::entities::{prelude::*, invitation, user};
use crate::services::EmailService;

#[derive(Clone)]
pub struct InvitationService {
    db: DatabaseConnection,
    email_service: EmailService,
}

impl InvitationService {
    pub fn new(db: DatabaseConnection, email_service: EmailService) -> Self {
        Self { db, email_service }
    }

    pub async fn create_invitation(
        &self,
        inviter_user_id: Uuid,
        email: &str,
        base_url: &str,
    ) -> Result<invitation::Model, Box<dyn std::error::Error>> {
        // Check if user already exists
        if let Some(_) = User::find()
            .filter(user::Column::Email.eq(email))
            .one(&self.db)
            .await?
        {
            return Err("User with this email already exists".into());
        }

        // Check if invitation already exists and is still valid
        if let Some(existing_invitation) = Invitation::find()
            .filter(invitation::Column::Email.eq(email))
            .filter(invitation::Column::IsUsed.eq(false))
            .filter(invitation::Column::ExpiresAt.gt(Utc::now()))
            .one(&self.db)
            .await?
        {
            return Err("An active invitation already exists for this email".into());
        }

        // Generate invitation token
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::days(7); // 7 days to accept invitation

        // Create invitation
        let new_invitation = invitation::ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set(email.to_string()),
            inviter_user_id: Set(inviter_user_id),
            token: Set(token.clone()),
            expires_at: Set(expires_at.into()),
            is_used: Set(false),
            used_at: Set(None),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };

        let invitation = new_invitation.insert(&self.db).await?;

        // Send invitation email
        self.email_service
            .send_invitation_email(email, &token, base_url)
            .await?;

        Ok(invitation)
    }

    pub async fn validate_invitation_token(
        &self,
        token: &str,
    ) -> Result<invitation::Model, Box<dyn std::error::Error>> {
        let invitation = Invitation::find()
            .filter(invitation::Column::Token.eq(token))
            .filter(invitation::Column::IsUsed.eq(false))
            .one(&self.db)
            .await?
            .ok_or("Invalid or used invitation token")?;

        // Check if invitation has expired
        if Utc::now() > invitation.expires_at {
            return Err("Invitation has expired".into());
        }

        Ok(invitation)
    }

    pub async fn use_invitation(
        &self,
        token: &str,
    ) -> Result<invitation::Model, Box<dyn std::error::Error>> {
        let invitation = self.validate_invitation_token(token).await?;

        // Mark invitation as used
        let mut invitation_active: invitation::ActiveModel = invitation.clone().into();
        invitation_active.is_used = Set(true);
        invitation_active.used_at = Set(Some(Utc::now().into()));
        invitation_active.updated_at = Set(Utc::now().into());

        let updated_invitation = invitation_active.update(&self.db).await?;
        Ok(updated_invitation)
    }

    pub async fn get_invitations_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<invitation::Model>, Box<dyn std::error::Error>> {
        let invitations = Invitation::find()
            .filter(invitation::Column::InviterUserId.eq(user_id))
            .all(&self.db)
            .await?;

        Ok(invitations)
    }
}