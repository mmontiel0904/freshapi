use async_graphql::*;

use crate::graphql::types::{AuthPayload, LoginInput, MessageResponse, RefreshTokenInput, RegisterInput, User};
use crate::services::{EmailService, UserService};

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn register(&self, ctx: &Context<'_>, input: RegisterInput) -> Result<User> {
        let user_service = ctx.data::<UserService>()?;
        let email_service = ctx.data::<EmailService>()?;

        let user = user_service
            .register_user(
                &input.email,
                &input.password,
                input.first_name,
                input.last_name,
            )
            .await
            .map_err(|e| Error::new(format!("Registration failed: {}", e)))?;

        // Send verification email
        if let Some(token) = &user.email_verification_token {
            if let Err(e) = email_service
                .send_verification_email(&user.email, token, "http://localhost:8080")
                .await
            {
                // Log error but don't fail registration
                eprintln!("Failed to send verification email: {}", e);
            }
        }

        Ok(user.into())
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
}