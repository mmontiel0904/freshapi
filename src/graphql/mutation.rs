use async_graphql::*;
use sea_orm::{EntityTrait, ActiveModelTrait, Set, ColumnTrait, QueryFilter, PaginatorTrait};
use chrono::Utc;
use uuid::Uuid;

use crate::auth::{require_user_management, AuthenticatedUser};
use crate::graphql::types::{AcceptInvitationInput, AdminResetUserPasswordInput, AuthPayload, ChangePasswordInput, Invitation, InviteUserInput, InviteUserWithRoleInput, LoginInput, MessageResponse, RefreshTokenInput, RegisterInput, RequestPasswordResetInput, ResetPasswordInput, User, AssignRoleInput, Project, Task, CreateProjectInput, UpdateProjectInput, AddProjectMemberInput, UpdateMemberRoleInput, RemoveProjectMemberInput, CreateTaskInput, UpdateTaskInput, AssignTaskInput, Role, Permission, Resource, CreateRoleInput, UpdateRoleInput, CreatePermissionInput, UpdatePermissionInput, CreateResourceInput, UpdateResourceInput, AssignPermissionToRoleInput, RemovePermissionFromRoleInput, GrantUserPermissionInput, RevokeUserPermissionInput, AddCommentInput, Activity, GraphQLEntityType, CompleteTaskWithRecurrenceResponse};
use crate::services::{EmailService, InvitationService, UserService, ProjectService, TaskService, ProjectRole, ActivityService};
use crate::services::activity::EntityType;
// Task enums imported when needed

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

        // Validate invitation without marking as used
        let _invitation = invitation_service
            .validate_invitation_token(&input.invitation_token)
            .await
            .map_err(|e| Error::new(format!("Invalid invitation: {}", e)))?;

        // Register user with invitation token (this will mark invitation as used within transaction)
        let (user, access_token, refresh_token) = user_service
            .register_user_with_invitation(
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

    // Project mutations
    async fn create_project(&self, ctx: &Context<'_>, input: CreateProjectInput) -> Result<Project> {
        use crate::auth::require_permission;
        require_permission(ctx, "task_system", "project_create").await?;
        
        let project_service = ctx.data::<ProjectService>()?;
        let authenticated_user = ctx.data::<crate::auth::AuthenticatedUser>()?;
        
        let project = project_service
            .create_project(authenticated_user.id, &input.name, input.description)
            .await
            .map_err(|e| Error::new(format!("Failed to create project: {}", e)))?;
            
        Ok(project.into())
    }

    async fn update_project(&self, ctx: &Context<'_>, input: UpdateProjectInput) -> Result<Project> {
        use crate::auth::require_permission;
        require_permission(ctx, "task_system", "task_write").await?;
        
        let project_service = ctx.data::<ProjectService>()?;
        let authenticated_user = ctx.data::<crate::auth::AuthenticatedUser>()?;
        
        let project = project_service
            .update_project(input.project_id, authenticated_user.id, input.name, input.description)
            .await
            .map_err(|e| Error::new(format!("Failed to update project: {}", e)))?;
            
        Ok(project.into())
    }

    async fn delete_project(&self, ctx: &Context<'_>, project_id: uuid::Uuid) -> Result<MessageResponse> {
        use crate::auth::require_permission;
        require_permission(ctx, "task_system", "project_admin").await?;
        
        let project_service = ctx.data::<ProjectService>()?;
        let authenticated_user = ctx.data::<crate::auth::AuthenticatedUser>()?;
        
        project_service
            .delete_project(project_id, authenticated_user.id)
            .await
            .map_err(|e| Error::new(format!("Failed to delete project: {}", e)))?;
            
        Ok(MessageResponse {
            message: "Project deleted successfully".to_string(),
        })
    }

    async fn add_project_member(&self, ctx: &Context<'_>, input: AddProjectMemberInput) -> Result<MessageResponse> {
        use crate::auth::require_permission;
        require_permission(ctx, "task_system", "project_invite").await?;
        
        let project_service = ctx.data::<ProjectService>()?;
        let authenticated_user = ctx.data::<crate::auth::AuthenticatedUser>()?;
        
        let role = ProjectRole::from_str(&input.role)
            .ok_or_else(|| Error::new("Invalid project role"))?;
        
        project_service
            .add_project_member(input.project_id, authenticated_user.id, input.user_id, role)
            .await
            .map_err(|e| Error::new(format!("Failed to add project member: {}", e)))?;
            
        Ok(MessageResponse {
            message: "Project member added successfully".to_string(),
        })
    }

    async fn update_member_role(&self, ctx: &Context<'_>, input: UpdateMemberRoleInput) -> Result<MessageResponse> {
        use crate::auth::require_permission;
        require_permission(ctx, "task_system", "project_admin").await?;
        
        let project_service = ctx.data::<ProjectService>()?;
        let authenticated_user = ctx.data::<crate::auth::AuthenticatedUser>()?;
        
        let role = ProjectRole::from_str(&input.role)
            .ok_or_else(|| Error::new("Invalid project role"))?;
        
        project_service
            .update_member_role(input.project_id, authenticated_user.id, input.user_id, role)
            .await
            .map_err(|e| Error::new(format!("Failed to update member role: {}", e)))?;
            
        Ok(MessageResponse {
            message: "Member role updated successfully".to_string(),
        })
    }

    async fn remove_project_member(&self, ctx: &Context<'_>, input: RemoveProjectMemberInput) -> Result<MessageResponse> {
        use crate::auth::require_permission;
        require_permission(ctx, "task_system", "project_admin").await?;
        
        let project_service = ctx.data::<ProjectService>()?;
        let authenticated_user = ctx.data::<crate::auth::AuthenticatedUser>()?;
        
        project_service
            .remove_project_member(input.project_id, authenticated_user.id, input.user_id)
            .await
            .map_err(|e| Error::new(format!("Failed to remove project member: {}", e)))?;
            
        Ok(MessageResponse {
            message: "Project member removed successfully".to_string(),
        })
    }

    // Task mutations
    async fn create_task(&self, ctx: &Context<'_>, input: CreateTaskInput) -> Result<Task> {
        use crate::auth::require_permission;
        require_permission(ctx, "task_system", "task_create").await?;
        
        let task_service = ctx.data::<TaskService>()?;
        let authenticated_user = ctx.data::<crate::auth::AuthenticatedUser>()?;
        
        let task = task_service
            .create_task(
                input.project_id,
                authenticated_user.id,
                &input.name,
                input.description,
                input.assignee_id,
                input.priority,
                input.recurrence_type,
                input.recurrence_day,
                input.due_date,
            )
            .await
            .map_err(|e| Error::new(format!("Failed to create task: {}", e)))?;
            
        Ok(task.into())
    }

    async fn update_task(&self, ctx: &Context<'_>, input: UpdateTaskInput) -> Result<Task> {
        use crate::auth::require_permission;
        require_permission(ctx, "task_system", "task_write").await?;
        
        let task_service = ctx.data::<TaskService>()?;
        let authenticated_user = ctx.data::<crate::auth::AuthenticatedUser>()?;
        
        let task = task_service
            .update_task(
                input.task_id,
                authenticated_user.id,
                input.name,
                input.description,
                input.status,
                input.priority,
                input.recurrence_type,
                input.recurrence_day,
                input.due_date,
            )
            .await
            .map_err(|e| Error::new(format!("Failed to update task: {}", e)))?;
            
        Ok(task.into())
    }

    async fn assign_task(&self, ctx: &Context<'_>, input: AssignTaskInput) -> Result<Task> {
        use crate::auth::require_permission;
        require_permission(ctx, "task_system", "task_assign").await?;
        
        let task_service = ctx.data::<TaskService>()?;
        let authenticated_user = ctx.data::<crate::auth::AuthenticatedUser>()?;
        
        let task = task_service
            .assign_task(input.task_id, authenticated_user.id, input.assignee_id)
            .await
            .map_err(|e| Error::new(format!("Failed to assign task: {}", e)))?;
            
        Ok(task.into())
    }

    async fn delete_task(&self, ctx: &Context<'_>, task_id: uuid::Uuid) -> Result<MessageResponse> {
        use crate::auth::require_permission;
        require_permission(ctx, "task_system", "task_delete").await?;
        
        let task_service = ctx.data::<TaskService>()?;
        let authenticated_user = ctx.data::<crate::auth::AuthenticatedUser>()?;
        
        task_service
            .delete_task(task_id, authenticated_user.id)
            .await
            .map_err(|e| Error::new(format!("Failed to delete task: {}", e)))?;
            
        Ok(MessageResponse {
            message: "Task deleted successfully".to_string(),
        })
    }

    async fn complete_task_with_recurrence(&self, ctx: &Context<'_>, task_id: uuid::Uuid) -> Result<CompleteTaskWithRecurrenceResponse> {
        use crate::auth::require_permission;
        require_permission(ctx, "task_system", "task_write").await?;
        
        let task_service = ctx.data::<TaskService>()?;
        let authenticated_user = ctx.data::<crate::auth::AuthenticatedUser>()?;
        
        // Complete the task and get both the completed task and next instance (if recurring)
        let (completed_task, next_instance) = task_service
            .complete_task_with_recurrence(task_id, authenticated_user.id)
            .await
            .map_err(|e| Error::new(format!("Failed to complete recurring task: {}", e)))?;
        
        Ok(CompleteTaskWithRecurrenceResponse {
            original_task: completed_task.into(),
            next_instance: next_instance.map(|t| t.into()),
        })
    }

    // ========================================
    // RBAC CRUD MUTATIONS - Admin Only
    // ========================================

    // Role Management
    async fn create_role(&self, ctx: &Context<'_>, input: CreateRoleInput) -> Result<Role> {
        use crate::auth::require_admin;
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        // Check if role name already exists
        let existing = crate::entities::role::Entity::find()
            .filter(crate::entities::role::Column::Name.eq(&input.name))
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?;
            
        if existing.is_some() {
            return Err(Error::new("Role name already exists"));
        }
        
        let new_role = crate::entities::role::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            name: Set(input.name),
            description: Set(input.description),
            level: Set(input.level),
            is_active: Set(true),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };
        
        let role = new_role
            .insert(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to create role: {}", e)))?;
            
        Ok(role.into())
    }

    async fn update_role(&self, ctx: &Context<'_>, input: UpdateRoleInput) -> Result<Role> {
        use crate::auth::require_admin;
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        // Get existing role
        let role = crate::entities::role::Entity::find_by_id(input.role_id)
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?
            .ok_or_else(|| Error::new("Role not found"))?;
            
        // Check if new name conflicts (if changing name)
        if let Some(ref new_name) = input.name {
            if new_name != &role.name {
                let existing = crate::entities::role::Entity::find()
                    .filter(crate::entities::role::Column::Name.eq(new_name))
                    .filter(crate::entities::role::Column::Id.ne(input.role_id))
                    .one(user_service.get_db())
                    .await
                    .map_err(|e| Error::new(format!("Database error: {}", e)))?;
                    
                if existing.is_some() {
                    return Err(Error::new("Role name already exists"));
                }
            }
        }
        
        let mut active_role: crate::entities::role::ActiveModel = role.into();
        
        if let Some(name) = input.name {
            active_role.name = Set(name);
        }
        if let Some(description) = input.description {
            active_role.description = Set(description);
        }
        if let Some(level) = input.level {
            active_role.level = Set(level);
        }
        if let Some(is_active) = input.is_active {
            active_role.is_active = Set(is_active);
        }
        active_role.updated_at = Set(Utc::now().into());
        
        let updated_role = active_role
            .update(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to update role: {}", e)))?;
            
        Ok(updated_role.into())
    }

    async fn delete_role(&self, ctx: &Context<'_>, role_id: uuid::Uuid) -> Result<MessageResponse> {
        use crate::auth::require_admin;
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        // Check if role exists
        let role = crate::entities::role::Entity::find_by_id(role_id)
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?
            .ok_or_else(|| Error::new("Role not found"))?;
            
        // Check if any users have this role
        let user_count = crate::entities::user::Entity::find()
            .filter(crate::entities::user::Column::RoleId.eq(role_id))
            .count(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?;
            
        if user_count > 0 {
            return Err(Error::new(format!("Cannot delete role '{}' as it is assigned to {} user(s)", role.name, user_count)));
        }
        
        // Use soft delete (set is_active to false) for safety
        let mut active_role: crate::entities::role::ActiveModel = role.into();
        active_role.is_active = Set(false);
        active_role.updated_at = Set(Utc::now().into());
        
        active_role
            .update(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to delete role: {}", e)))?;
            
        Ok(MessageResponse {
            message: "Role deleted successfully".to_string(),
        })
    }

    // Permission Management
    async fn create_permission(&self, ctx: &Context<'_>, input: CreatePermissionInput) -> Result<Permission> {
        use crate::auth::require_admin;
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        // Check if permission already exists for this resource
        let existing = crate::entities::permission::Entity::find()
            .filter(crate::entities::permission::Column::Action.eq(&input.action))
            .filter(crate::entities::permission::Column::ResourceId.eq(input.resource_id))
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?;
            
        if existing.is_some() {
            return Err(Error::new("Permission already exists for this resource"));
        }
        
        // Verify resource exists
        let _resource = crate::entities::resource::Entity::find_by_id(input.resource_id)
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?
            .ok_or_else(|| Error::new("Resource not found"))?;
        
        let new_permission = crate::entities::permission::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            action: Set(input.action),
            resource_id: Set(input.resource_id),
            description: Set(input.description),
            is_active: Set(true),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };
        
        let permission = new_permission
            .insert(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to create permission: {}", e)))?;
            
        Ok(permission.into())
    }

    async fn update_permission(&self, ctx: &Context<'_>, input: UpdatePermissionInput) -> Result<Permission> {
        use crate::auth::require_admin;
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        // Get existing permission
        let permission = crate::entities::permission::Entity::find_by_id(input.permission_id)
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?
            .ok_or_else(|| Error::new("Permission not found"))?;
            
        // Check for conflicts if changing action or resource
        if input.action.is_some() || input.resource_id.is_some() {
            let new_action = input.action.as_ref().unwrap_or(&permission.action);
            let new_resource_id = input.resource_id.unwrap_or(permission.resource_id);
            
            if new_action != &permission.action || new_resource_id != permission.resource_id {
                let existing = crate::entities::permission::Entity::find()
                    .filter(crate::entities::permission::Column::Action.eq(new_action))
                    .filter(crate::entities::permission::Column::ResourceId.eq(new_resource_id))
                    .filter(crate::entities::permission::Column::Id.ne(input.permission_id))
                    .one(user_service.get_db())
                    .await
                    .map_err(|e| Error::new(format!("Database error: {}", e)))?;
                    
                if existing.is_some() {
                    return Err(Error::new("Permission already exists for this resource"));
                }
            }
        }
        
        // Verify new resource exists if changing
        if let Some(resource_id) = input.resource_id {
            let _resource = crate::entities::resource::Entity::find_by_id(resource_id)
                .one(user_service.get_db())
                .await
                .map_err(|e| Error::new(format!("Database error: {}", e)))?
                .ok_or_else(|| Error::new("Resource not found"))?;
        }
        
        let mut active_permission: crate::entities::permission::ActiveModel = permission.into();
        
        if let Some(action) = input.action {
            active_permission.action = Set(action);
        }
        if let Some(resource_id) = input.resource_id {
            active_permission.resource_id = Set(resource_id);
        }
        if let Some(description) = input.description {
            active_permission.description = Set(description);
        }
        if let Some(is_active) = input.is_active {
            active_permission.is_active = Set(is_active);
        }
        active_permission.updated_at = Set(Utc::now().into());
        
        let updated_permission = active_permission
            .update(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to update permission: {}", e)))?;
            
        Ok(updated_permission.into())
    }

    async fn delete_permission(&self, ctx: &Context<'_>, permission_id: uuid::Uuid) -> Result<MessageResponse> {
        use crate::auth::require_admin;
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        // Check if permission exists
        let permission = crate::entities::permission::Entity::find_by_id(permission_id)
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?
            .ok_or_else(|| Error::new("Permission not found"))?;
            
        // Check if any roles have this permission
        let role_count = crate::entities::role_permission::Entity::find()
            .filter(crate::entities::role_permission::Column::PermissionId.eq(permission_id))
            .count(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?;
            
        // Check if any users have this permission directly
        let user_count = crate::entities::user_permission::Entity::find()
            .filter(crate::entities::user_permission::Column::PermissionId.eq(permission_id))
            .count(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?;
            
        if role_count > 0 || user_count > 0 {
            return Err(Error::new(format!("Cannot delete permission '{}' as it is assigned to {} role(s) and {} user(s)", permission.action, role_count, user_count)));
        }
        
        // Use soft delete (set is_active to false) for safety
        let mut active_permission: crate::entities::permission::ActiveModel = permission.into();
        active_permission.is_active = Set(false);
        active_permission.updated_at = Set(Utc::now().into());
        
        active_permission
            .update(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to delete permission: {}", e)))?;
            
        Ok(MessageResponse {
            message: "Permission deleted successfully".to_string(),
        })
    }

    // Resource Management
    async fn create_resource(&self, ctx: &Context<'_>, input: CreateResourceInput) -> Result<Resource> {
        use crate::auth::require_admin;
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        // Check if resource name already exists
        let existing = crate::entities::resource::Entity::find()
            .filter(crate::entities::resource::Column::Name.eq(&input.name))
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?;
            
        if existing.is_some() {
            return Err(Error::new("Resource name already exists"));
        }
        
        let new_resource = crate::entities::resource::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            name: Set(input.name),
            description: Set(input.description),
            is_active: Set(true),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };
        
        let resource = new_resource
            .insert(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to create resource: {}", e)))?;
            
        Ok(resource.into())
    }

    async fn update_resource(&self, ctx: &Context<'_>, input: UpdateResourceInput) -> Result<Resource> {
        use crate::auth::require_admin;
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        // Get existing resource
        let resource = crate::entities::resource::Entity::find_by_id(input.resource_id)
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?
            .ok_or_else(|| Error::new("Resource not found"))?;
            
        // Check if new name conflicts (if changing name)
        if let Some(ref new_name) = input.name {
            if new_name != &resource.name {
                let existing = crate::entities::resource::Entity::find()
                    .filter(crate::entities::resource::Column::Name.eq(new_name))
                    .filter(crate::entities::resource::Column::Id.ne(input.resource_id))
                    .one(user_service.get_db())
                    .await
                    .map_err(|e| Error::new(format!("Database error: {}", e)))?;
                    
                if existing.is_some() {
                    return Err(Error::new("Resource name already exists"));
                }
            }
        }
        
        let mut active_resource: crate::entities::resource::ActiveModel = resource.into();
        
        if let Some(name) = input.name {
            active_resource.name = Set(name);
        }
        if let Some(description) = input.description {
            active_resource.description = Set(description);
        }
        if let Some(is_active) = input.is_active {
            active_resource.is_active = Set(is_active);
        }
        active_resource.updated_at = Set(Utc::now().into());
        
        let updated_resource = active_resource
            .update(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to update resource: {}", e)))?;
            
        Ok(updated_resource.into())
    }

    async fn delete_resource(&self, ctx: &Context<'_>, resource_id: uuid::Uuid) -> Result<MessageResponse> {
        use crate::auth::require_admin;
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        // Check if resource exists
        let resource = crate::entities::resource::Entity::find_by_id(resource_id)
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?
            .ok_or_else(|| Error::new("Resource not found"))?;
            
        // Check if any permissions use this resource
        let permission_count = crate::entities::permission::Entity::find()
            .filter(crate::entities::permission::Column::ResourceId.eq(resource_id))
            .count(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?;
            
        if permission_count > 0 {
            return Err(Error::new(format!("Cannot delete resource '{}' as it has {} permission(s) associated with it", resource.name, permission_count)));
        }
        
        // Use soft delete (set is_active to false) for safety
        let mut active_resource: crate::entities::resource::ActiveModel = resource.into();
        active_resource.is_active = Set(false);
        active_resource.updated_at = Set(Utc::now().into());
        
        active_resource
            .update(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to delete resource: {}", e)))?;
            
        Ok(MessageResponse {
            message: "Resource deleted successfully".to_string(),
        })
    }

    // Role-Permission Assignment
    async fn assign_permission_to_role(&self, ctx: &Context<'_>, input: AssignPermissionToRoleInput) -> Result<MessageResponse> {
        use crate::auth::require_admin;
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        // Verify role and permission exist
        let _role = crate::entities::role::Entity::find_by_id(input.role_id)
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?
            .ok_or_else(|| Error::new("Role not found"))?;
            
        let _permission = crate::entities::permission::Entity::find_by_id(input.permission_id)
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?
            .ok_or_else(|| Error::new("Permission not found"))?;
        
        // Check if assignment already exists
        let existing = crate::entities::role_permission::Entity::find()
            .filter(crate::entities::role_permission::Column::RoleId.eq(input.role_id))
            .filter(crate::entities::role_permission::Column::PermissionId.eq(input.permission_id))
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?;
            
        if existing.is_some() {
            return Ok(MessageResponse {
                message: "Permission already assigned to role".to_string(),
            });
        }
        
        let role_permission = crate::entities::role_permission::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            role_id: Set(input.role_id),
            permission_id: Set(input.permission_id),
            created_at: Set(Utc::now().into()),
        };
        
        role_permission
            .insert(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to assign permission: {}", e)))?;
            
        Ok(MessageResponse {
            message: "Permission assigned to role successfully".to_string(),
        })
    }

    async fn remove_permission_from_role(&self, ctx: &Context<'_>, input: RemovePermissionFromRoleInput) -> Result<MessageResponse> {
        use crate::auth::require_admin;
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        let deleted = crate::entities::role_permission::Entity::delete_many()
            .filter(crate::entities::role_permission::Column::RoleId.eq(input.role_id))
            .filter(crate::entities::role_permission::Column::PermissionId.eq(input.permission_id))
            .exec(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to remove permission: {}", e)))?;
            
        if deleted.rows_affected == 0 {
            return Err(Error::new("Permission was not assigned to this role"));
        }
        
        Ok(MessageResponse {
            message: "Permission removed from role successfully".to_string(),
        })
    }

    // User Direct Permission Management
    async fn grant_user_permission(&self, ctx: &Context<'_>, input: GrantUserPermissionInput) -> Result<MessageResponse> {
        use crate::auth::require_admin;
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        // Verify user and permission exist
        let _user = crate::entities::user::Entity::find_by_id(input.user_id)
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?
            .ok_or_else(|| Error::new("User not found"))?;
            
        let _permission = crate::entities::permission::Entity::find_by_id(input.permission_id)
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?
            .ok_or_else(|| Error::new("Permission not found"))?;
        
        // Check if permission already granted
        let existing = crate::entities::user_permission::Entity::find()
            .filter(crate::entities::user_permission::Column::UserId.eq(input.user_id))
            .filter(crate::entities::user_permission::Column::PermissionId.eq(input.permission_id))
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?;
            
        if let Some(existing_perm) = existing {
            if existing_perm.is_granted {
                return Ok(MessageResponse {
                    message: "Permission already granted to user".to_string(),
                });
            } else {
                // Update existing record to grant permission
                let mut active_permission: crate::entities::user_permission::ActiveModel = existing_perm.into();
                active_permission.is_granted = Set(true);
                active_permission.updated_at = Set(Utc::now().into());
                
                active_permission
                    .update(user_service.get_db())
                    .await
                    .map_err(|e| Error::new(format!("Failed to grant permission: {}", e)))?;
            }
        } else {
            // Create new permission grant
            let user_permission = crate::entities::user_permission::ActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                user_id: Set(input.user_id),
                permission_id: Set(input.permission_id),
                is_granted: Set(true),
                created_at: Set(Utc::now().into()),
                updated_at: Set(Utc::now().into()),
            };
            
            user_permission
                .insert(user_service.get_db())
                .await
                .map_err(|e| Error::new(format!("Failed to grant permission: {}", e)))?;
        }
        
        Ok(MessageResponse {
            message: "Permission granted to user successfully".to_string(),
        })
    }

    async fn revoke_user_permission(&self, ctx: &Context<'_>, input: RevokeUserPermissionInput) -> Result<MessageResponse> {
        use crate::auth::require_admin;
        require_admin(ctx, "freshapi").await?;
        
        let user_service = ctx.data::<UserService>()?;
        
        // Find existing permission
        let existing = crate::entities::user_permission::Entity::find()
            .filter(crate::entities::user_permission::Column::UserId.eq(input.user_id))
            .filter(crate::entities::user_permission::Column::PermissionId.eq(input.permission_id))
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?;
            
        if let Some(existing_perm) = existing {
            if !existing_perm.is_granted {
                return Ok(MessageResponse {
                    message: "Permission already revoked from user".to_string(),
                });
            }
            
            // Update to revoke permission (set is_granted to false)
            let mut active_permission: crate::entities::user_permission::ActiveModel = existing_perm.into();
            active_permission.is_granted = Set(false);
            active_permission.updated_at = Set(Utc::now().into());
            
            active_permission
                .update(user_service.get_db())
                .await
                .map_err(|e| Error::new(format!("Failed to revoke permission: {}", e)))?;
        } else {
            return Err(Error::new("User does not have this permission"));
        }
        
        Ok(MessageResponse {
            message: "Permission revoked from user successfully".to_string(),
        })
    }

    // Comment system mutations
    async fn add_comment(&self, ctx: &Context<'_>, input: AddCommentInput) -> Result<Activity> {
        let auth_user = ctx.data::<crate::auth::AuthenticatedUser>()?;
        
        // Convert GraphQLEntityType to EntityType
        let entity_type = match input.entity_type {
            GraphQLEntityType::Task => EntityType::Task,
            GraphQLEntityType::Project => EntityType::Project,
            GraphQLEntityType::User => EntityType::User,
            GraphQLEntityType::Settings => EntityType::Settings,
        };

        // Verify user can access the entity they want to comment on
        match entity_type {
            EntityType::Task => {
                let task_service = ctx.data::<TaskService>()?;
                let can_access = task_service
                    .can_user_access_task(input.entity_id, auth_user.id)
                    .await
                    .map_err(|e| Error::new(format!("Failed to check task access: {}", e)))?;
                
                if !can_access {
                    return Err(Error::new("You don't have permission to comment on this task"));
                }
            },
            EntityType::Project => {
                let project_service = ctx.data::<ProjectService>()?;
                let can_access = project_service
                    .can_user_access_project(input.entity_id, auth_user.id)
                    .await
                    .map_err(|e| Error::new(format!("Failed to check project access: {}", e)))?;
                
                if !can_access {
                    return Err(Error::new("You don't have permission to comment on this project"));
                }
            },
            EntityType::User => {
                // Users can comment on user profiles if they have user_management permission
                use crate::auth::require_permission;
                require_permission(ctx, "freshapi", "user_management").await?;
            },
            EntityType::Settings => {
                // Only admins can comment on settings
                use crate::auth::require_admin;
                require_admin(ctx, "freshapi").await?;
            },
        }

        let activity_service = ctx.data::<ActivityService>()?;

        // Add comment through ActivityService
        let activity = activity_service
            .add_comment(
                entity_type,
                input.entity_id,
                auth_user.id,
                &input.content,
                input.mentions,
            )
            .await
            .map_err(|e| Error::new(format!("Failed to add comment: {}", e)))?;

        Ok(activity.into())
    }

    // ============================================================================
    // ProjectMind Context System Mutations
    // ============================================================================

    /// Create a new context category for a project
    async fn create_context_category(
        &self,
        ctx: &Context<'_>,
        input: crate::graphql::types::CreateContextCategoryInput,
    ) -> Result<crate::graphql::types::ProjectContextCategory> {
        let context_service = ctx.data::<crate::services::ContextService>()?;
        let authenticated_user = ctx.data::<AuthenticatedUser>()?;

        let category = context_service
            .create_context_category(input, Some(authenticated_user.id))
            .await
            .map_err(|e| Error::new(format!("Failed to create category: {}", e)))?;

        Ok(category.into())
    }

    /// Update an existing context category
    async fn update_context_category(
        &self,
        ctx: &Context<'_>,
        input: crate::graphql::types::UpdateContextCategoryInput,
    ) -> Result<crate::graphql::types::ProjectContextCategory> {
        let context_service = ctx.data::<crate::services::ContextService>()?;
        let _authenticated_user = ctx.data::<AuthenticatedUser>()?;

        let category = context_service
            .update_context_category(input)
            .await
            .map_err(|e| Error::new(format!("Failed to update category: {}", e)))?;

        Ok(category.into())
    }

    /// Delete (soft delete) a context category
    async fn delete_context_category(
        &self,
        ctx: &Context<'_>,
        category_id: Uuid,
    ) -> Result<crate::graphql::types::MessageResponse> {
        let context_service = ctx.data::<crate::services::ContextService>()?;
        let _authenticated_user = ctx.data::<AuthenticatedUser>()?;

        context_service
            .delete_context_category(category_id)
            .await
            .map_err(|e| Error::new(format!("Failed to delete category: {}", e)))?;

        Ok(crate::graphql::types::MessageResponse {
            message: "Category deleted successfully".to_string(),
        })
    }

    /// Ingest email context (webhook endpoint)
    async fn ingest_email_context(
        &self,
        ctx: &Context<'_>,
        input: crate::graphql::types::EmailIngestInput,
    ) -> Result<crate::graphql::types::EmailContext> {
        let email_service = ctx.data::<crate::services::EmailContextService>()?;
        // Note: This endpoint may be called without authentication for webhook usage
        
        let email = email_service
            .ingest_email(input)
            .await
            .map_err(|e| Error::new(format!("Failed to ingest email: {}", e)))?;

        Ok(email.into())
    }

    /// Update email processing status
    async fn update_email_processing_status(
        &self,
        ctx: &Context<'_>,
        email_id: Uuid,
        status: crate::graphql::types::ProcessingStatus,
        notes: Option<String>,
    ) -> Result<crate::graphql::types::EmailContext> {
        let email_service = ctx.data::<crate::services::EmailContextService>()?;
        let _authenticated_user = ctx.data::<AuthenticatedUser>()?;

        let email = email_service
            .update_processing_status(email_id, status, notes)
            .await
            .map_err(|e| Error::new(format!("Failed to update processing status: {}", e)))?;

        Ok(email.into())
    }

    /// Archive a project context
    async fn archive_context(
        &self,
        ctx: &Context<'_>,
        context_id: Uuid,
    ) -> Result<crate::graphql::types::ProjectContext> {
        let context_service = ctx.data::<crate::services::ContextService>()?;
        let _authenticated_user = ctx.data::<AuthenticatedUser>()?;

        let context = context_service
            .archive_context(context_id)
            .await
            .map_err(|e| Error::new(format!("Failed to archive context: {}", e)))?;

        Ok(context.into())
    }

    /// Restore an archived project context
    async fn restore_context(
        &self,
        ctx: &Context<'_>,
        context_id: Uuid,
    ) -> Result<crate::graphql::types::ProjectContext> {
        let context_service = ctx.data::<crate::services::ContextService>()?;
        let _authenticated_user = ctx.data::<AuthenticatedUser>()?;

        let context = context_service
            .restore_context(context_id)
            .await
            .map_err(|e| Error::new(format!("Failed to restore context: {}", e)))?;

        Ok(context.into())
    }

    /// Create a task directly from a context element
    async fn create_task_from_context(
        &self,
        ctx: &Context<'_>,
        input: crate::graphql::types::CreateTaskFromContextInput,
    ) -> Result<crate::graphql::types::Task> {
        let task_service = ctx.data::<crate::services::TaskService>()?;
        let authenticated_user = ctx.data::<AuthenticatedUser>()?;

        let task = task_service
            .create_task_from_context(input, authenticated_user.id)
            .await
            .map_err(|e| Error::new(format!("Failed to create task from context: {}", e)))?;

        Ok(task.into())
    }
}