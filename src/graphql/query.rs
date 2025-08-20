use async_graphql::*;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use crate::auth::{AuthenticatedUser, PermissionService, require_admin};
use crate::graphql::types::{Invitation, User, Role, RoleWithPermissions, Permission, Resource, UserWithRole, Project, Task, TaskStats};
use crate::graphql::DataLoaderContext;
use crate::services::{InvitationService, UserService, ProjectService, TaskService, ActivityService};
use crate::services::activity::EntityType;
use crate::graphql::types::{TaskStatus, Activity, GraphQLEntityType};

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
            // DataLoader automatically batches all these calls into single database query - GET ALL PERMISSIONS
            let permissions = dataloader
                .load_user_all_permissions(user.id)
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
            .get_user_all_permissions(user_id)
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

        // Use DataLoader for caching (beneficial if called multiple times) - GET ALL PERMISSIONS
        let permissions = dataloader
            .load_user_all_permissions(user.id)
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
            // DataLoader automatically batches all these calls into single database query - GET ALL PERMISSIONS
            let permissions = dataloader
                .load_user_all_permissions(user.id)
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
        require_permission(ctx, "task_system", "project_read").await?;
        
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
        require_permission(ctx, "task_system", "project_read").await?;
        
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
        require_permission(ctx, "task_system", "task_read").await?;
        
        let task_service = ctx.data::<TaskService>()?;
        let authenticated_user = ctx.data::<AuthenticatedUser>()?;
        
        let task = task_service
            .get_task(task_id, authenticated_user.id)
            .await
            .map_err(|e| Error::new(format!("Failed to fetch task: {}", e)))?;
            
        Ok(task.map(|t| t.into()))
    }

    async fn my_assigned_tasks(&self, ctx: &Context<'_>, status: Option<TaskStatus>, limit: Option<i32>, offset: Option<i32>) -> Result<Vec<Task>> {
        use crate::auth::require_permission;
        require_permission(ctx, "task_system", "task_read").await?;
        
        let task_service = ctx.data::<TaskService>()?;
        let authenticated_user = ctx.data::<AuthenticatedUser>()?;
        
        let status_filter = status;
        
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

    async fn project_tasks(&self, ctx: &Context<'_>, project_id: uuid::Uuid, status: Option<TaskStatus>, assignee_id: Option<uuid::Uuid>, limit: Option<i32>, offset: Option<i32>) -> Result<Vec<Task>> {
        use crate::auth::require_permission;
        require_permission(ctx, "task_system", "task_read").await?;
        
        let task_service = ctx.data::<TaskService>()?;
        let authenticated_user = ctx.data::<AuthenticatedUser>()?;
        
        let status_filter = status;
        
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
        require_permission(ctx, "task_system", "task_read").await?;
        
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

    // RBAC CRUD Queries - Admin only
    async fn all_roles_with_permissions(&self, ctx: &Context<'_>) -> Result<Vec<RoleWithPermissions>> {
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        let roles = crate::entities::role::Entity::find()
            .filter(crate::entities::role::Column::IsActive.eq(true))
            .order_by_asc(crate::entities::role::Column::Level)
            .all(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to fetch roles: {}", e)))?;

        Ok(roles.into_iter().map(|role| role.into()).collect())
    }

    async fn role_by_id(&self, ctx: &Context<'_>, role_id: uuid::Uuid) -> Result<Option<RoleWithPermissions>> {
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        let role = crate::entities::role::Entity::find_by_id(role_id)
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to fetch role: {}", e)))?;

        Ok(role.map(|r| r.into()))
    }

    async fn all_permissions(&self, ctx: &Context<'_>, resource_id: Option<uuid::Uuid>) -> Result<Vec<Permission>> {
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        let mut query = crate::entities::permission::Entity::find()
            .filter(crate::entities::permission::Column::IsActive.eq(true));
            
        if let Some(resource_id) = resource_id {
            query = query.filter(crate::entities::permission::Column::ResourceId.eq(resource_id));
        }
        
        let permissions = query
            .order_by_asc(crate::entities::permission::Column::Action)
            .all(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to fetch permissions: {}", e)))?;

        Ok(permissions.into_iter().map(|p| p.into()).collect())
    }

    async fn permission_by_id(&self, ctx: &Context<'_>, permission_id: uuid::Uuid) -> Result<Option<Permission>> {
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        let permission = crate::entities::permission::Entity::find_by_id(permission_id)
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to fetch permission: {}", e)))?;

        Ok(permission.map(|p| p.into()))
    }

    async fn all_resources(&self, ctx: &Context<'_>) -> Result<Vec<Resource>> {
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        let resources = crate::entities::resource::Entity::find()
            .filter(crate::entities::resource::Column::IsActive.eq(true))
            .order_by_asc(crate::entities::resource::Column::Name)
            .all(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to fetch resources: {}", e)))?;

        Ok(resources.into_iter().map(|r| r.into()).collect())
    }

    async fn resource_by_id(&self, ctx: &Context<'_>, resource_id: uuid::Uuid) -> Result<Option<Resource>> {
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        let resource = crate::entities::resource::Entity::find_by_id(resource_id)
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to fetch resource: {}", e)))?;

        Ok(resource.map(|r| r.into()))
    }

    async fn role_permissions(&self, ctx: &Context<'_>, role_id: uuid::Uuid) -> Result<Vec<Permission>> {
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        use sea_orm::{JoinType, QuerySelect, RelationTrait};
        
        let permissions = crate::entities::role_permission::Entity::find()
            .filter(crate::entities::role_permission::Column::RoleId.eq(role_id))
            .join(JoinType::InnerJoin, crate::entities::role_permission::Relation::Permission.def())
            .select_only()
            .column_as(crate::entities::permission::Column::Id, "id")
            .column_as(crate::entities::permission::Column::Action, "action")
            .column_as(crate::entities::permission::Column::ResourceId, "resource_id")
            .column_as(crate::entities::permission::Column::Description, "description")
            .column_as(crate::entities::permission::Column::IsActive, "is_active")
            .column_as(crate::entities::permission::Column::CreatedAt, "created_at")
            .column_as(crate::entities::permission::Column::UpdatedAt, "updated_at")
            .into_model::<crate::entities::permission::Model>()
            .all(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to fetch role permissions: {}", e)))?;

        Ok(permissions.into_iter().map(|p| p.into()).collect())
    }

    async fn user_direct_permissions(&self, ctx: &Context<'_>, user_id: uuid::Uuid) -> Result<Vec<Permission>> {
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        use sea_orm::{JoinType, QuerySelect, RelationTrait};
        
        let permissions = crate::entities::user_permission::Entity::find()
            .filter(crate::entities::user_permission::Column::UserId.eq(user_id))
            .filter(crate::entities::user_permission::Column::IsGranted.eq(true))
            .join(JoinType::InnerJoin, crate::entities::user_permission::Relation::Permission.def())
            .select_only()
            .column_as(crate::entities::permission::Column::Id, "id")
            .column_as(crate::entities::permission::Column::Action, "action")
            .column_as(crate::entities::permission::Column::ResourceId, "resource_id")
            .column_as(crate::entities::permission::Column::Description, "description")
            .column_as(crate::entities::permission::Column::IsActive, "is_active")
            .column_as(crate::entities::permission::Column::CreatedAt, "created_at")
            .column_as(crate::entities::permission::Column::UpdatedAt, "updated_at")
            .into_model::<crate::entities::permission::Model>()
            .all(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to fetch user permissions: {}", e)))?;

        Ok(permissions.into_iter().map(|p| p.into()).collect())
    }

    // Generic activities query
    async fn activities(
        &self, 
        ctx: &Context<'_>, 
        entity_type: GraphQLEntityType,
        entity_id: Uuid,
        limit: Option<i32>,
        offset: Option<i32>
    ) -> Result<Vec<Activity>> {
        let auth_user = ctx.data::<AuthenticatedUser>()?;

        // Convert GraphQLEntityType to EntityType for ActivityService
        let entity_type_enum = match entity_type {
            GraphQLEntityType::Task => EntityType::Task,
            GraphQLEntityType::Project => EntityType::Project,
            GraphQLEntityType::User => EntityType::User,
            GraphQLEntityType::Settings => EntityType::Settings,
        };

        // Verify user can access the entity they want to view activities for
        match entity_type_enum {
            EntityType::Task => {
                let task_service = ctx.data::<TaskService>()?;
                let can_access = task_service
                    .can_user_access_task(entity_id, auth_user.id)
                    .await
                    .map_err(|e| Error::new(format!("Failed to check task access: {}", e)))?;
                
                if !can_access {
                    return Err(Error::new("You don't have permission to view activities for this task"));
                }
            },
            EntityType::Project => {
                let project_service = ctx.data::<ProjectService>()?;
                let can_access = project_service
                    .can_user_access_project(entity_id, auth_user.id)
                    .await
                    .map_err(|e| Error::new(format!("Failed to check project access: {}", e)))?;
                
                if !can_access {
                    return Err(Error::new("You don't have permission to view activities for this project"));
                }
            },
            EntityType::User => {
                // Users can view activities on user profiles if they have user_management permission
                use crate::auth::require_permission;
                require_permission(ctx, "freshapi", "user_management").await?;
            },
            EntityType::Settings => {
                // Only admins can view activities on settings
                use crate::auth::require_admin;
                require_admin(ctx, "freshapi").await?;
            },
        }

        let activity_service = ctx.data::<ActivityService>()?;

        let activities = activity_service
            .get_entity_activities(
                entity_type_enum,
                entity_id,
                limit.map(|l| l.max(0) as u64),
                offset.map(|o| o.max(0) as u64),
            )
            .await
            .map_err(|e| Error::new(format!("Failed to fetch activities: {}", e)))?;

        Ok(activities.into_iter().map(|a| a.into()).collect())
    }

    // ============================================================================
    // ProjectMind Context System Queries
    // ============================================================================

    /// Get all available context types
    async fn context_types(&self, ctx: &Context<'_>, active_only: Option<bool>) -> Result<Vec<crate::graphql::types::ContextType>> {
        let context_service = ctx.data::<crate::services::ContextService>()?;
        
        let types = context_service
            .get_context_types(active_only.unwrap_or(true))
            .await
            .map_err(|e| Error::new(format!("Failed to fetch context types: {}", e)))?;

        Ok(types.into_iter().map(Into::into).collect())
    }

    /// Get project context categories
    async fn project_context_categories(
        &self,
        ctx: &Context<'_>,
        project_id: Uuid,
        context_type_name: Option<String>,
    ) -> Result<Vec<crate::graphql::types::ProjectContextCategory>> {
        let context_service = ctx.data::<crate::services::ContextService>()?;
        let _authenticated_user = ctx.data::<AuthenticatedUser>()?;
        
        let categories = context_service
            .get_project_categories(project_id, context_type_name)
            .await
            .map_err(|e| Error::new(format!("Failed to fetch categories: {}", e)))?;

        Ok(categories.into_iter().map(Into::into).collect())
    }

    /// Get project contexts with filtering and pagination
    async fn project_contexts(
        &self,
        ctx: &Context<'_>,
        project_id: Uuid,
        filters: Option<crate::graphql::types::ContextFilters>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<crate::graphql::types::ContextConnection> {
        let context_service = ctx.data::<crate::services::ContextService>()?;
        let _authenticated_user = ctx.data::<AuthenticatedUser>()?;
        
        let result = context_service
            .get_project_contexts(
                project_id,
                filters,
                limit.map(|l| l.max(0) as u64),
                offset.map(|o| o.max(0) as u64),
            )
            .await
            .map_err(|e| Error::new(format!("Failed to fetch project contexts: {}", e)))?;

        Ok(result)
    }

    /// Get email contexts with filtering and pagination
    async fn email_contexts(
        &self,
        ctx: &Context<'_>,
        project_id: Uuid,
        filters: Option<crate::graphql::types::EmailContextFilters>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<crate::graphql::types::EmailContextConnection> {
        let email_service = ctx.data::<crate::services::EmailContextService>()?;
        let _authenticated_user = ctx.data::<AuthenticatedUser>()?;
        
        let result = email_service
            .get_email_contexts(
                project_id,
                filters,
                limit.map(|l| l.max(0) as u64),
                offset.map(|o| o.max(0) as u64),
            )
            .await
            .map_err(|e| Error::new(format!("Failed to fetch email contexts: {}", e)))?;

        Ok(result)
    }

    /// Get single email context by ID
    async fn email_context(&self, ctx: &Context<'_>, email_id: Uuid) -> Result<Option<crate::graphql::types::EmailContext>> {
        let email_service = ctx.data::<crate::services::EmailContextService>()?;
        let _authenticated_user = ctx.data::<AuthenticatedUser>()?;
        
        let email = email_service
            .get_email_context_by_id(email_id)
            .await
            .map_err(|e| Error::new(format!("Failed to fetch email context: {}", e)))?;

        Ok(email.map(Into::into))
    }

    /// Search email contexts with full-text search
    async fn search_email_contexts(
        &self,
        ctx: &Context<'_>,
        project_id: Uuid,
        query: String,
        limit: Option<i32>,
    ) -> Result<Vec<crate::graphql::types::EmailContext>> {
        let email_service = ctx.data::<crate::services::EmailContextService>()?;
        let _authenticated_user = ctx.data::<AuthenticatedUser>()?;
        
        let results = email_service
            .search_emails(
                project_id,
                &query,
                limit.map(|l| l.max(0) as u64),
            )
            .await
            .map_err(|e| Error::new(format!("Failed to search email contexts: {}", e)))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    /// Get email thread by thread ID
    async fn email_thread(
        &self,
        ctx: &Context<'_>,
        project_id: Uuid,
        thread_id: String,
    ) -> Result<Vec<crate::graphql::types::EmailContext>> {
        let email_service = ctx.data::<crate::services::EmailContextService>()?;
        let _authenticated_user = ctx.data::<AuthenticatedUser>()?;
        
        let emails = email_service
            .get_email_thread(&thread_id, project_id)
            .await
            .map_err(|e| Error::new(format!("Failed to fetch email thread: {}", e)))?;

        Ok(emails.into_iter().map(Into::into).collect())
    }
}