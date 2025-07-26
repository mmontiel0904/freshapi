use async_graphql::*;

use crate::auth::AuthenticatedUser;
use crate::graphql::types::User;
use crate::services::UserService;

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
}