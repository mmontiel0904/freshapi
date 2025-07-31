use async_graphql::*;

use crate::graphql::types::{AcceptInvitationInput, AuthPayload, Invitation, InviteUserInput, LoginInput, MessageResponse, RefreshTokenInput, RegisterInput, RequestPasswordResetInput, ResetPasswordInput, User};
use crate::services::{EmailService, InvitationService, UserService};

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn invite_user(&self, ctx: &Context<'_>, input: InviteUserInput) -> Result<Invitation> {
        let invitation_service = ctx.data::<InvitationService>()?;
        let auth_user = ctx.data::<crate::auth::AuthenticatedUser>()?;
        let frontend_url = ctx.data::<String>()?;

        let invitation = invitation_service
            .create_invitation(auth_user.id, &input.email, frontend_url)
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

    async fn register(&self, ctx: &Context<'_>, input: RegisterInput) -> Result<User> {
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
}