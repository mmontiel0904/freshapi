use async_graphql::*;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};

use crate::auth::{AuthenticatedUser, PermissionService, require_admin};
use crate::graphql::types::{Invitation, User, Role, UserWithRole, Project, Task, TaskStats};
use crate::graphql::DataLoaderContext;
use crate::services::{InvitationService, UserService, ProjectService, TaskService, TaskStatus};

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
        use crate::auth::require_permission;
        require_permission(ctx, "freshapi", "invite_users").await?;
        
        let invitation_service = ctx.data::<InvitationService>()?;
        let authenticated_user = ctx.data::<AuthenticatedUser>()?;

        let invitations = invitation_service
            .get_invitations_by_user(authenticated_user.id)
            .await
            .map_err(|e| Error::new(format!("Failed to fetch invitations: {}", e)))?;

        Ok(invitations.into_iter().map(|inv| inv.into()).collect())
    }

    // Admin-only queries - OPTIMIZED with DataLoader (automatic batching + caching)
    async fn all_users(&self, ctx: &Context<'_>) -> Result<Vec<UserWithRole>> {
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        let dataloader = ctx.data::<DataLoaderContext>()?;
        
        // Get all users with their roles
        let users_with_roles = crate::entities::user::Entity::find()
            .find_also_related(crate::entities::role::Entity)
            .all(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to fetch users: {}", e)))?;

        // Use DataLoader for automatic batching and caching
        let mut result = Vec::new();
        
        for (user, role_opt) in users_with_roles {
            // DataLoader automatically batches all these calls into single database query
            let permissions = dataloader
                .load_user_permissions(user.id)
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

    async fn user_by_id(&self, ctx: &Context<'_>, user_id: uuid::Uuid) -> Result<UserWithRole> {
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        let dataloader = ctx.data::<DataLoaderContext>()?;
        
        // Get user with role
        let (user, role_opt) = crate::entities::user::Entity::find_by_id(user_id)
            .find_also_related(crate::entities::role::Entity)
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to fetch user: {}", e)))?
            .ok_or_else(|| Error::new("User not found"))?;

        // Use DataLoader for caching (beneficial if called multiple times)
        let permissions = dataloader
            .load_user_permissions(user.id)
            .await
            .map_err(|e| Error::new(format!("Failed to get permissions: {}", e)))?;

        Ok(UserWithRole {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            is_email_verified: user.is_email_verified,
            role: role_opt.map(|r| r.into()),
            permissions,
            created_at: user.created_at.into(),
            updated_at: user.updated_at.into(),
        })
    }

    async fn users_by_role(&self, ctx: &Context<'_>, role_name: String) -> Result<Vec<UserWithRole>> {
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        let dataloader = ctx.data::<DataLoaderContext>()?;
        
        // Find role by name
        let role = crate::entities::role::Entity::find()
            .filter(crate::entities::role::Column::Name.eq(&role_name))
            .filter(crate::entities::role::Column::IsActive.eq(true))
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to find role: {}", e)))?
            .ok_or_else(|| Error::new("Role not found"))?;

        // Get users with this role
        let users_with_roles = crate::entities::user::Entity::find()
            .filter(crate::entities::user::Column::RoleId.eq(role.id))
            .find_also_related(crate::entities::role::Entity)
            .all(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to fetch users: {}", e)))?;

        // Use DataLoader for automatic batching and caching
        let mut result = Vec::new();
        
        for (user, role_opt) in users_with_roles {
            // DataLoader automatically batches all these calls into single database query
            let permissions = dataloader
                .load_user_permissions(user.id)
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

    // Project queries
    async fn my_projects(&self, ctx: &Context<'_>, limit: Option<i32>, offset: Option<i32>) -> Result<Vec<Project>> {
        use crate::auth::require_permission;
        require_permission(ctx, "task_system", "read").await?;
        
        let project_service = ctx.data::<ProjectService>()?;
        let authenticated_user = ctx.data::<AuthenticatedUser>()?;
        
        let projects = project_service
            .get_user_projects(
                authenticated_user.id,
                limit.map(|l| l.max(0) as u64),
                offset.map(|o| o.max(0) as u64),
            )
            .await
            .map_err(|e| Error::new(format!("Failed to fetch projects: {}", e)))?;
            
        Ok(projects.into_iter().map(|p| p.into()).collect())
    }

    async fn project(&self, ctx: &Context<'_>, project_id: uuid::Uuid) -> Result<Option<Project>> {
        use crate::auth::require_permission;
        require_permission(ctx, "task_system", "read").await?;
        
        let project_service = ctx.data::<ProjectService>()?;
        let authenticated_user = ctx.data::<AuthenticatedUser>()?;
        
        let project = project_service
            .get_project(project_id, authenticated_user.id)
            .await
            .map_err(|e| Error::new(format!("Failed to fetch project: {}", e)))?;
            
        Ok(project.map(|p| p.into()))
    }

    // Task queries
    async fn task(&self, ctx: &Context<'_>, task_id: uuid::Uuid) -> Result<Option<Task>> {
        use crate::auth::require_permission;
        require_permission(ctx, "task_system", "read").await?;
        
        let task_service = ctx.data::<TaskService>()?;
        let authenticated_user = ctx.data::<AuthenticatedUser>()?;
        
        let task = task_service
            .get_task(task_id, authenticated_user.id)
            .await
            .map_err(|e| Error::new(format!("Failed to fetch task: {}", e)))?;
            
        Ok(task.map(|t| t.into()))
    }

    async fn my_assigned_tasks(&self, ctx: &Context<'_>, status: Option<String>, limit: Option<i32>, offset: Option<i32>) -> Result<Vec<Task>> {
        use crate::auth::require_permission;
        require_permission(ctx, "task_system", "read").await?;
        
        let task_service = ctx.data::<TaskService>()?;
        let authenticated_user = ctx.data::<AuthenticatedUser>()?;
        
        let status_filter = status.and_then(|s| TaskStatus::from_str(&s));
        
        let tasks = task_service
            .get_user_assigned_tasks(
                authenticated_user.id,
                status_filter,
                limit.map(|l| l.max(0) as u64),
                offset.map(|o| o.max(0) as u64),
            )
            .await
            .map_err(|e| Error::new(format!("Failed to fetch assigned tasks: {}", e)))?;
            
        Ok(tasks.into_iter().map(|t| t.into()).collect())
    }

    async fn project_tasks(&self, ctx: &Context<'_>, project_id: uuid::Uuid, status: Option<String>, assignee_id: Option<uuid::Uuid>, limit: Option<i32>, offset: Option<i32>) -> Result<Vec<Task>> {
        use crate::auth::require_permission;
        require_permission(ctx, "task_system", "read").await?;
        
        let task_service = ctx.data::<TaskService>()?;
        let authenticated_user = ctx.data::<AuthenticatedUser>()?;
        
        let status_filter = status.and_then(|s| TaskStatus::from_str(&s));
        
        let tasks = task_service
            .get_project_tasks(
                project_id,
                authenticated_user.id,
                status_filter,
                assignee_id,
                limit.map(|l| l.max(0) as u64),
                offset.map(|o| o.max(0) as u64),
            )
            .await
            .map_err(|e| Error::new(format!("Failed to fetch project tasks: {}", e)))?;
            
        Ok(tasks.into_iter().map(|t| t.into()).collect())
    }

    async fn project_task_stats(&self, ctx: &Context<'_>, project_id: uuid::Uuid) -> Result<TaskStats> {
        use crate::auth::require_permission;
        require_permission(ctx, "task_system", "read").await?;
        
        let task_service = ctx.data::<TaskService>()?;
        let authenticated_user = ctx.data::<AuthenticatedUser>()?;
        
        let stats = task_service
            .get_project_task_stats(project_id, authenticated_user.id)
            .await
            .map_err(|e| Error::new(format!("Failed to fetch task stats: {}", e)))?;
            
        Ok(TaskStats {
            total: stats.total,
            todo: stats.todo,
            in_progress: stats.in_progress,
            completed: stats.completed,
            cancelled: stats.cancelled,
            overdue: stats.overdue,
        })
    }
}