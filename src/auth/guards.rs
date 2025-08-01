use async_graphql::{Context, Error, Result};
use crate::auth::{AuthenticatedUser, PermissionService};

/// Authorization guard for checking if user is authenticated
pub fn require_auth<'ctx>(ctx: &'ctx Context<'_>) -> Result<&'ctx AuthenticatedUser> {
    ctx.data::<AuthenticatedUser>()
        .map_err(|_| Error::new("Authentication required"))
}

/// Authorization guard for checking specific permission
pub async fn require_permission<'ctx>(
    ctx: &'ctx Context<'_>,
    resource: &str,
    action: &str,
) -> Result<&'ctx AuthenticatedUser> {
    let user = require_auth(ctx)?;
    let permission_service = ctx.data::<PermissionService>()?;
    
    let has_permission = permission_service
        .user_has_permission(user.id, resource, action)
        .await
        .map_err(|e| Error::new(format!("Permission check failed: {}", e)))?;
    
    if !has_permission {
        return Err(Error::new(format!(
            "Insufficient permissions: {} required for {}",
            action, resource
        )));
    }
    
    Ok(user)
}

/// Authorization guard for admin permissions
pub async fn require_admin<'ctx>(ctx: &'ctx Context<'_>, resource: &str) -> Result<&'ctx AuthenticatedUser> {
    require_permission(ctx, resource, "admin").await
}

/// Authorization guard for system admin permissions
pub async fn require_system_admin<'ctx>(ctx: &'ctx Context<'_>, resource: &str) -> Result<&'ctx AuthenticatedUser> {
    require_permission(ctx, resource, "system_admin").await
}

/// Authorization guard for user management permissions
pub async fn require_user_management<'ctx>(ctx: &'ctx Context<'_>, resource: &str) -> Result<&'ctx AuthenticatedUser> {
    require_permission(ctx, resource, "user_management").await
}

/// Authorization guard for checking if user can manage another user
pub async fn require_user_can_manage<'ctx>(
    ctx: &'ctx Context<'_>,
    target_user_id: uuid::Uuid,
) -> Result<&'ctx AuthenticatedUser> {
    let user = require_auth(ctx)?;
    let permission_service = ctx.data::<PermissionService>()?;
    
    let can_manage = permission_service
        .user_can_manage_user(user.id, target_user_id)
        .await
        .map_err(|e| Error::new(format!("User management check failed: {}", e)))?;
    
    if !can_manage {
        return Err(Error::new("Insufficient permissions to manage this user"));
    }
    
    Ok(user)
}

/// Macro for creating permission-based guards
#[macro_export]
macro_rules! permission_guard {
    ($resource:expr, $action:expr) => {
        |ctx: &async_graphql::Context<'_>| async move {
            $crate::auth::guards::require_permission(ctx, $resource, $action).await
        }
    };
}

/// Macro for creating resource admin guards
#[macro_export]
macro_rules! admin_guard {
    ($resource:expr) => {
        |ctx: &async_graphql::Context<'_>| async move {
            $crate::auth::guards::require_admin(ctx, $resource).await
        }
    };
}

pub use crate::{permission_guard, admin_guard};