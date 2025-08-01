use async_graphql::*;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};

use crate::auth::{AuthenticatedUser, PermissionService, require_admin};
use crate::graphql::types::{Invitation, User, Role, UserWithRole};
use crate::services::{InvitationService, UserService};
use crate::entities::prelude::*;

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

    // Admin-only queries
    async fn all_users(&self, ctx: &Context<'_>) -> Result<Vec<UserWithRole>> {
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        let permission_service = ctx.data::<PermissionService>()?;
        
        // Get all users with their roles
        let users_with_roles = crate::entities::user::Entity::find()
            .find_also_related(crate::entities::role::Entity)
            .all(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to fetch users: {}", e)))?;

        let mut result = Vec::new();
        
        for (user, role_opt) in users_with_roles {
            let permissions = permission_service
                .get_user_permissions(user.id, "freshapi")
                .await
                .map_err(|e| Error::new(format!("Failed to get permissions: {}", e)))?;

            result.push(UserWithRole {
                id: user.id,
                email: user.email,
                first_name: user.first_name,
                last_name: user.last_name,
                is_email_verified: user.is_email_verified,
                role: role_opt.map(|r| r.into()),
                permissions,
                created_at: user.created_at.into(),
                updated_at: user.updated_at.into(),
            });
        }

        Ok(result)
    }

    async fn all_roles(&self, ctx: &Context<'_>) -> Result<Vec<Role>> {
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        let roles = crate::entities::role::Entity::find()
            .filter(crate::entities::role::Column::IsActive.eq(true))
            .all(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to fetch roles: {}", e)))?;

        Ok(roles.into_iter().map(|role| role.into()).collect())
    }

    async fn user_permissions(&self, ctx: &Context<'_>, user_id: uuid::Uuid) -> Result<Vec<String>> {
        require_admin(ctx, "freshapi").await?;
        
        let permission_service = ctx.data::<PermissionService>()?;
        
        let permissions = permission_service
            .get_user_permissions(user_id, "freshapi")
            .await
            .map_err(|e| Error::new(format!("Failed to get permissions: {}", e)))?;

        Ok(permissions)
    }
}