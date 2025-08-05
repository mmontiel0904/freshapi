use async_graphql::*;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, PaginatorTrait};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_email_verified: bool,
    #[graphql(skip)]
    pub role_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::entities::user::Model> for User {
    fn from(user: crate::entities::user::Model) -> Self {
        Self {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            is_email_verified: user.is_email_verified,
            role_id: user.role_id,
            created_at: user.created_at.into(),
            updated_at: user.updated_at.into(),
        }
    }
}

#[ComplexObject]
impl User {
    async fn role(&self, ctx: &Context<'_>) -> Result<Option<Role>> {
        if let Some(role_id) = self.role_id {
            let user_service = ctx.data::<crate::services::UserService>()?;
            
            let role = crate::entities::role::Entity::find_by_id(role_id)
                .one(user_service.get_db())
                .await
                .map_err(|e| Error::new(format!("Failed to fetch role: {}", e)))?;
                
            Ok(role.map(|r| r.into()))
        } else {
            Ok(None)
        }
    }

    async fn permissions(&self, ctx: &Context<'_>) -> Result<Vec<String>> {
        let dataloader = ctx.data::<crate::graphql::DataLoaderContext>()?;
        
        dataloader
            .load_user_permissions(self.id)
            .await
            .map_err(|e| Error::new(format!("Failed to fetch permissions: {}", e)))
    }
}

#[derive(InputObject)]
pub struct RegisterInput {
    pub email: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(InputObject)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(SimpleObject)]
pub struct AuthPayload {
    pub user: User,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(InputObject)]
pub struct RefreshTokenInput {
    pub refresh_token: String,
}

#[derive(SimpleObject)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(SimpleObject)]
pub struct Invitation {
    pub id: Uuid,
    pub email: String,
    pub inviter_user_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub is_used: bool,
    pub used_at: Option<DateTime<Utc>>,
    pub role: Option<Role>,
    pub created_at: DateTime<Utc>,
}

impl From<crate::entities::invitation::Model> for Invitation {
    fn from(invitation: crate::entities::invitation::Model) -> Self {
        Self {
            id: invitation.id,
            email: invitation.email,
            inviter_user_id: invitation.inviter_user_id,
            expires_at: invitation.expires_at.into(),
            is_used: invitation.is_used,
            used_at: invitation.used_at.map(|dt| dt.into()),
            role: None, // Will be populated by resolver when needed
            created_at: invitation.created_at.into(),
        }
    }
}

#[derive(InputObject)]
pub struct InviteUserInput {
    pub email: String,
}

#[derive(InputObject)]
pub struct AcceptInvitationInput {
    pub invitation_token: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(InputObject)]
pub struct RequestPasswordResetInput {
    pub email: String,
}

#[derive(InputObject)]
pub struct ResetPasswordInput {
    pub token: String,
    pub new_password: String,
}

#[derive(InputObject)]
pub struct ChangePasswordInput {
    pub current_password: String,
    pub new_password: String,
}

#[derive(InputObject)]
pub struct AdminResetUserPasswordInput {
    pub user_id: Uuid,
}

