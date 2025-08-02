use async_graphql::*;
use sea_orm::{EntityTrait, ActiveModelTrait, Set};
use chrono::Utc;

use crate::auth::require_user_management;
use crate::graphql::types::{AcceptInvitationInput, AdminResetUserPasswordInput, AuthPayload, ChangePasswordInput, Invitation, InviteUserInput, InviteUserWithRoleInput, LoginInput, MessageResponse, RefreshTokenInput, RegisterInput, RequestPasswordResetInput, ResetPasswordInput, User, AssignRoleInput};
use crate::services::{EmailService, InvitationService, UserService};

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn invite_user(&self, ctx: &Context<'_>, input: InviteUserInput) -> Result<Invitation> {
        use crate::auth::require_permission;
        require_permission(ctx, "freshapi", "invite_users").await?;
        
        let invitation_service = ctx.data::<InvitationService>()?;
        let auth_user = ctx.data::<crate::auth::AuthenticatedUser>()?;
        let frontend_url = ctx.data::<String>()?;

        let invitation = invitation_service
            .create_invitation(auth_user.id, &input.email, frontend_url)
            .await
            .map_err(|e| Error::new(format!("Failed to create invitation: {}", e)))?;

        Ok(invitation.into())
    }

    async fn invite_user_with_role(&self, ctx: &Context<'_>, input: InviteUserWithRoleInput) -> Result<Invitation> {
        require_user_management(ctx, "freshapi").await?;
        
        let invitation_service = ctx.data::<InvitationService>()?;
        let auth_user = ctx.data::<crate::auth::AuthenticatedUser>()?;
        let frontend_url = ctx.data::<String>()?;

        let invitation = invitation_service
            .create_invitation_with_role(auth_user.id, &input.email, input.role_id, frontend_url)
            .await
            .map_err(|e| Error::new(format!("Failed to create invitation: {}", e)))?;

        Ok(invitation.into())
    }

    async fn accept_invitation(&self, ctx: &Context<'_>, input: AcceptInvitationInput) -> Result<AuthPayload> {
        let user_service = ctx.data::<UserService>()?;
        let invitation_service = ctx.data::<InvitationService>()?;

        // Validate and use invitation
        let invitation = invitation_service
            .use_invitation(&input.invitation_token)
            .await
            .map_err(|e| Error::new(format!("Invalid invitation: {}", e)))?;

        // Register user with invitation token
        let (user, access_token, refresh_token) = user_service
            .register_user_with_invitation(
                &invitation.email,
                &input.password,
                input.first_name,
                input.last_name,
                &input.invitation_token,
            )
            .await
            .map_err(|e| Error::new(format!("Registration failed: {}", e)))?;

        Ok(AuthPayload {
            user: user.into(),
            access_token,
            refresh_token,
        })
    }

    async fn register(&self, _ctx: &Context<'_>, _input: RegisterInput) -> Result<User> {
        return Err(Error::new(
            "Public registration is disabled. Please use an invitation link to register."
        ));
    }

    async fn login(&self, ctx: &Context<'_>, input: LoginInput) -> Result<AuthPayload> {
        let user_service = ctx.data::<UserService>()?;

        let (user, access_token, refresh_token) = user_service
            .authenticate_user(&input.email, &input.password)
            .await
            .map_err(|e| Error::new(format!("Authentication failed: {}", e)))?;

        Ok(AuthPayload {
            user: user.into(),
            access_token,
            refresh_token,
        })
    }

    async fn refresh_token(&self, ctx: &Context<'_>, input: RefreshTokenInput) -> Result<AuthPayload> {
        let user_service = ctx.data::<UserService>()?;

        let (user, access_token, refresh_token) = user_service
            .refresh_token(&input.refresh_token)
            .await
            .map_err(|e| Error::new(format!("Token refresh failed: {}", e)))?;

        Ok(AuthPayload {
            user: user.into(),
            access_token,
            refresh_token,
        })
    }

    async fn logout(&self, ctx: &Context<'_>) -> Result<MessageResponse> {
        let user_service = ctx.data::<UserService>()?;
        
        if let Some(auth_user) = ctx.data_opt::<crate::auth::AuthenticatedUser>() {
            user_service
                .revoke_refresh_token(auth_user.id)
                .await
                .map_err(|e| Error::new(format!("Logout failed: {}", e)))?;
        }

        Ok(MessageResponse {
            message: "Logged out successfully".to_string(),
        })
    }

    async fn verify_email(&self, ctx: &Context<'_>, token: String) -> Result<MessageResponse> {
        let user_service = ctx.data::<UserService>()?;

        user_service
            .verify_email(&token)
            .await
            .map_err(|e| Error::new(format!("Email verification failed: {}", e)))?;

        Ok(MessageResponse {
            message: "Email verified successfully".to_string(),
        })
    }

    async fn request_password_reset(&self, ctx: &Context<'_>, input: RequestPasswordResetInput) -> Result<MessageResponse> {
        let user_service = ctx.data::<UserService>()?;
        let email_service = ctx.data::<EmailService>()?;

        let user = user_service
            .request_password_reset(&input.email)
            .await
            .map_err(|e| Error::new(format!("Password reset request failed: {}", e)))?;

        // Send password reset email
        if let Some(reset_token) = &user.password_reset_token {
            let frontend_url = ctx.data::<String>()?;
            if let Err(e) = email_service
                .send_password_reset_email(&user.email, reset_token, frontend_url)
                .await
            {
                eprintln!("Failed to send password reset email: {}", e);
            }
        }

        Ok(MessageResponse {
            message: "Password reset instructions have been sent to your email".to_string(),
        })
    }

    async fn reset_password(&self, ctx: &Context<'_>, input: ResetPasswordInput) -> Result<MessageResponse> {
        let user_service = ctx.data::<UserService>()?;

        user_service
            .reset_password(&input.token, &input.new_password)
            .await
            .map_err(|e| Error::new(format!("Password reset failed: {}", e)))?;

        Ok(MessageResponse {
            message: "Password has been reset successfully".to_string(),
        })
    }

    // Admin-only mutations
    async fn assign_role(&self, ctx: &Context<'_>, input: AssignRoleInput) -> Result<User> {
        require_user_management(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        // Get user
        let user = crate::entities::user::Entity::find_by_id(input.user_id)
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?
            .ok_or_else(|| Error::new("User not found"))?;

        // Verify role exists
        let _role = crate::entities::role::Entity::find_by_id(input.role_id)
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?
            .ok_or_else(|| Error::new("Role not found"))?;

        // Update user role
        let mut user_active: crate::entities::user::ActiveModel = user.into();
        user_active.role_id = Set(Some(input.role_id));
        user_active.updated_at = Set(Utc::now().into());

        let updated_user = user_active
            .update(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to assign role: {}", e)))?;

        Ok(updated_user.into())
    }

    async fn remove_user_role(&self, ctx: &Context<'_>, user_id: uuid::Uuid) -> Result<User> {
        require_user_management(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        // Get user
        let user = crate::entities::user::Entity::find_by_id(user_id)
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?
            .ok_or_else(|| Error::new("User not found"))?;

        // Remove role
        let mut user_active: crate::entities::user::ActiveModel = user.into();
        user_active.role_id = Set(None);
        user_active.updated_at = Set(Utc::now().into());

        let updated_user = user_active
            .update(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to remove role: {}", e)))?;

        Ok(updated_user.into())
    }

    async fn change_password(&self, ctx: &Context<'_>, input: ChangePasswordInput) -> Result<MessageResponse> {
        let user_service = ctx.data::<UserService>()?;
        let auth_user = ctx.data::<crate::auth::AuthenticatedUser>()?;

        user_service
            .change_password(auth_user.id, &input.current_password, &input.new_password)
            .await
            .map_err(|e| Error::new(format!("Password change failed: {}", e)))?;

        Ok(MessageResponse {
            message: "Password changed successfully. You will need to login again.".to_string(),
        })
    }

    async fn admin_reset_user_password(&self, ctx: &Context<'_>, input: AdminResetUserPasswordInput) -> Result<MessageResponse> {
        require_user_management(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        let email_service = ctx.data::<EmailService>()?;
        let auth_user = ctx.data::<crate::auth::AuthenticatedUser>()?;
        let frontend_url = ctx.data::<String>()?;

        // Get the user to reset
        let target_user = user_service
            .find_user_by_id(input.user_id)
            .await
            .map_err(|e| Error::new(format!("Failed to find user: {}", e)))?
            .ok_or_else(|| Error::new("User not found"))?;

        // Get admin user details for email
        let admin_user = user_service
            .find_user_by_id(auth_user.id)
            .await
            .map_err(|e| Error::new(format!("Failed to find admin user: {}", e)))?
            .ok_or_else(|| Error::new("Admin user not found"))?;

        let admin_name = format!("{} {}",
            admin_user.first_name.unwrap_or_else(|| "Admin".to_string()),
            admin_user.last_name.unwrap_or_else(|| "User".to_string())
        ).trim().to_string();

        // Generate reset token
        let updated_user = user_service
            .admin_reset_user_password(input.user_id)
            .await
            .map_err(|e| Error::new(format!("Failed to generate reset token: {}", e)))?;

        // Send password reset email
        if let Some(reset_token) = &updated_user.password_reset_token {
            if let Err(e) = email_service
                .send_admin_password_reset_email(&target_user.email, reset_token, frontend_url, &admin_name)
                .await
            {
                eprintln!("Failed to send admin password reset email: {}", e);
            }
        }

        Ok(MessageResponse {
            message: format!("Password reset email sent to {}", target_user.email),
        })
    }
}