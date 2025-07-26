use async_graphql::*;

use crate::graphql::types::{AuthPayload, LoginInput, MessageResponse, RegisterInput, User};
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

        let (user, token) = user_service
            .authenticate_user(&input.email, &input.password)
            .await
            .map_err(|e| Error::new(format!("Authentication failed: {}", e)))?;

        Ok(AuthPayload {
            user: user.into(),
            token,
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