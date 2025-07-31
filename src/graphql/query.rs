use async_graphql::*;

use crate::auth::AuthenticatedUser;
use crate::graphql::types::{Invitation, User};
use crate::services::{InvitationService, UserService};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn me(&self, ctx: &Context<'_>) -> Result<User> {
        let user_service = ctx.data::<UserService>()?;
        let authenticated_user = ctx.data::<AuthenticatedUser>()?;

        let user = user_service
            .find_user_by_id(authenticated_user.id)
            .await
            .map_err(|e| Error::new(format!("Failed to find user: {}", e)))?
            .ok_or_else(|| Error::new("User not found"))?;

        Ok(user.into())
    }

    async fn health(&self) -> &str {
        "OK"
    }

    async fn my_invitations(&self, ctx: &Context<'_>) -> Result<Vec<Invitation>> {
        let invitation_service = ctx.data::<InvitationService>()?;
        let authenticated_user = ctx.data::<AuthenticatedUser>()?;

        let invitations = invitation_service
            .get_invitations_by_user(authenticated_user.id)
            .await
            .map_err(|e| Error::new(format!("Failed to fetch invitations: {}", e)))?;

        Ok(invitations.into_iter().map(|inv| inv.into()).collect())
    }
}