// RBAC Types
#[derive(SimpleObject)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub level: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::entities::role::Model> for Role {
    fn from(role: crate::entities::role::Model) -> Self {
        Self {
            id: role.id,
            name: role.name,
            description: role.description,
            level: role.level,
            is_active: role.is_active,
            created_at: role.created_at.into(),
            updated_at: role.updated_at.into(),
        }
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Permission {
    pub id: Uuid,
    pub action: String,
    pub resource_id: Uuid,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::entities::permission::Model> for Permission {
    fn from(permission: crate::entities::permission::Model) -> Self {
        Self {
            id: permission.id,
            action: permission.action,
            resource_id: permission.resource_id,
            description: permission.description,
            is_active: permission.is_active,
            created_at: permission.created_at.into(),
            updated_at: permission.updated_at.into(),
        }
    }
}

#[ComplexObject]
impl Permission {
    async fn resource(&self, ctx: &Context<'_>) -> Result<Option<Resource>> {
        let user_service = ctx.data::<crate::services::UserService>()?;
        
        let resource = crate::entities::resource::Entity::find_by_id(self.resource_id)
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to fetch resource: {}", e)))?;
            
        Ok(resource.map(|r| r.into()))
    }

    async fn resource_name(&self, ctx: &Context<'_>) -> Result<String> {
        let user_service = ctx.data::<crate::services::UserService>()?;
        
        let resource = crate::entities::resource::Entity::find_by_id(self.resource_id)
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to fetch resource: {}", e)))?
            .ok_or_else(|| Error::new("Resource not found"))?;
            
        Ok(resource.name)
    }
}

#[derive(SimpleObject)]
pub struct Resource {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::entities::resource::Model> for Resource {
    fn from(resource: crate::entities::resource::Model) -> Self {
        Self {
            id: resource.id,
            name: resource.name,
            description: resource.description,
            is_active: resource.is_active,
            created_at: resource.created_at.into(),
            updated_at: resource.updated_at.into(),
        }
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct RoleWithPermissions {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub level: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::entities::role::Model> for RoleWithPermissions {
    fn from(role: crate::entities::role::Model) -> Self {
        Self {
            id: role.id,
            name: role.name,
            description: role.description,
            level: role.level,
            is_active: role.is_active,
            created_at: role.created_at.into(),
            updated_at: role.updated_at.into(),
        }
    }
}

#[ComplexObject]
impl RoleWithPermissions {
    async fn permissions(&self, ctx: &Context<'_>) -> Result<Vec<Permission>> {
        let user_service = ctx.data::<crate::services::UserService>()?;
        
        use sea_orm::{JoinType, QuerySelect, RelationTrait};
        
        let permissions = crate::entities::role_permission::Entity::find()
            .filter(crate::entities::role_permission::Column::RoleId.eq(self.id))
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
            .map_err(|e| Error::new(format!("Failed to fetch permissions: {}", e)))?;
            
        Ok(permissions.into_iter().map(|p| p.into()).collect())
    }

    async fn user_count(&self, ctx: &Context<'_>) -> Result<u32> {
        let user_service = ctx.data::<crate::services::UserService>()?;
        
        let count = crate::entities::user::Entity::find()
            .filter(crate::entities::user::Column::RoleId.eq(self.id))
            .count(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to count users: {}", e)))?;
            
        Ok(count as u32)
    }
}

#[derive(SimpleObject)]
pub struct UserWithRole {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_email_verified: bool,
    pub role: Option<Role>,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(InputObject)]
pub struct AssignRoleInput {
    pub user_id: Uuid,
    pub role_id: Uuid,
}

#[derive(InputObject)]
pub struct InviteUserWithRoleInput {
    pub email: String,
    pub role_id: Option<Uuid>,
}

// Role CRUD Input Types
#[derive(InputObject)]
pub struct CreateRoleInput {
    pub name: String,
    pub description: Option<String>,
    pub level: i32,
}

#[derive(InputObject)]
pub struct UpdateRoleInput {
    pub role_id: Uuid,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub level: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(InputObject)]
pub struct AssignPermissionToRoleInput {
    pub role_id: Uuid,
    pub permission_id: Uuid,
}

#[derive(InputObject)]
pub struct RemovePermissionFromRoleInput {
    pub role_id: Uuid,
    pub permission_id: Uuid,
}

// Permission CRUD Input Types
#[derive(InputObject)]
pub struct CreatePermissionInput {
    pub action: String,
    pub resource_id: Uuid,
    pub description: Option<String>,
}

#[derive(InputObject)]
pub struct UpdatePermissionInput {
    pub permission_id: Uuid,
    pub action: Option<String>,
    pub resource_id: Option<Uuid>,
    pub description: Option<Option<String>>,
    pub is_active: Option<bool>,
}

// Resource CRUD Input Types
#[derive(InputObject)]
pub struct CreateResourceInput {
    pub name: String,
    pub description: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateResourceInput {
    pub resource_id: Uuid,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub is_active: Option<bool>,
}

// User Permission Management
#[derive(InputObject)]
pub struct GrantUserPermissionInput {
    pub user_id: Uuid,
    pub permission_id: Uuid,
}

#[derive(InputObject)]
pub struct RevokeUserPermissionInput {
    pub user_id: Uuid,
    pub permission_id: Uuid,
}

// Project Types
#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::entities::project::Model> for Project {
    fn from(project: crate::entities::project::Model) -> Self {
        Self {
            id: project.id,
            name: project.name,
            description: project.description,
            owner_id: project.owner_id,
            is_active: project.is_active,
            created_at: project.created_at.into(),
            updated_at: project.updated_at.into(),
        }
    }
}

#[ComplexObject]
impl Project {
    async fn owner(&self, ctx: &Context<'_>) -> Result<Option<User>> {
        let user_service = ctx.data::<crate::services::UserService>()?;
        
        let user = crate::entities::user::Entity::find_by_id(self.owner_id)
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to fetch owner: {}", e)))?;
            
        Ok(user.map(|u| u.into()))
    }

    async fn members(&self, ctx: &Context<'_>) -> Result<Vec<ProjectMember>> {
        let project_service = ctx.data::<crate::services::ProjectService>()?;
        let current_user = ctx.data::<crate::entities::user::Model>()?;
        
        let members = project_service
            .get_project_members(self.id, current_user.id)
            .await
            .map_err(|e| Error::new(format!("Failed to fetch members: {}", e)))?;
            
        Ok(members.into_iter().map(|(member, user)| ProjectMember {
            id: member.id,
            project_id: member.project_id,
            user_id: member.user_id,
            role: member.role,
            joined_at: member.joined_at.into(),
            user: user.into(),
        }).collect())
    }

    async fn tasks(&self, ctx: &Context<'_>, status: Option<String>, assignee_id: Option<Uuid>, limit: Option<i32>, offset: Option<i32>) -> Result<Vec<Task>> {
        let task_service = ctx.data::<crate::services::TaskService>()?;
        let current_user = ctx.data::<crate::entities::user::Model>()?;
        
        let status_filter = status.and_then(|s| crate::services::TaskStatus::from_str(&s));
        
        let tasks = task_service
            .get_project_tasks(
                self.id,
                current_user.id,
                status_filter,
                assignee_id,
                limit.map(|l| l.max(0) as u64),
                offset.map(|o| o.max(0) as u64),
            )
            .await
            .map_err(|e| Error::new(format!("Failed to fetch tasks: {}", e)))?;
            
        Ok(tasks.into_iter().map(|t| t.into()).collect())
    }
}

#[derive(SimpleObject)]
pub struct ProjectMember {
    pub id: Uuid,
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
    pub user: User,
}

#[derive(InputObject)]
pub struct CreateProjectInput {
    pub name: String,
    pub description: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateProjectInput {
    pub project_id: Uuid,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
}

#[derive(InputObject)]
pub struct AddProjectMemberInput {
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
}

#[derive(InputObject)]
pub struct UpdateMemberRoleInput {
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
}

#[derive(InputObject)]
pub struct RemoveProjectMemberInput {
    pub project_id: Uuid,
    pub user_id: Uuid,
}

// Task Types
#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub project_id: Uuid,
    pub assignee_id: Option<Uuid>,
    pub creator_id: Uuid,
    pub status: String,
    pub priority: String,
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::entities::task::Model> for Task {
    fn from(task: crate::entities::task::Model) -> Self {
        Self {
            id: task.id,
            name: task.name,
            description: task.description,
            project_id: task.project_id,
            assignee_id: task.assignee_id,
            creator_id: task.creator_id,
            status: task.status,
            priority: task.priority,
            due_date: task.due_date.map(|dt| dt.into()),
            created_at: task.created_at.into(),
            updated_at: task.updated_at.into(),
        }
    }
}

#[ComplexObject]
impl Task {
    async fn project(&self, ctx: &Context<'_>) -> Result<Option<Project>> {
        let project_service = ctx.data::<crate::services::ProjectService>()?;
        let current_user = ctx.data::<crate::entities::user::Model>()?;
        
        let project = project_service
            .get_project(self.project_id, current_user.id)
            .await
            .map_err(|e| Error::new(format!("Failed to fetch project: {}", e)))?;
            
        Ok(project.map(|p| p.into()))
    }

    async fn assignee(&self, ctx: &Context<'_>) -> Result<Option<User>> {
        if let Some(assignee_id) = self.assignee_id {
            let user_service = ctx.data::<crate::services::UserService>()?;
            
            let user = crate::entities::user::Entity::find_by_id(assignee_id)
                .one(user_service.get_db())
                .await
                .map_err(|e| Error::new(format!("Failed to fetch assignee: {}", e)))?;
                
            Ok(user.map(|u| u.into()))
        } else {
            Ok(None)
        }
    }

    async fn creator(&self, ctx: &Context<'_>) -> Result<Option<User>> {
        let user_service = ctx.data::<crate::services::UserService>()?;
        
        let user = crate::entities::user::Entity::find_by_id(self.creator_id)
            .one(user_service.get_db())
            .await
            .map_err(|e| Error::new(format!("Failed to fetch creator: {}", e)))?;
            
        Ok(user.map(|u| u.into()))
    }
}

#[derive(InputObject)]
pub struct CreateTaskInput {
    pub project_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub assignee_id: Option<Uuid>,
    pub priority: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(InputObject)]
pub struct UpdateTaskInput {
    pub task_id: Uuid,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub due_date: Option<Option<DateTime<Utc>>>,
}

#[derive(InputObject)]
pub struct AssignTaskInput {
    pub task_id: Uuid,
    pub assignee_id: Option<Uuid>,
}

#[derive(SimpleObject)]
pub struct TaskStats {
    pub total: u32,
    pub todo: u32,
    pub in_progress: u32,
    pub completed: u32,
    pub cancelled: u32,
    pub overdue: u32,
}